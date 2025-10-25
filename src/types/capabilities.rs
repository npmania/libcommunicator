//! Platform capabilities system
//!
//! Defines what features each platform supports, allowing consumers to query
//! and adapt to different platform capabilities.

use serde::{Deserialize, Serialize};

/// Platform capabilities and feature flags
///
/// This struct describes what features a particular platform implementation supports.
/// Consumers can check these flags before calling optional methods on the Platform trait.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformCapabilities {
    /// Platform name (e.g., "mattermost", "slack", "discord")
    pub platform_name: String,

    /// Platform version or API version
    pub platform_version: Option<String>,

    // Organizational features
    /// Does this platform have workspaces/teams/guilds?
    pub has_workspaces: bool,

    /// Does this platform support threaded conversations?
    pub has_threads: bool,

    // Messaging features
    /// Can messages be edited after posting?
    pub supports_message_editing: bool,

    /// Can messages be deleted?
    pub supports_message_deletion: bool,

    /// Does the platform support message reactions/emoji?
    pub supports_reactions: bool,

    /// Does the platform support file attachments?
    pub supports_file_attachments: bool,

    /// Does the platform support rich text/markdown?
    pub supports_rich_text: bool,

    // Status and presence
    /// Does the platform support basic user status (online/away/dnd/offline)?
    pub supports_status: bool,

    /// Does the platform support custom status messages?
    pub supports_custom_status: bool,

    /// Does the platform support user typing indicators?
    pub supports_typing_indicators: bool,

    // Channel features
    /// Can users create public channels?
    pub supports_public_channels: bool,

    /// Can users create private channels?
    pub supports_private_channels: bool,

    /// Does the platform support direct messages?
    pub supports_direct_messages: bool,

    /// Does the platform support group direct messages?
    pub supports_group_messages: bool,

    // Real-time features
    /// Does the platform support real-time event subscriptions?
    pub supports_realtime_events: bool,

    /// Does the platform support webhooks?
    pub supports_webhooks: bool,

    // Search and history
    /// Can users search message history?
    pub supports_search: bool,

    /// Can users load message history?
    pub supports_message_history: bool,
}

impl PlatformCapabilities {
    /// Create a new capabilities struct with all features disabled by default
    pub fn new(platform_name: impl Into<String>) -> Self {
        PlatformCapabilities {
            platform_name: platform_name.into(),
            platform_version: None,
            has_workspaces: false,
            has_threads: false,
            supports_message_editing: false,
            supports_message_deletion: false,
            supports_reactions: false,
            supports_file_attachments: false,
            supports_rich_text: false,
            supports_status: false,
            supports_custom_status: false,
            supports_typing_indicators: false,
            supports_public_channels: false,
            supports_private_channels: false,
            supports_direct_messages: false,
            supports_group_messages: false,
            supports_realtime_events: false,
            supports_webhooks: false,
            supports_search: false,
            supports_message_history: false,
        }
    }

    /// Set platform version
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.platform_version = Some(version.into());
        self
    }

    /// Enable workspace support
    pub fn with_workspaces(mut self) -> Self {
        self.has_workspaces = true;
        self
    }

    /// Enable thread support
    pub fn with_threads(mut self) -> Self {
        self.has_threads = true;
        self
    }

    /// Enable message editing
    pub fn with_message_editing(mut self) -> Self {
        self.supports_message_editing = true;
        self
    }

    /// Enable message deletion
    pub fn with_message_deletion(mut self) -> Self {
        self.supports_message_deletion = true;
        self
    }

    /// Enable reactions
    pub fn with_reactions(mut self) -> Self {
        self.supports_reactions = true;
        self
    }

    /// Enable file attachments
    pub fn with_file_attachments(mut self) -> Self {
        self.supports_file_attachments = true;
        self
    }

    /// Enable rich text support
    pub fn with_rich_text(mut self) -> Self {
        self.supports_rich_text = true;
        self
    }

    /// Enable basic status support
    pub fn with_status(mut self) -> Self {
        self.supports_status = true;
        self
    }

    /// Enable custom status messages
    pub fn with_custom_status(mut self) -> Self {
        self.supports_custom_status = true;
        self
    }

    /// Enable typing indicators
    pub fn with_typing_indicators(mut self) -> Self {
        self.supports_typing_indicators = true;
        self
    }

    /// Enable public channels
    pub fn with_public_channels(mut self) -> Self {
        self.supports_public_channels = true;
        self
    }

    /// Enable private channels
    pub fn with_private_channels(mut self) -> Self {
        self.supports_private_channels = true;
        self
    }

    /// Enable direct messages
    pub fn with_direct_messages(mut self) -> Self {
        self.supports_direct_messages = true;
        self
    }

    /// Enable group messages
    pub fn with_group_messages(mut self) -> Self {
        self.supports_group_messages = true;
        self
    }

    /// Enable real-time events
    pub fn with_realtime_events(mut self) -> Self {
        self.supports_realtime_events = true;
        self
    }

    /// Enable webhooks
    pub fn with_webhooks(mut self) -> Self {
        self.supports_webhooks = true;
        self
    }

    /// Enable search
    pub fn with_search(mut self) -> Self {
        self.supports_search = true;
        self
    }

    /// Enable message history
    pub fn with_message_history(mut self) -> Self {
        self.supports_message_history = true;
        self
    }
}

/// Preset capabilities for common platforms
impl PlatformCapabilities {
    /// Create capabilities for Mattermost
    pub fn mattermost() -> Self {
        PlatformCapabilities::new("mattermost")
            .with_version("v4")
            .with_workspaces()
            .with_threads()
            .with_message_editing()
            .with_message_deletion()
            .with_reactions()
            .with_file_attachments()
            .with_rich_text()
            .with_status()
            .with_custom_status()
            .with_typing_indicators()
            .with_public_channels()
            .with_private_channels()
            .with_direct_messages()
            .with_group_messages()
            .with_realtime_events()
            .with_webhooks()
            .with_search()
            .with_message_history()
    }

    /// Create capabilities for Slack
    pub fn slack() -> Self {
        PlatformCapabilities::new("slack")
            .with_workspaces()
            .with_threads()
            .with_message_editing()
            .with_message_deletion()
            .with_reactions()
            .with_file_attachments()
            .with_rich_text()
            .with_status()
            .with_custom_status()
            .with_public_channels()
            .with_private_channels()
            .with_direct_messages()
            .with_group_messages()
            .with_realtime_events()
            .with_webhooks()
            .with_search()
            .with_message_history()
    }

    /// Create capabilities for Discord
    pub fn discord() -> Self {
        PlatformCapabilities::new("discord")
            .with_workspaces() // Discord has "guilds/servers"
            .with_threads()
            .with_message_editing()
            .with_message_deletion()
            .with_reactions()
            .with_file_attachments()
            .with_rich_text()
            .with_status()
            .with_custom_status()
            .with_typing_indicators()
            .with_public_channels()
            .with_private_channels()
            .with_direct_messages()
            .with_group_messages()
            .with_realtime_events()
            .with_webhooks()
            .with_message_history()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_capabilities() {
        let caps = PlatformCapabilities::new("test-platform");
        assert_eq!(caps.platform_name, "test-platform");
        assert!(!caps.has_workspaces);
        assert!(!caps.supports_reactions);
    }

    #[test]
    fn test_builder_pattern() {
        let caps = PlatformCapabilities::new("custom")
            .with_workspaces()
            .with_reactions()
            .with_version("1.0");

        assert!(caps.has_workspaces);
        assert!(caps.supports_reactions);
        assert_eq!(caps.platform_version, Some("1.0".to_string()));
    }

    #[test]
    fn test_mattermost_preset() {
        let caps = PlatformCapabilities::mattermost();
        assert_eq!(caps.platform_name, "mattermost");
        assert!(caps.has_workspaces);
        assert!(caps.has_threads);
        assert!(caps.supports_custom_status);
    }

    #[test]
    fn test_slack_preset() {
        let caps = PlatformCapabilities::slack();
        assert_eq!(caps.platform_name, "slack");
        assert!(caps.has_workspaces);
        assert!(caps.supports_custom_status);
    }

    #[test]
    fn test_discord_preset() {
        let caps = PlatformCapabilities::discord();
        assert_eq!(caps.platform_name, "discord");
        assert!(caps.has_workspaces); // Discord guilds
        assert!(caps.supports_typing_indicators);
    }
}
