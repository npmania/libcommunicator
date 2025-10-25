use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    pub channel_type: String, // "O" (Open/Public), "P" (Private), "D" (Direct), "G" (Group)
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
    pub omit_users: HashMap<String, bool>,
    #[serde(default)]
    pub user_id: String,
    #[serde(default)]
    pub channel_id: String,
    #[serde(default)]
    pub team_id: String,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_post_request() {
        let req = CreatePostRequest::new(
            "channel123".to_string(),
            "Hello, world!".to_string(),
        );

        assert_eq!(req.channel_id, "channel123");
        assert_eq!(req.message, "Hello, world!");
        assert!(req.root_id.is_none());
        assert!(req.file_ids.is_none());
    }

    #[test]
    fn test_create_post_request_with_root_id() {
        let req = CreatePostRequest::new(
            "channel123".to_string(),
            "Reply!".to_string(),
        )
        .with_root_id("post456".to_string());

        assert_eq!(req.root_id, Some("post456".to_string()));
    }

    #[test]
    fn test_login_request_serialization() {
        let login = LoginRequest {
            login_id: "user@example.com".to_string(),
            password: "password123".to_string(),
        };

        let json = serde_json::to_string(&login).unwrap();
        assert!(json.contains("login_id"));
        assert!(json.contains("password"));
    }
}
