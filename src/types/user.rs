//! User types for chat platforms

use serde::{Deserialize, Serialize};

/// Represents a user on a chat platform
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// Unique identifier for this user
    pub id: String,
    /// Username (unique login name)
    pub username: String,
    /// Display name (what other users see)
    pub display_name: String,
    /// Email address (optional)
    pub email: Option<String>,
    /// Avatar/profile picture URL (optional)
    pub avatar_url: Option<String>,
    /// Current status (online, away, offline, etc.)
    pub status: UserStatus,
    /// Optional custom status message/text set by the user (e.g., "In a meeting", "Working remotely")
    /// Note: Not all platforms support custom status messages - check PlatformCapabilities.supports_custom_status
    pub status_message: Option<String>,
    /// Whether this user is a bot
    pub is_bot: bool,
    /// Optional metadata (platform-specific)
    pub metadata: Option<serde_json::Value>,
}

/// User status/presence
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum UserStatus {
    /// User is online and active
    Online,
    /// User is away/idle
    Away,
    /// User is in "do not disturb" mode
    DoNotDisturb,
    /// User is offline
    Offline,
    /// Status is unknown
    #[default]
    Unknown,
}

impl User {
    /// Create a new user
    pub fn new(
        id: impl Into<String>,
        username: impl Into<String>,
        display_name: impl Into<String>,
    ) -> Self {
        User {
            id: id.into(),
            username: username.into(),
            display_name: display_name.into(),
            email: None,
            avatar_url: None,
            status: UserStatus::Unknown,
            status_message: None,
            is_bot: false,
            metadata: None,
        }
    }

    /// Set email address
    pub fn with_email(mut self, email: impl Into<String>) -> Self {
        self.email = Some(email.into());
        self
    }

    /// Set avatar URL
    pub fn with_avatar(mut self, avatar_url: impl Into<String>) -> Self {
        self.avatar_url = Some(avatar_url.into());
        self
    }

    /// Set user status
    pub fn with_status(mut self, status: UserStatus) -> Self {
        self.status = status;
        self
    }

    /// Set status message
    pub fn with_status_message(mut self, message: impl Into<String>) -> Self {
        self.status_message = Some(message.into());
        self
    }

    /// Mark as bot
    pub fn as_bot(mut self) -> Self {
        self.is_bot = true;
        self
    }

    /// Set metadata
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_creation() {
        let user = User::new("user-1", "alice", "Alice Smith");
        assert_eq!(user.id, "user-1");
        assert_eq!(user.username, "alice");
        assert_eq!(user.display_name, "Alice Smith");
        assert!(user.email.is_none());
        assert!(!user.is_bot);
        assert_eq!(user.status, UserStatus::Unknown);
    }

    #[test]
    fn test_user_builder() {
        let user = User::new("user-1", "bob", "Bob Jones")
            .with_email("bob@example.com")
            .with_status(UserStatus::Online)
            .with_avatar("https://example.com/avatar.png");

        assert_eq!(user.email, Some("bob@example.com".to_string()));
        assert_eq!(user.status, UserStatus::Online);
        assert_eq!(user.avatar_url, Some("https://example.com/avatar.png".to_string()));
    }

    #[test]
    fn test_bot_user() {
        let bot = User::new("bot-1", "helper-bot", "Helper Bot").as_bot();
        assert!(bot.is_bot);
    }

    #[test]
    fn test_user_status_serialization() {
        let status = UserStatus::Online;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"online\"");
    }
}
