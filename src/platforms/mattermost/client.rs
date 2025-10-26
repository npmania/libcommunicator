use reqwest::Client;
use std::sync::Arc;
use tokio::sync::RwLock;
use url::Url;

use crate::error::{Error, ErrorCode, Result};
use crate::types::{ConnectionInfo, ConnectionState};

/// Rate limit information from Mattermost API response headers
#[derive(Debug, Clone)]
pub struct RateLimitInfo {
    /// Maximum requests allowed per second
    pub limit: u32,
    /// Requests remaining in current window
    pub remaining: u32,
    /// UTC epoch seconds when the limit resets
    pub reset_at: u64,
}

/// Mattermost client for interacting with Mattermost servers
pub struct MattermostClient {
    /// HTTP client for REST API calls
    pub(crate) http_client: Client,
    /// Base URL for the Mattermost server (e.g., "https://mattermost.example.com")
    base_url: Url,
    /// Authentication token (session token or Personal Access Token)
    token: Arc<RwLock<Option<String>>>,
    /// Current connection state
    state: Arc<RwLock<ConnectionState>>,
    /// Team ID (workspace) we're connected to
    team_id: Arc<RwLock<Option<String>>>,
    /// Current user ID after authentication
    user_id: Arc<RwLock<Option<String>>>,
    /// Rate limit information from last API response
    rate_limit_info: Arc<RwLock<Option<RateLimitInfo>>>,
}

impl MattermostClient {
    /// Create a new Mattermost client
    ///
    /// # Arguments
    /// * `base_url` - The base URL of the Mattermost server (e.g., "https://mattermost.example.com")
    ///
    /// # Returns
    /// A Result containing the MattermostClient or an Error
    pub fn new(base_url: &str) -> Result<Self> {
        let base_url = Url::parse(base_url)
            .map_err(|e| Error::new(ErrorCode::InvalidArgument, format!("Invalid URL: {e}")))?;

        let http_client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| Error::new(ErrorCode::NetworkError, format!("Failed to create HTTP client: {e}")))?;

        Ok(Self {
            http_client,
            base_url,
            token: Arc::new(RwLock::new(None)),
            state: Arc::new(RwLock::new(ConnectionState::Disconnected)),
            team_id: Arc::new(RwLock::new(None)),
            user_id: Arc::new(RwLock::new(None)),
            rate_limit_info: Arc::new(RwLock::new(None)),
        })
    }

    /// Set the authentication token (session token or Personal Access Token)
    pub async fn set_token(&self, token: String) {
        let mut t = self.token.write().await;
        *t = Some(token);
    }

    /// Get the current authentication token
    pub async fn get_token(&self) -> Option<String> {
        self.token.read().await.clone()
    }

    /// Set the team ID
    pub async fn set_team_id(&self, team_id: Option<String>) {
        let mut t = self.team_id.write().await;
        *t = team_id;
    }

    /// Get the current team ID
    pub async fn get_team_id(&self) -> Option<String> {
        self.team_id.read().await.clone()
    }

    /// Set the user ID
    pub async fn set_user_id(&self, user_id: Option<String>) {
        let mut u = self.user_id.write().await;
        *u = user_id;
    }

    /// Get the current user ID
    pub async fn get_user_id(&self) -> Option<String> {
        self.user_id.read().await.clone()
    }

    /// Get the current user ID, returning an error if not authenticated
    pub async fn current_user_id(&self) -> Result<String> {
        self.get_user_id().await.ok_or_else(|| {
            Error::new(
                ErrorCode::InvalidState,
                "Not authenticated - no user ID available",
            )
        })
    }

    /// Update the connection state
    pub async fn set_state(&self, state: ConnectionState) {
        let mut s = self.state.write().await;
        *s = state;
    }

    /// Get the current connection state
    pub async fn get_state(&self) -> ConnectionState {
        *self.state.read().await
    }

    /// Get connection information
    pub async fn connection_info(&self, server_url: &str, user_display_name: &str) -> ConnectionInfo {
        let state = self.get_state().await;
        let user_id = self.user_id.read().await.clone().unwrap_or_default();
        let team_id = self.team_id.read().await.clone();

        let mut info = ConnectionInfo::new(
            "mattermost",
            server_url,
            user_id,
            user_display_name,
        ).with_state(state);

        if let Some(tid) = team_id {
            info = info.with_team(tid, "");
        }

        info
    }

    /// Get the current rate limit information
    ///
    /// # Returns
    /// The most recent rate limit info from API responses, or None if no requests have been made yet
    pub async fn get_rate_limit_info(&self) -> Option<RateLimitInfo> {
        self.rate_limit_info.read().await.clone()
    }

    /// Extract rate limit information from response headers
    ///
    /// # Arguments
    /// * `response` - The HTTP response containing rate limit headers
    ///
    /// # Returns
    /// RateLimitInfo if all headers are present and valid, None otherwise
    fn extract_rate_limit_info(&self, response: &reqwest::Response) -> Option<RateLimitInfo> {
        let headers = response.headers();

        let limit = headers
            .get("X-Ratelimit-Limit")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u32>().ok())?;

        let remaining = headers
            .get("X-Ratelimit-Remaining")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u32>().ok())?;

        let reset_at = headers
            .get("X-Ratelimit-Reset")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u64>().ok())?;

        Some(RateLimitInfo {
            limit,
            remaining,
            reset_at,
        })
    }

    /// Update stored rate limit info from a response
    async fn update_rate_limit_info(&self, response: &reqwest::Response) {
        if let Some(info) = self.extract_rate_limit_info(response) {
            let mut rate_limit = self.rate_limit_info.write().await;
            *rate_limit = Some(info);
        }
    }

    /// Retry an operation with exponential backoff when rate limited
    ///
    /// # Arguments
    /// * `operation` - The async operation to retry
    /// * `max_retries` - Maximum number of retry attempts (default: 3)
    ///
    /// # Returns
    /// Result from the operation, or the last error if all retries failed
    pub async fn with_retry<F, T, Fut>(&self, operation: F, max_retries: u32) -> Result<T>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        let mut retries = 0;
        let mut backoff_ms = 1000u64; // Start with 1 second

        loop {
            match operation().await {
                Ok(result) => return Ok(result),
                Err(e) if e.code == ErrorCode::RateLimited && retries < max_retries => {
                    retries += 1;

                    // Use exponential backoff: 1s, 2s, 4s, 8s, etc.
                    tokio::time::sleep(tokio::time::Duration::from_millis(backoff_ms)).await;
                    backoff_ms = backoff_ms.saturating_mul(2).min(30000); // Cap at 30 seconds
                }
                Err(e) => return Err(e),
            }
        }
    }

    /// Build the full API URL for a given endpoint
    ///
    /// # Arguments
    /// * `endpoint` - The API endpoint path (e.g., "/users/me")
    ///
    /// # Returns
    /// The full URL string
    pub fn api_url(&self, endpoint: &str) -> String {
        let endpoint = endpoint.trim_start_matches('/');
        let base = self.base_url.as_str().trim_end_matches('/');
        format!("{base}/api/v4/{endpoint}")
    }

    /// Make a GET request to the Mattermost API
    ///
    /// # Arguments
    /// * `endpoint` - The API endpoint path
    ///
    /// # Returns
    /// A Result containing the reqwest::Response or an Error
    pub async fn get(&self, endpoint: &str) -> Result<reqwest::Response> {
        let url = self.api_url(endpoint);
        let mut request = self.http_client.get(&url);

        if let Some(token) = self.get_token().await {
            request = request.bearer_auth(token);
        }

        request
            .send()
            .await
            .map_err(|e| Error::new(ErrorCode::NetworkError, format!("GET request failed: {e}")))
    }

    /// Make a POST request to the Mattermost API
    ///
    /// # Arguments
    /// * `endpoint` - The API endpoint path
    /// * `body` - The request body (will be serialized to JSON)
    ///
    /// # Returns
    /// A Result containing the reqwest::Response or an Error
    pub async fn post<T: serde::Serialize>(&self, endpoint: &str, body: &T) -> Result<reqwest::Response> {
        let url = self.api_url(endpoint);
        let mut request = self.http_client.post(&url);

        if let Some(token) = self.get_token().await {
            request = request.bearer_auth(token);
        }

        request
            .json(body)
            .send()
            .await
            .map_err(|e| Error::new(ErrorCode::NetworkError, format!("POST request failed: {e}")))
    }

    /// Make a PUT request to the Mattermost API
    ///
    /// # Arguments
    /// * `endpoint` - The API endpoint path
    /// * `body` - The request body (will be serialized to JSON)
    ///
    /// # Returns
    /// A Result containing the reqwest::Response or an Error
    pub async fn put<T: serde::Serialize>(&self, endpoint: &str, body: &T) -> Result<reqwest::Response> {
        let url = self.api_url(endpoint);
        let mut request = self.http_client.put(&url);

        if let Some(token) = self.get_token().await {
            request = request.bearer_auth(token);
        }

        request
            .json(body)
            .send()
            .await
            .map_err(|e| Error::new(ErrorCode::NetworkError, format!("PUT request failed: {e}")))
    }

    /// Make a DELETE request to the Mattermost API
    ///
    /// # Arguments
    /// * `endpoint` - The API endpoint path
    ///
    /// # Returns
    /// A Result containing the reqwest::Response or an Error
    pub async fn delete(&self, endpoint: &str) -> Result<reqwest::Response> {
        let url = self.api_url(endpoint);
        let mut request = self.http_client.delete(&url);

        if let Some(token) = self.get_token().await {
            request = request.bearer_auth(token);
        }

        request
            .send()
            .await
            .map_err(|e| Error::new(ErrorCode::NetworkError, format!("DELETE request failed: {e}")))
    }

    /// Check if the response is successful and extract the JSON body
    ///
    /// # Arguments
    /// * `response` - The HTTP response from the API
    ///
    /// # Returns
    /// A Result containing the deserialized response body or an Error
    pub async fn handle_response<T: serde::de::DeserializeOwned>(&self, response: reqwest::Response) -> Result<T> {
        let status = response.status();

        // Extract and store rate limit info from headers
        self.update_rate_limit_info(&response).await;

        if status.is_success() {
            response
                .json::<T>()
                .await
                .map_err(|e| Error::new(ErrorCode::Unknown, format!("Failed to parse response: {e}")))
        } else if status.as_u16() == 429 {
            // Rate limited - return specific error
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Rate limit exceeded".to_string());

            Err(Error::new(
                ErrorCode::RateLimited,
                format!("Rate limit exceeded: {error_text}"),
            ))
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            Err(Error::new(
                ErrorCode::NetworkError,
                format!("API request failed with status {status}: {error_text}"),
            ))
        }
    }

    /// Get a list of custom emojis
    ///
    /// # Arguments
    /// * `page` - The page to select (default: 0)
    /// * `per_page` - The number of emojis per page (default: 60, max: 200)
    /// * `sort` - Either empty string for no sorting or "name" to sort by emoji names
    ///
    /// # Returns
    /// A Result containing a Vec of MattermostEmoji or an Error
    pub async fn get_emojis(&self, page: u32, per_page: u32, sort: &str) -> Result<Vec<super::types::MattermostEmoji>> {
        let endpoint = format!("/emoji?page={}&per_page={}&sort={}", page, per_page, sort);
        let response = self.get(&endpoint).await?;
        self.handle_response(response).await
    }

    /// Get a custom emoji by ID
    ///
    /// # Arguments
    /// * `emoji_id` - The ID of the emoji
    ///
    /// # Returns
    /// A Result containing the MattermostEmoji or an Error
    pub async fn get_emoji_by_id(&self, emoji_id: &str) -> Result<super::types::MattermostEmoji> {
        let endpoint = format!("/emoji/{}", emoji_id);
        let response = self.get(&endpoint).await?;
        self.handle_response(response).await
    }

    /// Get a custom emoji by name
    ///
    /// # Arguments
    /// * `emoji_name` - The name of the emoji (without colons)
    ///
    /// # Returns
    /// A Result containing the MattermostEmoji or an Error
    pub async fn get_emoji_by_name(&self, emoji_name: &str) -> Result<super::types::MattermostEmoji> {
        let endpoint = format!("/emoji/name/{}", emoji_name);
        let response = self.get(&endpoint).await?;
        self.handle_response(response).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_client() {
        let client = MattermostClient::new("https://mattermost.example.com");
        assert!(client.is_ok());
    }

    #[test]
    fn test_invalid_url() {
        let client = MattermostClient::new("not a url");
        assert!(client.is_err());
    }

    #[test]
    fn test_api_url() {
        let client = MattermostClient::new("https://mattermost.example.com").unwrap();
        assert_eq!(
            client.api_url("/users/me"),
            "https://mattermost.example.com/api/v4/users/me"
        );
        assert_eq!(
            client.api_url("users/me"),
            "https://mattermost.example.com/api/v4/users/me"
        );
    }

    #[tokio::test]
    async fn test_token_management() {
        let client = MattermostClient::new("https://mattermost.example.com").unwrap();

        assert!(client.get_token().await.is_none());

        client.set_token("test_token".to_string()).await;
        assert_eq!(client.get_token().await, Some("test_token".to_string()));
    }

    #[tokio::test]
    async fn test_state_management() {
        let client = MattermostClient::new("https://mattermost.example.com").unwrap();

        assert_eq!(client.get_state().await, ConnectionState::Disconnected);

        client.set_state(ConnectionState::Connected).await;
        assert_eq!(client.get_state().await, ConnectionState::Connected);
    }

    #[test]
    fn test_rate_limit_info_creation() {
        let info = RateLimitInfo {
            limit: 100,
            remaining: 50,
            reset_at: 1234567890,
        };

        assert_eq!(info.limit, 100);
        assert_eq!(info.remaining, 50);
        assert_eq!(info.reset_at, 1234567890);
    }

    #[tokio::test]
    async fn test_rate_limit_tracking() {
        let client = MattermostClient::new("https://mattermost.example.com").unwrap();

        // Initially no rate limit info
        assert!(client.get_rate_limit_info().await.is_none());

        // Simulate setting rate limit info (would normally come from headers)
        let info = RateLimitInfo {
            limit: 100,
            remaining: 95,
            reset_at: 1234567890,
        };

        {
            let mut rate_limit = client.rate_limit_info.write().await;
            *rate_limit = Some(info.clone());
        }

        // Verify we can retrieve it
        let retrieved = client.get_rate_limit_info().await;
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.limit, 100);
        assert_eq!(retrieved.remaining, 95);
        assert_eq!(retrieved.reset_at, 1234567890);
    }
}
