//! Platform trait defining the interface all platform adapters must implement

use crate::error::Result;
use crate::types::{Channel, ConnectionInfo, Message, User};
use async_trait::async_trait;
use std::collections::HashMap;

/// Configuration for connecting to a platform
#[derive(Debug, Clone)]
pub struct PlatformConfig {
    /// Server URL or endpoint
    pub server: String,
    /// Authentication credentials (e.g., token, username/password)
    pub credentials: HashMap<String, String>,
    /// Optional team/workspace identifier
    pub team_id: Option<String>,
    /// Additional platform-specific configuration
    pub extra: HashMap<String, String>,
}

impl PlatformConfig {
    /// Create a new platform configuration
    pub fn new(server: impl Into<String>) -> Self {
        PlatformConfig {
            server: server.into(),
            credentials: HashMap::new(),
            team_id: None,
            extra: HashMap::new(),
        }
    }

    /// Add a credential
    pub fn with_credential(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.credentials.insert(key.into(), value.into());
        self
    }

    /// Set team/workspace ID
    pub fn with_team(mut self, team_id: impl Into<String>) -> Self {
        self.team_id = Some(team_id.into());
        self
    }

    /// Add extra configuration
    pub fn with_extra(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.extra.insert(key.into(), value.into());
        self
    }
}

/// Event types that can be received from a platform
#[derive(Debug, Clone)]
pub enum PlatformEvent {
    /// A new message was posted
    MessagePosted(Message),
    /// A message was updated/edited
    MessageUpdated(Message),
    /// A message was deleted
    MessageDeleted { message_id: String, channel_id: String },
    /// A user's status changed
    UserStatusChanged { user_id: String, status: crate::types::user::UserStatus },
    /// A user started typing
    UserTyping { user_id: String, channel_id: String },
    /// A channel was created
    ChannelCreated(Channel),
    /// A channel was updated
    ChannelUpdated(Channel),
    /// A channel was deleted
    ChannelDeleted { channel_id: String },
    /// User joined a channel
    UserJoinedChannel { user_id: String, channel_id: String },
    /// User left a channel
    UserLeftChannel { user_id: String, channel_id: String },
    /// Connection state changed
    ConnectionStateChanged(crate::types::connection::ConnectionState),
}

/// Trait that all platform adapters must implement
///
/// This defines the common interface for interacting with different chat platforms
/// (Mattermost, Slack, Discord, etc.)
#[async_trait]
pub trait Platform: Send + Sync {
    /// Connect to the platform and authenticate
    ///
    /// # Arguments
    /// * `config` - Configuration including server URL and credentials
    ///
    /// # Returns
    /// Connection information on success
    async fn connect(&mut self, config: PlatformConfig) -> Result<ConnectionInfo>;

    /// Disconnect from the platform
    async fn disconnect(&mut self) -> Result<()>;

    /// Get current connection information
    ///
    /// Returns None if not connected
    fn connection_info(&self) -> Option<&ConnectionInfo>;

    /// Check if currently connected
    fn is_connected(&self) -> bool {
        self.connection_info()
            .map(|info| info.is_connected())
            .unwrap_or(false)
    }

    /// Send a message to a channel
    ///
    /// # Arguments
    /// * `channel_id` - The channel to send the message to
    /// * `text` - The message text
    ///
    /// # Returns
    /// The created message
    async fn send_message(&self, channel_id: &str, text: &str) -> Result<Message>;

    /// Get a list of channels the user has access to
    async fn get_channels(&self) -> Result<Vec<Channel>>;

    /// Get details about a specific channel
    async fn get_channel(&self, channel_id: &str) -> Result<Channel>;

    /// Get recent messages from a channel
    ///
    /// # Arguments
    /// * `channel_id` - The channel ID
    /// * `limit` - Maximum number of messages to retrieve
    ///
    /// # Returns
    /// List of messages, most recent first
    async fn get_messages(&self, channel_id: &str, limit: usize) -> Result<Vec<Message>>;

    /// Get a list of users in a channel
    async fn get_channel_members(&self, channel_id: &str) -> Result<Vec<User>>;

    /// Get details about a specific user
    async fn get_user(&self, user_id: &str) -> Result<User>;

    /// Get details about the currently authenticated user
    async fn get_current_user(&self) -> Result<User>;

    /// Create a direct message channel with another user
    ///
    /// # Arguments
    /// * `user_id` - The user to create a DM channel with
    ///
    /// # Returns
    /// The created or existing DM channel
    async fn create_direct_channel(&self, user_id: &str) -> Result<Channel>;

    /// Subscribe to real-time events (WebSocket, webhook, etc.)
    ///
    /// This method should establish a connection for receiving real-time events.
    /// Events should be delivered through the event callback.
    async fn subscribe_events(&mut self) -> Result<()>;

    /// Unsubscribe from real-time events
    async fn unsubscribe_events(&mut self) -> Result<()>;

    /// Poll for the next event (if available)
    ///
    /// This is a non-blocking check for new events.
    /// Returns None if no events are available.
    async fn poll_event(&mut self) -> Result<Option<PlatformEvent>>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_config_builder() {
        let config = PlatformConfig::new("https://chat.example.com")
            .with_credential("token", "secret-token")
            .with_team("team-123")
            .with_extra("timeout", "30");

        assert_eq!(config.server, "https://chat.example.com");
        assert_eq!(config.credentials.get("token"), Some(&"secret-token".to_string()));
        assert_eq!(config.team_id, Some("team-123".to_string()));
        assert_eq!(config.extra.get("timeout"), Some(&"30".to_string()));
    }
}
