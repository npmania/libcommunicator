use reqwest::Client;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use url::Url;

use crate::error::{Error, ErrorCode, Result};
use crate::types::{ConnectionInfo, ConnectionState};

use super::cache::Cache;
use super::types::{MattermostChannel, MattermostTeam, MattermostUser};

/// Configuration for caching API responses
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Time-to-live for user cache entries (default: 5 minutes)
    pub user_ttl: Duration,
    /// Time-to-live for channel cache entries (default: 2 minutes)
    pub channel_ttl: Duration,
    /// Time-to-live for team cache entries (default: 10 minutes)
    pub team_ttl: Duration,
    /// Enable caching (default: true)
    pub enable_cache: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            user_ttl: Duration::from_secs(300),    // 5 minutes
            channel_ttl: Duration::from_secs(120), // 2 minutes
            team_ttl: Duration::from_secs(600),    // 10 minutes
            enable_cache: true,
        }
    }
}

impl CacheConfig {
    /// Create a configuration with caching disabled
    pub fn disabled() -> Self {
        Self {
            enable_cache: false,
            ..Default::default()
        }
    }
}

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
    /// Cache for user objects
    user_cache: Cache<MattermostUser>,
    /// Cache for channel objects
    channel_cache: Cache<MattermostChannel>,
    /// Cache for team objects
    team_cache: Cache<MattermostTeam>,
    /// Cache configuration
    cache_config: CacheConfig,
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
        Self::with_cache_config(base_url, CacheConfig::default())
    }

    /// Create a new Mattermost client with custom cache configuration
    ///
    /// # Arguments
    /// * `base_url` - The base URL of the Mattermost server
    /// * `cache_config` - Cache configuration
    ///
    /// # Returns
    /// A Result containing the MattermostClient or an Error
    pub fn with_cache_config(base_url: &str, cache_config: CacheConfig) -> Result<Self> {
        let base_url = Url::parse(base_url)
            .map_err(|e| Error::new(ErrorCode::InvalidArgument, format!("Invalid URL: {e}")))?;

        let http_client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| {
                Error::new(
                    ErrorCode::NetworkError,
                    format!("Failed to create HTTP client: {e}"),
                )
            })?;

        Ok(Self {
            http_client,
            base_url,
            token: Arc::new(RwLock::new(None)),
            state: Arc::new(RwLock::new(ConnectionState::Disconnected)),
            team_id: Arc::new(RwLock::new(None)),
            user_id: Arc::new(RwLock::new(None)),
            rate_limit_info: Arc::new(RwLock::new(None)),
            user_cache: Cache::new(cache_config.user_ttl),
            channel_cache: Cache::new(cache_config.channel_ttl),
            team_cache: Cache::new(cache_config.team_ttl),
            cache_config,
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

    /// Get the base URL of the Mattermost server
    pub fn get_base_url(&self) -> &str {
        self.base_url.as_str()
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
    pub async fn connection_info(
        &self,
        server_url: &str,
        user_display_name: &str,
    ) -> ConnectionInfo {
        let state = self.get_state().await;
        let user_id = self.user_id.read().await.clone().unwrap_or_default();
        let team_id = self.team_id.read().await.clone();

        let mut info = ConnectionInfo::new("mattermost", server_url, user_id, user_display_name)
            .with_state(state);

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
    pub async fn post<T: serde::Serialize>(
        &self,
        endpoint: &str,
        body: &T,
    ) -> Result<reqwest::Response> {
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
    pub async fn put<T: serde::Serialize>(
        &self,
        endpoint: &str,
        body: &T,
    ) -> Result<reqwest::Response> {
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

        request.send().await.map_err(|e| {
            Error::new(
                ErrorCode::NetworkError,
                format!("DELETE request failed: {e}"),
            )
        })
    }

    /// Map Mattermost error ID to appropriate ErrorCode
    ///
    /// # Arguments
    /// * `error_id` - The Mattermost error ID (e.g., "api.user.login.invalid_credentials")
    ///
    /// # Returns
    /// The appropriate ErrorCode for this error ID
    fn map_mattermost_error_id(error_id: &str) -> ErrorCode {
        // Based on common Mattermost error ID patterns
        // Check MFA-specific errors first (before general login errors)
        if error_id.contains("mfa_required") {
            ErrorCode::AuthenticationFailed
        } else if error_id.contains("invalid_mfa") || error_id.contains("mfa") {
            ErrorCode::AuthenticationFailed
        } else if error_id.contains("invalid_credentials") || error_id.contains("login") {
            ErrorCode::AuthenticationFailed
        } else if error_id.contains("not_found") {
            ErrorCode::NotFound
        } else if error_id.contains("permission") || error_id.contains("forbidden") {
            ErrorCode::PermissionDenied
        } else if error_id.contains("rate_limit") {
            ErrorCode::RateLimited
        } else if error_id.contains("timeout") {
            ErrorCode::Timeout
        } else if error_id.contains("invalid_param") || error_id.contains("invalid_") {
            ErrorCode::InvalidArgument
        } else {
            ErrorCode::Unknown
        }
    }

    /// Check if the response is successful and extract the JSON body
    ///
    /// # Arguments
    /// * `response` - The HTTP response from the API
    ///
    /// # Returns
    /// A Result containing the deserialized response body or an Error
    pub async fn handle_response<T: serde::de::DeserializeOwned>(
        &self,
        response: reqwest::Response,
    ) -> Result<T> {
        let status = response.status();

        // Extract request ID from headers for debugging
        let request_id = response
            .headers()
            .get("X-Request-Id")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        // Extract and store rate limit info from headers
        self.update_rate_limit_info(&response).await;

        if status.is_success() {
            // Success case - parse response body
            response.json::<T>().await.map_err(|e| {
                Error::new(ErrorCode::Unknown, format!("Failed to parse response: {e}"))
            })
        } else {
            // Error case - try to parse as Mattermost error response
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            // Try to parse as structured Mattermost error
            if let Ok(mm_error) =
                serde_json::from_str::<super::types::MattermostErrorResponse>(&error_text)
            {
                // Successfully parsed Mattermost error response
                let error_code = Self::map_mattermost_error_id(&mm_error.id);
                let mut error = Error::new(error_code, mm_error.message)
                    .with_mattermost_error_id(mm_error.id)
                    .with_http_status(status.as_u16());

                if let Some(req_id) = request_id {
                    error = error.with_request_id(req_id);
                }

                Err(error)
            } else {
                // Fallback for non-structured errors - infer error code from HTTP status
                let error_code = match status.as_u16() {
                    401 | 403 => ErrorCode::AuthenticationFailed,
                    404 => ErrorCode::NotFound,
                    429 => ErrorCode::RateLimited,
                    500..=599 => ErrorCode::NetworkError,
                    _ => ErrorCode::Unknown,
                };

                let mut error = Error::new(
                    error_code,
                    format!("API request failed with status {status}: {error_text}"),
                )
                .with_http_status(status.as_u16());

                if let Some(req_id) = request_id {
                    error = error.with_request_id(req_id);
                }

                Err(error)
            }
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
    pub async fn get_emojis(
        &self,
        page: u32,
        per_page: u32,
        sort: &str,
    ) -> Result<Vec<super::types::MattermostEmoji>> {
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
    pub async fn get_emoji_by_name(
        &self,
        emoji_name: &str,
    ) -> Result<super::types::MattermostEmoji> {
        let endpoint = format!("/emoji/name/{}", emoji_name);
        let response = self.get(&endpoint).await?;
        self.handle_response(response).await
    }

    // ========================================================================
    // Cached API Methods
    // ========================================================================

    /// Get a user by ID with caching
    ///
    /// Checks the cache first. If not found or expired, fetches from the API
    /// and stores in cache before returning.
    ///
    /// # Arguments
    /// * `user_id` - The ID of the user to retrieve
    ///
    /// # Returns
    /// A Result containing the user information or an Error
    pub async fn get_user_cached(&self, user_id: &str) -> Result<MattermostUser> {
        // Return early if caching is disabled
        if !self.cache_config.enable_cache {
            return self.get_user(user_id).await;
        }

        // Check cache first
        if let Some(user) = self.user_cache.get(user_id).await {
            return Ok(user);
        }

        // Cache miss - fetch from API
        let user = self.get_user(user_id).await?;

        // Store in cache before returning
        self.user_cache.set(user_id.to_string(), user.clone()).await;

        Ok(user)
    }

    /// Get a channel by ID with caching
    ///
    /// Checks the cache first. If not found or expired, fetches from the API
    /// and stores in cache before returning.
    ///
    /// # Arguments
    /// * `channel_id` - The ID of the channel to retrieve
    ///
    /// # Returns
    /// A Result containing the channel information or an Error
    pub async fn get_channel_cached(&self, channel_id: &str) -> Result<MattermostChannel> {
        // Return early if caching is disabled
        if !self.cache_config.enable_cache {
            return self.get_channel(channel_id).await;
        }

        // Check cache first
        if let Some(channel) = self.channel_cache.get(channel_id).await {
            return Ok(channel);
        }

        // Cache miss - fetch from API
        let channel = self.get_channel(channel_id).await?;

        // Store in cache before returning
        self.channel_cache
            .set(channel_id.to_string(), channel.clone())
            .await;

        Ok(channel)
    }

    /// Get a team by ID with caching
    ///
    /// Checks the cache first. If not found or expired, fetches from the API
    /// and stores in cache before returning.
    ///
    /// # Arguments
    /// * `team_id` - The ID of the team to retrieve
    ///
    /// # Returns
    /// A Result containing the team information or an Error
    pub async fn get_team_cached(&self, team_id: &str) -> Result<MattermostTeam> {
        // Return early if caching is disabled
        if !self.cache_config.enable_cache {
            return self.get_team(team_id).await;
        }

        // Check cache first
        if let Some(team) = self.team_cache.get(team_id).await {
            return Ok(team);
        }

        // Cache miss - fetch from API
        let team = self.get_team(team_id).await?;

        // Store in cache before returning
        self.team_cache.set(team_id.to_string(), team.clone()).await;

        Ok(team)
    }

    /// Get multiple users by their IDs with caching
    ///
    /// This method intelligently uses the cache to minimize API calls:
    /// 1. Checks cache for all requested users
    /// 2. Only fetches uncached users from the API
    /// 3. Caches newly fetched users
    /// 4. Returns all users in the order requested
    ///
    /// # Arguments
    /// * `user_ids` - A list of user IDs to retrieve
    ///
    /// # Returns
    /// A Result containing a list of users or an Error
    ///
    /// # Performance
    /// If all users are cached, this makes zero API calls.
    /// Otherwise, it makes one batch API call for all uncached users.
    pub async fn get_users_by_ids_cached(
        &self,
        user_ids: &[String],
    ) -> Result<Vec<MattermostUser>> {
        // Return early if caching is disabled
        if !self.cache_config.enable_cache {
            return self.get_users_by_ids(user_ids).await;
        }

        let mut result = Vec::with_capacity(user_ids.len());
        let mut uncached_ids = Vec::new();

        // Check cache for each user
        for user_id in user_ids {
            if let Some(user) = self.user_cache.get(user_id).await {
                result.push((user_id.clone(), user));
            } else {
                uncached_ids.push(user_id.clone());
            }
        }

        // If there are uncached users, fetch them from API
        if !uncached_ids.is_empty() {
            let fetched_users = self.get_users_by_ids(&uncached_ids).await?;

            // Cache the newly fetched users and add to result
            for user in fetched_users {
                self.user_cache.set(user.id.clone(), user.clone()).await;
                result.push((user.id.clone(), user));
            }
        }

        // Sort result to match the order of input user_ids
        let user_map: std::collections::HashMap<String, MattermostUser> =
            result.into_iter().collect();

        let ordered_result: Vec<MattermostUser> = user_ids
            .iter()
            .filter_map(|id| user_map.get(id).cloned())
            .collect();

        Ok(ordered_result)
    }

    /// Invalidate a user in the cache
    ///
    /// This is typically called when a WebSocket event indicates
    /// that the user has been updated.
    ///
    /// # Arguments
    /// * `user_id` - The ID of the user to invalidate
    pub async fn invalidate_user_cache(&self, user_id: &str) {
        self.user_cache.invalidate(user_id).await;
    }

    /// Invalidate a channel in the cache
    ///
    /// This is typically called when a WebSocket event indicates
    /// that the channel has been updated or deleted.
    ///
    /// # Arguments
    /// * `channel_id` - The ID of the channel to invalidate
    pub async fn invalidate_channel_cache(&self, channel_id: &str) {
        self.channel_cache.invalidate(channel_id).await;
    }

    /// Invalidate a team in the cache
    ///
    /// This is typically called when a WebSocket event indicates
    /// that the team has been updated.
    ///
    /// # Arguments
    /// * `team_id` - The ID of the team to invalidate
    pub async fn invalidate_team_cache(&self, team_id: &str) {
        self.team_cache.invalidate(team_id).await;
    }

    /// Update a channel in the cache
    ///
    /// This is typically called after creating or updating a channel
    /// to ensure the cache reflects the latest state.
    ///
    /// # Arguments
    /// * `channel` - The channel to cache
    pub async fn update_channel_cache(&self, channel: &MattermostChannel) {
        self.channel_cache
            .set(channel.id.clone(), channel.clone())
            .await;
    }

    /// Remove a channel from the cache
    ///
    /// This is typically called after deleting/archiving a channel.
    ///
    /// # Arguments
    /// * `channel_id` - The ID of the channel to remove from cache
    pub async fn remove_channel_from_cache(&self, channel_id: &str) {
        self.channel_cache.invalidate(channel_id).await;
    }

    /// Clear all caches
    ///
    /// This is useful when major changes occur (e.g., user logout/login,
    /// team changes) that may affect many cached entries.
    pub async fn clear_all_caches(&self) {
        self.user_cache.clear().await;
        self.channel_cache.clear().await;
        self.team_cache.clear().await;
    }

    /// Get cache statistics
    ///
    /// Returns statistics for all caches: (cache_name, total_entries, expired_entries)
    ///
    /// # Returns
    /// A vector of tuples containing cache statistics
    pub async fn get_cache_stats(&self) -> Vec<(&'static str, usize, usize)> {
        vec![
            (
                "user",
                self.user_cache.stats().await.0,
                self.user_cache.stats().await.1,
            ),
            (
                "channel",
                self.channel_cache.stats().await.0,
                self.channel_cache.stats().await.1,
            ),
            (
                "team",
                self.team_cache.stats().await.0,
                self.team_cache.stats().await.1,
            ),
        ]
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

    #[test]
    fn test_mattermost_error_id_mapping() {
        // Test authentication errors
        assert_eq!(
            MattermostClient::map_mattermost_error_id("api.user.login.invalid_credentials"),
            ErrorCode::AuthenticationFailed
        );
        assert_eq!(
            MattermostClient::map_mattermost_error_id("api.user.login.failed"),
            ErrorCode::AuthenticationFailed
        );

        // Test not found errors
        assert_eq!(
            MattermostClient::map_mattermost_error_id("api.user.get.not_found"),
            ErrorCode::NotFound
        );
        assert_eq!(
            MattermostClient::map_mattermost_error_id("api.channel.not_found"),
            ErrorCode::NotFound
        );

        // Test permission errors
        assert_eq!(
            MattermostClient::map_mattermost_error_id("api.context.permissions_error"),
            ErrorCode::PermissionDenied
        );
        assert_eq!(
            MattermostClient::map_mattermost_error_id("api.team.forbidden"),
            ErrorCode::PermissionDenied
        );

        // Test rate limit errors
        assert_eq!(
            MattermostClient::map_mattermost_error_id("api.rate_limit.exceeded"),
            ErrorCode::RateLimited
        );

        // Test timeout errors
        assert_eq!(
            MattermostClient::map_mattermost_error_id("api.request.timeout"),
            ErrorCode::Timeout
        );

        // Test invalid argument errors
        assert_eq!(
            MattermostClient::map_mattermost_error_id("api.post.invalid_param.message"),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            MattermostClient::map_mattermost_error_id("api.channel.invalid_id"),
            ErrorCode::InvalidArgument
        );

        // Test unknown errors
        assert_eq!(
            MattermostClient::map_mattermost_error_id("api.unknown.error"),
            ErrorCode::Unknown
        );
    }
}
