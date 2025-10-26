use crate::error::Result;
use serde::{Deserialize, Serialize};

use super::client::MattermostClient;
use super::types::{MattermostChannel, MattermostUser, PostList};

// ============================================================================
// Search Request/Response Types
// ============================================================================

/// Request body for user search
#[derive(Debug, Clone, Serialize)]
pub struct UserSearchRequest {
    /// Search term to match against username, first name, last name, nickname, or email
    pub term: String,
    /// Limit the search to a team
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_id: Option<String>,
    /// Limit the search to users not in a channel
    #[serde(skip_serializing_if = "Option::is_none")]
    pub not_in_channel_id: Option<String>,
    /// Limit the search to users in a channel
    #[serde(skip_serializing_if = "Option::is_none")]
    pub in_channel_id: Option<String>,
    /// Allow inactive users to be returned
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_inactive: Option<bool>,
    /// Limit the search to users without a team
    #[serde(skip_serializing_if = "Option::is_none")]
    pub without_team: Option<bool>,
    /// Maximum number of users to return (default: 100)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
}

impl UserSearchRequest {
    /// Create a new user search request
    pub fn new(term: String) -> Self {
        Self {
            term,
            team_id: None,
            not_in_channel_id: None,
            in_channel_id: None,
            allow_inactive: None,
            without_team: None,
            limit: None,
        }
    }

    /// Set the team ID filter
    pub fn with_team_id(mut self, team_id: String) -> Self {
        self.team_id = Some(team_id);
        self
    }

    /// Filter to users not in a specific channel
    pub fn not_in_channel(mut self, channel_id: String) -> Self {
        self.not_in_channel_id = Some(channel_id);
        self
    }

    /// Filter to users in a specific channel
    pub fn in_channel(mut self, channel_id: String) -> Self {
        self.in_channel_id = Some(channel_id);
        self
    }

    /// Include inactive users in results
    pub fn allow_inactive(mut self, allow: bool) -> Self {
        self.allow_inactive = Some(allow);
        self
    }

    /// Filter to users without a team
    pub fn without_team(mut self, without: bool) -> Self {
        self.without_team = Some(without);
        self
    }

    /// Set the maximum number of results
    pub fn with_limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }
}

/// Request body for channel search
#[derive(Debug, Clone, Serialize)]
pub struct ChannelSearchRequest {
    /// Search term to match against channel name or display name
    pub term: String,
}

impl ChannelSearchRequest {
    /// Create a new channel search request
    pub fn new(term: String) -> Self {
        Self { term }
    }
}

/// Request body for file search
#[derive(Debug, Clone, Serialize)]
pub struct FileSearchRequest {
    /// Search terms
    pub terms: String,
    /// Limit search to specific channel
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_id: Option<String>,
    /// Filter by file extensions (e.g., "pdf", "png")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ext: Option<Vec<String>>,
    /// Time zone offset in seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_zone_offset: Option<i32>,
}

impl FileSearchRequest {
    /// Create a new file search request
    pub fn new(terms: String) -> Self {
        Self {
            terms,
            channel_id: None,
            ext: None,
            time_zone_offset: None,
        }
    }

    /// Limit search to a specific channel
    pub fn in_channel(mut self, channel_id: String) -> Self {
        self.channel_id = Some(channel_id);
        self
    }

    /// Filter by file extensions
    pub fn with_extensions(mut self, extensions: Vec<String>) -> Self {
        self.ext = Some(extensions);
        self
    }

    /// Set timezone offset
    pub fn with_timezone_offset(mut self, offset: i32) -> Self {
        self.time_zone_offset = Some(offset);
        self
    }
}

/// File search result item
#[derive(Debug, Clone, Deserialize)]
pub struct FileSearchResult {
    pub id: String,
    pub user_id: String,
    pub post_id: String,
    pub channel_id: String,
    pub create_at: i64,
    pub update_at: i64,
    pub delete_at: i64,
    pub name: String,
    pub extension: String,
    pub size: i64,
    pub mime_type: String,
    #[serde(default)]
    pub width: i32,
    #[serde(default)]
    pub height: i32,
    #[serde(default)]
    pub has_preview_image: bool,
}

/// Response from file search
#[derive(Debug, Clone, Deserialize)]
pub struct FileSearchResponse {
    pub file_infos: Vec<FileSearchResult>,
    pub order: Vec<String>,
}

/// Advanced search options for posts
#[derive(Debug, Clone, Default)]
pub struct PostSearchOptions {
    /// Search is case-insensitive
    pub is_or_search: bool,
    /// Include deleted posts
    pub include_deleted_channels: bool,
    /// Time zone offset for date searches
    pub time_zone_offset: i32,
    /// Limit the number of results
    pub page: u32,
    pub per_page: u32,
}

// ============================================================================
// Search API Implementation
// ============================================================================

impl MattermostClient {
    /// Search for users
    ///
    /// # Arguments
    /// * `request` - User search request with filters
    ///
    /// # Returns
    /// A Result containing a vector of MattermostUser or an Error
    ///
    /// # Example
    /// ```no_run
    /// let request = UserSearchRequest::new("john".to_string())
    ///     .with_team_id(team_id)
    ///     .with_limit(50);
    /// let users = client.search_users(&request).await?;
    /// ```
    pub async fn search_users(&self, request: &UserSearchRequest) -> Result<Vec<MattermostUser>> {
        let response = self.post("/users/search", request).await?;
        self.handle_response(response).await
    }

    /// Autocomplete users for mentions
    ///
    /// # Arguments
    /// * `team_id` - Team ID to search within
    /// * `channel_id` - Channel ID to search within
    /// * `name` - Username prefix to autocomplete
    /// * `limit` - Maximum number of results (optional)
    ///
    /// # Returns
    /// A Result containing a vector of MattermostUser or an Error
    pub async fn autocomplete_users(
        &self,
        team_id: &str,
        channel_id: &str,
        name: &str,
        limit: Option<u32>,
    ) -> Result<Vec<MattermostUser>> {
        let mut endpoint = format!(
            "/users/autocomplete?in_team={}&in_channel={}&name={}",
            team_id, channel_id, name
        );

        if let Some(limit) = limit {
            endpoint.push_str(&format!("&limit={}", limit));
        }

        let response = self.get(&endpoint).await?;

        // The autocomplete endpoint returns a special structure with "users" and "out_of_channel" arrays
        #[derive(Deserialize)]
        struct AutocompleteResponse {
            users: Vec<MattermostUser>,
            #[serde(default)]
            out_of_channel: Vec<MattermostUser>,
        }

        let autocomplete: AutocompleteResponse = self.handle_response(response).await?;

        // Combine both arrays
        let mut all_users = autocomplete.users;
        all_users.extend(autocomplete.out_of_channel);

        Ok(all_users)
    }

    /// Search for channels in a team
    ///
    /// # Arguments
    /// * `team_id` - Team ID to search within
    /// * `request` - Channel search request
    ///
    /// # Returns
    /// A Result containing a vector of MattermostChannel or an Error
    ///
    /// # Example
    /// ```no_run
    /// let request = ChannelSearchRequest::new("general".to_string());
    /// let channels = client.search_channels(team_id, &request).await?;
    /// ```
    pub async fn search_channels(
        &self,
        team_id: &str,
        request: &ChannelSearchRequest,
    ) -> Result<Vec<MattermostChannel>> {
        let endpoint = format!("/teams/{}/channels/search", team_id);
        let response = self.post(&endpoint, request).await?;
        self.handle_response(response).await
    }

    /// Autocomplete channels for references
    ///
    /// # Arguments
    /// * `team_id` - Team ID to search within
    /// * `name` - Channel name prefix to autocomplete
    ///
    /// # Returns
    /// A Result containing a vector of MattermostChannel or an Error
    pub async fn autocomplete_channels(
        &self,
        team_id: &str,
        name: &str,
    ) -> Result<Vec<MattermostChannel>> {
        let endpoint = format!("/teams/{}/channels/autocomplete?name={}", team_id, name);
        let response = self.get(&endpoint).await?;
        self.handle_response(response).await
    }

    /// Search for files
    ///
    /// # Arguments
    /// * `team_id` - Team ID to search within
    /// * `request` - File search request
    ///
    /// # Returns
    /// A Result containing FileSearchResponse or an Error
    ///
    /// # Example
    /// ```no_run
    /// let request = FileSearchRequest::new("report".to_string())
    ///     .with_extensions(vec!["pdf".to_string(), "doc".to_string()]);
    /// let results = client.search_files(team_id, &request).await?;
    /// ```
    pub async fn search_files(
        &self,
        team_id: &str,
        request: &FileSearchRequest,
    ) -> Result<FileSearchResponse> {
        let endpoint = format!("/teams/{}/files/search", team_id);
        let response = self.post(&endpoint, request).await?;
        self.handle_response(response).await
    }

    /// Advanced post search with support for search operators
    ///
    /// # Arguments
    /// * `team_id` - Team ID to search within
    /// * `terms` - Search terms (supports operators like from:, in:, before:, after:, on:)
    /// * `options` - Additional search options
    ///
    /// # Returns
    /// A Result containing a PostList or an Error
    ///
    /// # Search Operators
    /// - `from:username` - Posts from a specific user
    /// - `in:channel-name` - Posts in a specific channel
    /// - `before:YYYY-MM-DD` - Posts before a date
    /// - `after:YYYY-MM-DD` - Posts after a date
    /// - `on:YYYY-MM-DD` - Posts on a specific date
    /// - `"exact phrase"` - Phrase search with quotes
    /// - `-word` - Exclude posts with this word
    ///
    /// # Example
    /// ```no_run
    /// // Search for posts from john in town-square containing "project"
    /// let terms = "from:john in:town-square project";
    /// let results = client.search_posts_advanced(team_id, terms, PostSearchOptions::default()).await?;
    /// ```
    pub async fn search_posts_advanced(
        &self,
        team_id: &str,
        terms: &str,
        options: PostSearchOptions,
    ) -> Result<PostList> {
        let body = serde_json::json!({
            "terms": terms,
            "is_or_search": options.is_or_search,
            "include_deleted_channels": options.include_deleted_channels,
            "time_zone_offset": options.time_zone_offset,
            "page": options.page,
            "per_page": options.per_page,
        });

        let endpoint = format!("/teams/{}/posts/search", team_id);
        let response = self.post(&endpoint, &body).await?;
        self.handle_response(response).await
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_search_request_builder() {
        let request = UserSearchRequest::new("john".to_string())
            .with_team_id("team123".to_string())
            .in_channel("channel456".to_string())
            .with_limit(50);

        assert_eq!(request.term, "john");
        assert_eq!(request.team_id, Some("team123".to_string()));
        assert_eq!(request.in_channel_id, Some("channel456".to_string()));
        assert_eq!(request.limit, Some(50));
    }

    #[test]
    fn test_channel_search_request() {
        let request = ChannelSearchRequest::new("general".to_string());
        assert_eq!(request.term, "general");
    }

    #[test]
    fn test_file_search_request_builder() {
        let request = FileSearchRequest::new("report".to_string())
            .in_channel("channel123".to_string())
            .with_extensions(vec!["pdf".to_string(), "doc".to_string()])
            .with_timezone_offset(3600);

        assert_eq!(request.terms, "report");
        assert_eq!(request.channel_id, Some("channel123".to_string()));
        assert_eq!(request.ext, Some(vec!["pdf".to_string(), "doc".to_string()]));
        assert_eq!(request.time_zone_offset, Some(3600));
    }

    #[test]
    fn test_post_search_options_default() {
        let options = PostSearchOptions::default();
        assert_eq!(options.is_or_search, false);
        assert_eq!(options.include_deleted_channels, false);
        assert_eq!(options.time_zone_offset, 0);
        assert_eq!(options.page, 0);
        assert_eq!(options.per_page, 0);
    }
}
