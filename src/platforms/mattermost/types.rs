use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Mattermost channel type
/// Based on the Mattermost API specification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MattermostChannelType {
    /// Open/Public channel - "O"
    #[serde(rename = "O")]
    Open,
    /// Private channel - "P"
    #[serde(rename = "P")]
    Private,
    /// Direct message channel - "D"
    #[serde(rename = "D")]
    Direct,
    /// Group message channel - "G"
    #[serde(rename = "G")]
    Group,
}

impl MattermostChannelType {
    /// Get the string representation of the channel type
    pub fn as_str(&self) -> &'static str {
        match self {
            MattermostChannelType::Open => "O",
            MattermostChannelType::Private => "P",
            MattermostChannelType::Direct => "D",
            MattermostChannelType::Group => "G",
        }
    }

    /// Check if this is a direct message (1-on-1)
    pub fn is_direct(&self) -> bool {
        matches!(self, MattermostChannelType::Direct)
    }

    /// Check if this is a group message
    pub fn is_group(&self) -> bool {
        matches!(self, MattermostChannelType::Group)
    }

    /// Check if this is a public channel
    pub fn is_public(&self) -> bool {
        matches!(self, MattermostChannelType::Open)
    }

    /// Check if this is a private channel
    pub fn is_private(&self) -> bool {
        matches!(self, MattermostChannelType::Private)
    }
}

/// Mattermost User object from API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MattermostUser {
    pub id: String,
    pub username: String,
    #[serde(default)]
    pub email: String,
    #[serde(default)]
    pub first_name: String,
    #[serde(default)]
    pub last_name: String,
    #[serde(default)]
    pub nickname: String,
    #[serde(default)]
    pub position: String,
    #[serde(default)]
    pub roles: String,
    #[serde(default)]
    pub locale: String,
    #[serde(default)]
    pub timezone: HashMap<String, String>,
    #[serde(default)]
    pub props: HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub is_bot: bool,
    pub create_at: i64,
    pub update_at: i64,
    pub delete_at: i64,
}

/// Mattermost Channel object from API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MattermostChannel {
    pub id: String,
    pub create_at: i64,
    pub update_at: i64,
    pub delete_at: i64,
    pub team_id: String,
    #[serde(rename = "type")]
    pub channel_type: MattermostChannelType,
    pub display_name: String,
    pub name: String,
    #[serde(default)]
    pub header: String,
    #[serde(default)]
    pub purpose: String,
    #[serde(default)]
    pub last_post_at: i64,
    #[serde(default)]
    pub total_msg_count: i64,
    #[serde(default)]
    pub creator_id: String,
}

/// Mattermost Post (message) object from API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MattermostPost {
    pub id: String,
    pub create_at: i64,
    pub update_at: i64,
    pub delete_at: i64,
    pub edit_at: i64,
    pub user_id: String,
    pub channel_id: String,
    #[serde(default)]
    pub root_id: String,
    #[serde(default)]
    pub parent_id: String,
    #[serde(default)]
    pub original_id: String,
    pub message: String,
    #[serde(rename = "type")]
    #[serde(default)]
    pub post_type: String,
    #[serde(default)]
    pub props: HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub hashtags: String,
    #[serde(default)]
    pub file_ids: Vec<String>,
    #[serde(default)]
    pub pending_post_id: String,
    #[serde(default)]
    pub metadata: PostMetadata,
}

/// Metadata for a Mattermost Post
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PostMetadata {
    #[serde(default)]
    pub embeds: Vec<serde_json::Value>,
    #[serde(default)]
    pub emojis: Vec<serde_json::Value>,
    #[serde(default)]
    pub files: Vec<FileInfo>,
    #[serde(default)]
    pub images: HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub reactions: Vec<serde_json::Value>,
}

/// Mattermost File information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub id: String,
    pub user_id: String,
    pub post_id: String,
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

/// Mattermost Reaction object from API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reaction {
    pub user_id: String,
    pub post_id: String,
    pub emoji_name: String,
    pub create_at: i64,
}

/// Request to save a reaction
#[derive(Debug, Clone, Serialize)]
pub struct SaveReactionRequest {
    pub user_id: String,
    pub post_id: String,
    pub emoji_name: String,
}

/// Mattermost Team (workspace) object from API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MattermostTeam {
    pub id: String,
    pub create_at: i64,
    pub update_at: i64,
    pub delete_at: i64,
    pub display_name: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub email: String,
    #[serde(rename = "type")]
    pub team_type: String, // "O" (Open), "I" (Invite only)
    #[serde(default)]
    pub company_name: String,
    #[serde(default)]
    pub allowed_domains: String,
    #[serde(default)]
    pub invite_id: String,
    #[serde(default)]
    pub allow_open_invite: bool,
}

/// Login request payload
#[derive(Debug, Clone, Serialize)]
pub struct LoginRequest {
    pub login_id: String,
    pub password: String,
    /// MFA token (6-digit code from authenticator app)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    /// Device ID for tracking login devices
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_id: Option<String>,
}

/// Channel creation request for direct messages
#[derive(Debug, Clone, Serialize)]
pub struct CreateDirectChannelRequest {
    pub user_ids: Vec<String>,
}

/// Channel creation request for group messages
#[derive(Debug, Clone, Serialize)]
pub struct CreateGroupChannelRequest {
    pub user_ids: Vec<String>,
}

/// Post creation request
#[derive(Debug, Clone, Serialize)]
pub struct CreatePostRequest {
    pub channel_id: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub root_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub props: Option<HashMap<String, serde_json::Value>>,
}

/// Response containing a list of posts
#[derive(Debug, Clone, Deserialize)]
pub struct PostList {
    pub order: Vec<String>,
    pub posts: HashMap<String, MattermostPost>,
    #[serde(default)]
    pub next_post_id: String,
    #[serde(default)]
    pub prev_post_id: String,
}

/// Channel member object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelMember {
    pub channel_id: String,
    pub user_id: String,
    pub roles: String,
    pub last_viewed_at: i64,
    pub msg_count: i64,
    pub mention_count: i64,
    pub notify_props: HashMap<String, String>,
    pub last_update_at: i64,
}

// ============================================================================
// Channel Read State Types
// ============================================================================

/// Request to mark a channel as viewed (read)
#[derive(Debug, Clone, Serialize)]
pub struct ChannelViewRequest {
    pub channel_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prev_channel_id: Option<String>,
}

impl ChannelViewRequest {
    /// Create a new channel view request
    pub fn new(channel_id: String) -> Self {
        Self {
            channel_id,
            prev_channel_id: None,
        }
    }

    /// Set the previous channel ID (optional, for tracking channel switches)
    pub fn with_prev_channel(mut self, prev_channel_id: String) -> Self {
        self.prev_channel_id = Some(prev_channel_id);
        self
    }
}

/// Unread information for a single channel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelUnreadInfo {
    /// Team ID the channel belongs to
    pub team_id: String,
    /// Channel ID
    pub channel_id: String,
    /// Number of unread messages
    pub msg_count: i64,
    /// Number of unread mentions
    pub mention_count: i64,
    /// Timestamp when the channel was last viewed
    pub last_viewed_at: i64,
}

/// Unread counts for a team
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamUnread {
    /// Team ID
    pub team_id: String,
    /// Total unread message count across all channels in team
    pub msg_count: i64,
    /// Total unread mention count across all channels in team
    pub mention_count: i64,
}

/// Response from viewing a channel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelViewResponse {
    pub status: String,
    #[serde(default)]
    pub last_viewed_at_times: HashMap<String, i64>,
}

/// WebSocket event from Mattermost
#[derive(Debug, Clone, Deserialize)]
pub struct WebSocketEvent {
    #[serde(default)]
    pub event: String,
    #[serde(default)]
    pub data: HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub broadcast: WebSocketBroadcast,
    #[serde(default)]
    pub seq: i64,
}

/// WebSocket broadcast information
#[derive(Debug, Clone, Default, Deserialize)]
pub struct WebSocketBroadcast {
    #[serde(default)]
    pub omit_users: Option<HashMap<String, bool>>,
    #[serde(default)]
    pub user_id: String,
    #[serde(default)]
    pub channel_id: String,
    #[serde(default)]
    pub team_id: String,
    #[serde(default)]
    pub connection_id: String,
    #[serde(default)]
    pub omit_connection_id: String,
}

/// WebSocket authentication challenge
#[derive(Debug, Clone, Serialize)]
pub struct WebSocketAuthChallenge {
    pub seq: i64,
    pub action: String,
    pub data: WebSocketAuthData,
}

#[derive(Debug, Clone, Serialize)]
pub struct WebSocketAuthData {
    pub token: String,
}

/// WebSocket authentication response
#[derive(Debug, Clone, Deserialize)]
pub struct WebSocketAuthResponse {
    pub status: String,
    pub seq_reply: i64,
}

/// Status object for user presence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Status {
    pub user_id: String,
    pub status: String, // "online", "away", "dnd", "offline"
    pub manual: bool,
    pub last_activity_at: i64,
}

impl CreatePostRequest {
    /// Create a simple post request with just a message
    pub fn new(channel_id: String, message: String) -> Self {
        Self {
            channel_id,
            message,
            root_id: None,
            file_ids: None,
            props: None,
        }
    }

    /// Add a root_id to make this a reply to another post
    pub fn with_root_id(mut self, root_id: String) -> Self {
        self.root_id = Some(root_id);
        self
    }

    /// Add file attachments
    pub fn with_files(mut self, file_ids: Vec<String>) -> Self {
        self.file_ids = Some(file_ids);
        self
    }

    /// Add custom properties
    pub fn with_props(mut self, props: HashMap<String, serde_json::Value>) -> Self {
        self.props = Some(props);
        self
    }
}

/// User status response from Mattermost API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MattermostStatus {
    pub user_id: String,
    pub status: String, // "online", "away", "dnd", "offline"
    #[serde(default)]
    pub manual: bool,
    #[serde(default)]
    pub last_activity_at: i64,
}

/// Custom status for a user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomStatus {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub emoji: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<String>, // Duration like "thirty_minutes", "one_hour", "today", "this_week"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>, // ISO 8601 timestamp
}

/// Request to set user status
#[derive(Debug, Clone, Serialize)]
pub struct SetStatusRequest {
    pub user_id: String,
    pub status: String, // "online", "away", "dnd", "offline"
}

/// Request to get statuses for multiple users
#[derive(Debug, Clone, Serialize)]
pub struct GetStatusesByIdsRequest {
    pub user_ids: Vec<String>,
}

/// Mattermost custom emoji object from API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MattermostEmoji {
    /// The ID of the emoji
    pub id: String,
    /// The ID of the user that created the emoji
    pub creator_id: String,
    /// The name of the emoji
    pub name: String,
    /// The time in milliseconds the emoji was created
    pub create_at: i64,
    /// The time in milliseconds the emoji was last updated
    pub update_at: i64,
    /// The time in milliseconds the emoji was deleted (0 if not deleted)
    pub delete_at: i64,
}

/// Mattermost error response structure
/// Based on the Mattermost API specification (lines 141-155)
#[derive(Debug, Clone, Deserialize)]
pub struct MattermostErrorResponse {
    /// Error identifier (e.g., "api.user.login.invalid_credentials")
    pub id: String,
    /// Human-readable error message
    pub message: String,
    /// Request ID for debugging with server logs
    #[serde(default)]
    pub request_id: String,
    /// HTTP status code
    pub status_code: i32,
    /// OAuth-specific error flag
    #[serde(default)]
    pub is_oauth: bool,
}

/// Mattermost Thread object representing a followed thread
/// Based on API spec: UserThread schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserThread {
    /// ID of the post that is this thread's root
    pub id: String,
    /// Number of replies in this thread
    pub reply_count: i64,
    /// Timestamp of the last post to this thread
    pub last_reply_at: i64,
    /// Timestamp of the last time the user viewed this thread
    pub last_viewed_at: i64,
    /// List of users participating in this thread (user IDs or full user objects if extended=true)
    #[serde(default)]
    pub participants: Vec<serde_json::Value>,
    /// The root post of the thread
    pub post: MattermostPost,
    /// Number of unread replies
    #[serde(default)]
    pub unread_replies: i64,
    /// Number of unread mentions
    #[serde(default)]
    pub unread_mentions: i64,
}

/// Response wrapper for a list of threads
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserThreads {
    /// Total count of threads
    #[serde(default)]
    pub total: i64,
    /// Total count of unread threads
    #[serde(default)]
    pub total_unread_threads: i64,
    /// Total count of unread mentions across all threads
    #[serde(default)]
    pub total_unread_mentions: i64,
    /// List of threads
    #[serde(default)]
    pub threads: Vec<UserThread>,
}

/// Response for thread read/follow operations
/// Most operations return 200 OK with no body, but we need this for consistent handling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadResponse {
    #[serde(default)]
    pub status: String,
}

impl From<MattermostEmoji> for crate::types::Emoji {
    fn from(mm_emoji: MattermostEmoji) -> Self {
        crate::types::Emoji {
            id: mm_emoji.id,
            name: mm_emoji.name,
            creator_id: mm_emoji.creator_id,
            created_at: mm_emoji.create_at,
        }
    }
}

// ============================================================================
// User Preferences and Notifications
// ============================================================================

/// Notification level for channels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NotificationLevel {
    /// Notify for all messages
    All,
    /// Notify only for mentions
    Mention,
    /// No notifications
    None,
}

impl NotificationLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            NotificationLevel::All => "all",
            NotificationLevel::Mention => "mention",
            NotificationLevel::None => "none",
        }
    }
}

/// User preference object
/// Represents a single preference setting for a user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreference {
    /// User ID this preference belongs to
    pub user_id: String,
    /// Preference category (e.g., "display_settings", "notifications", "advanced_settings")
    pub category: String,
    /// Preference name within the category
    pub name: String,
    /// Preference value as a string
    pub value: String,
}

impl UserPreference {
    /// Create a new user preference
    pub fn new(user_id: String, category: String, name: String, value: String) -> Self {
        Self {
            user_id,
            category,
            name,
            value,
        }
    }
}

/// Channel notification properties
/// Controls notification behavior for a specific channel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelNotifyProps {
    /// Desktop notification level
    #[serde(skip_serializing_if = "Option::is_none")]
    pub desktop: Option<String>,
    /// Mobile push notification level
    #[serde(skip_serializing_if = "Option::is_none")]
    pub push: Option<String>,
    /// Email notification setting
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    /// Mark channel as unread
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mark_unread: Option<String>,
    /// Ignore channel mentions (mute)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ignore_channel_mentions: Option<String>,
}

impl ChannelNotifyProps {
    /// Create new channel notification properties with all values set to None
    pub fn new() -> Self {
        Self {
            desktop: None,
            push: None,
            email: None,
            mark_unread: None,
            ignore_channel_mentions: None,
        }
    }

    /// Create a muted channel configuration
    pub fn muted() -> Self {
        Self {
            desktop: Some("none".to_string()),
            push: Some("none".to_string()),
            email: Some("false".to_string()),
            mark_unread: Some("mention".to_string()),
            ignore_channel_mentions: Some("on".to_string()),
        }
    }

    /// Create an unmuted channel configuration (default notifications)
    pub fn unmuted() -> Self {
        Self {
            desktop: Some("default".to_string()),
            push: Some("default".to_string()),
            email: Some("default".to_string()),
            mark_unread: Some("all".to_string()),
            ignore_channel_mentions: Some("off".to_string()),
        }
    }

    /// Set desktop notification level
    pub fn with_desktop(mut self, level: NotificationLevel) -> Self {
        self.desktop = Some(level.as_str().to_string());
        self
    }

    /// Set push notification level
    pub fn with_push(mut self, level: NotificationLevel) -> Self {
        self.push = Some(level.as_str().to_string());
        self
    }

    /// Set email notification
    pub fn with_email(mut self, enabled: bool) -> Self {
        self.email = Some(if enabled { "true" } else { "false" }.to_string());
        self
    }

    /// Set mark unread behavior
    pub fn with_mark_unread(mut self, level: NotificationLevel) -> Self {
        self.mark_unread = Some(level.as_str().to_string());
        self
    }
}

impl Default for ChannelNotifyProps {
    fn default() -> Self {
        Self::new()
    }
}

/// Request to update channel member notify properties
#[derive(Debug, Clone, Serialize)]
pub struct UpdateChannelNotifyPropsRequest {
    #[serde(flatten)]
    pub notify_props: ChannelNotifyProps,
}

/// Request to delete user preferences
#[derive(Debug, Clone, Serialize)]
pub struct DeletePreferencesRequest {
    pub preferences: Vec<UserPreference>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_post_request() {
        let req = CreatePostRequest::new("channel123".to_string(), "Hello, world!".to_string());

        assert_eq!(req.channel_id, "channel123");
        assert_eq!(req.message, "Hello, world!");
        assert!(req.root_id.is_none());
        assert!(req.file_ids.is_none());
    }

    #[test]
    fn test_create_post_request_with_root_id() {
        let req = CreatePostRequest::new("channel123".to_string(), "Reply!".to_string())
            .with_root_id("post456".to_string());

        assert_eq!(req.root_id, Some("post456".to_string()));
    }

    #[test]
    fn test_login_request_serialization() {
        let login = LoginRequest {
            login_id: "user@example.com".to_string(),
            password: "password123".to_string(),
            token: None,
            device_id: None,
        };

        let json = serde_json::to_string(&login).unwrap();
        assert!(json.contains("login_id"));
        assert!(json.contains("password"));
    }

    #[test]
    fn test_channel_type_enum() {
        // Test string representation
        assert_eq!(MattermostChannelType::Open.as_str(), "O");
        assert_eq!(MattermostChannelType::Private.as_str(), "P");
        assert_eq!(MattermostChannelType::Direct.as_str(), "D");
        assert_eq!(MattermostChannelType::Group.as_str(), "G");

        // Test helper methods
        assert!(MattermostChannelType::Direct.is_direct());
        assert!(!MattermostChannelType::Open.is_direct());

        assert!(MattermostChannelType::Group.is_group());
        assert!(!MattermostChannelType::Private.is_group());

        assert!(MattermostChannelType::Open.is_public());
        assert!(!MattermostChannelType::Direct.is_public());

        assert!(MattermostChannelType::Private.is_private());
        assert!(!MattermostChannelType::Group.is_private());
    }

    #[test]
    fn test_channel_type_serialization() {
        // Test that the enum serializes to the correct string values
        let open = MattermostChannelType::Open;
        let json = serde_json::to_string(&open).unwrap();
        assert_eq!(json, "\"O\"");

        let private = MattermostChannelType::Private;
        let json = serde_json::to_string(&private).unwrap();
        assert_eq!(json, "\"P\"");

        let direct = MattermostChannelType::Direct;
        let json = serde_json::to_string(&direct).unwrap();
        assert_eq!(json, "\"D\"");

        let group = MattermostChannelType::Group;
        let json = serde_json::to_string(&group).unwrap();
        assert_eq!(json, "\"G\"");
    }

    #[test]
    fn test_channel_type_deserialization() {
        // Test that we can deserialize from JSON strings
        let open: MattermostChannelType = serde_json::from_str("\"O\"").unwrap();
        assert!(open.is_public());

        let private: MattermostChannelType = serde_json::from_str("\"P\"").unwrap();
        assert!(private.is_private());

        let direct: MattermostChannelType = serde_json::from_str("\"D\"").unwrap();
        assert!(direct.is_direct());

        let group: MattermostChannelType = serde_json::from_str("\"G\"").unwrap();
        assert!(group.is_group());
    }

    #[test]
    fn test_mattermost_error_response_deserialization() {
        let json = r#"{
            "id": "api.user.login.invalid_credentials",
            "message": "Invalid login credentials",
            "request_id": "abc123",
            "status_code": 401,
            "is_oauth": false
        }"#;

        let error: MattermostErrorResponse = serde_json::from_str(json).unwrap();
        assert_eq!(error.id, "api.user.login.invalid_credentials");
        assert_eq!(error.message, "Invalid login credentials");
        assert_eq!(error.request_id, "abc123");
        assert_eq!(error.status_code, 401);
        assert_eq!(error.is_oauth, false);
    }

    #[test]
    fn test_mattermost_error_response_with_defaults() {
        // Test deserialization when optional fields are missing
        let json = r#"{
            "id": "api.post.create.error",
            "message": "Failed to create post",
            "status_code": 500
        }"#;

        let error: MattermostErrorResponse = serde_json::from_str(json).unwrap();
        assert_eq!(error.id, "api.post.create.error");
        assert_eq!(error.message, "Failed to create post");
        assert_eq!(error.request_id, ""); // default value
        assert_eq!(error.status_code, 500);
        assert_eq!(error.is_oauth, false); // default value
    }
}
