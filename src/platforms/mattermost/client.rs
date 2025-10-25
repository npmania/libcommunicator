use reqwest::Client;
use std::sync::Arc;
use tokio::sync::RwLock;
use url::Url;

use crate::error::{Error, ErrorCode, Result};
use crate::types::{ConnectionInfo, ConnectionState};

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

        if status.is_success() {
            response
                .json::<T>()
                .await
                .map_err(|e| Error::new(ErrorCode::Unknown, format!("Failed to parse response: {e}")))
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
}
