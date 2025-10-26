//! Connection state and information types

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Information about an active connection to a platform
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionInfo {
    /// Platform identifier (e.g., "mattermost", "slack")
    pub platform: String,
    /// Server URL or identifier
    pub server: String,
    /// Connected user ID
    pub user_id: String,
    /// Connected user's display name
    pub user_display_name: String,
    /// When the connection was established
    pub connected_at: DateTime<Utc>,
    /// Current connection state
    pub state: ConnectionState,
    /// Optional team/workspace identifier
    pub team_id: Option<String>,
    /// Optional team/workspace name
    pub team_name: Option<String>,
    /// Optional metadata (platform-specific)
    pub metadata: Option<serde_json::Value>,
}

/// Connection state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum ConnectionState {
    /// Currently connecting/authenticating
    Connecting,
    /// Successfully connected and authenticated
    Connected,
    /// Connection is being closed
    Disconnecting,
    /// Not connected
    #[default]
    Disconnected,
    /// Connection failed or encountered an error
    Error,
    /// Connection is being reconnected
    Reconnecting,
}

impl ConnectionInfo {
    /// Create a new connection info
    pub fn new(
        platform: impl Into<String>,
        server: impl Into<String>,
        user_id: impl Into<String>,
        user_display_name: impl Into<String>,
    ) -> Self {
        ConnectionInfo {
            platform: platform.into(),
            server: server.into(),
            user_id: user_id.into(),
            user_display_name: user_display_name.into(),
            connected_at: Utc::now(),
            state: ConnectionState::Connected,
            team_id: None,
            team_name: None,
            metadata: None,
        }
    }

    /// Set team information
    pub fn with_team(mut self, team_id: impl Into<String>, team_name: impl Into<String>) -> Self {
        self.team_id = Some(team_id.into());
        self.team_name = Some(team_name.into());
        self
    }

    /// Set connection state
    pub fn with_state(mut self, state: ConnectionState) -> Self {
        self.state = state;
        self
    }

    /// Set metadata
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Check if connection is active (connected state)
    pub fn is_connected(&self) -> bool {
        self.state == ConnectionState::Connected
    }

    /// Check if connection is in progress
    pub fn is_connecting(&self) -> bool {
        matches!(
            self.state,
            ConnectionState::Connecting | ConnectionState::Reconnecting
        )
    }

    /// Check if connection has failed
    pub fn is_error(&self) -> bool {
        self.state == ConnectionState::Error
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_info_creation() {
        let info = ConnectionInfo::new(
            "mattermost",
            "https://chat.example.com",
            "user-123",
            "Alice Smith",
        );
        assert_eq!(info.platform, "mattermost");
        assert_eq!(info.server, "https://chat.example.com");
        assert_eq!(info.user_id, "user-123");
        assert_eq!(info.user_display_name, "Alice Smith");
        assert_eq!(info.state, ConnectionState::Connected);
        assert!(info.is_connected());
    }

    #[test]
    fn test_connection_with_team() {
        let info = ConnectionInfo::new("slack", "workspace.slack.com", "U123", "Bob")
            .with_team("T456", "Engineering");
        assert_eq!(info.team_id, Some("T456".to_string()));
        assert_eq!(info.team_name, Some("Engineering".to_string()));
    }

    #[test]
    fn test_connection_states() {
        let mut info = ConnectionInfo::new("mattermost", "server", "user-1", "User");

        info.state = ConnectionState::Connecting;
        assert!(info.is_connecting());
        assert!(!info.is_connected());
        assert!(!info.is_error());

        info.state = ConnectionState::Connected;
        assert!(info.is_connected());
        assert!(!info.is_connecting());

        info.state = ConnectionState::Error;
        assert!(info.is_error());
        assert!(!info.is_connected());
    }

    #[test]
    fn test_reconnecting_state() {
        let info = ConnectionInfo::new("mattermost", "server", "user-1", "User")
            .with_state(ConnectionState::Reconnecting);
        assert!(info.is_connecting());
        assert!(!info.is_connected());
    }
}
