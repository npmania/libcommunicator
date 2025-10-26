//! Channel types for chat platforms

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Represents a chat channel/conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Channel {
    /// Unique identifier for this channel
    pub id: String,
    /// Channel name
    pub name: String,
    /// Human-readable display name
    pub display_name: String,
    /// Type of channel (public, private, direct message, etc.)
    #[serde(rename = "type")]
    pub channel_type: ChannelType,
    /// Optional channel topic/description
    pub topic: Option<String>,
    /// Optional channel purpose
    pub purpose: Option<String>,
    /// User IDs of channel members (may be None if not loaded)
    pub member_ids: Option<Vec<String>>,
    /// When the channel was created
    pub created_at: DateTime<Utc>,
    /// Last activity timestamp
    pub last_activity_at: Option<DateTime<Utc>>,
    /// Whether the channel is archived
    pub is_archived: bool,
    /// Optional metadata (platform-specific)
    pub metadata: Option<serde_json::Value>,
}

/// Type of channel
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChannelType {
    /// Public channel (anyone can join)
    Public,
    /// Private channel (invite-only)
    Private,
    /// Direct message between two users
    DirectMessage,
    /// Group direct message (multiple users)
    GroupMessage,
}

impl Channel {
    /// Create a new channel
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        display_name: impl Into<String>,
        channel_type: ChannelType,
    ) -> Self {
        Channel {
            id: id.into(),
            name: name.into(),
            display_name: display_name.into(),
            channel_type,
            topic: None,
            purpose: None,
            member_ids: None,
            created_at: Utc::now(),
            last_activity_at: None,
            is_archived: false,
            metadata: None,
        }
    }

    /// Set channel topic
    pub fn with_topic(mut self, topic: impl Into<String>) -> Self {
        self.topic = Some(topic.into());
        self
    }

    /// Set channel purpose
    pub fn with_purpose(mut self, purpose: impl Into<String>) -> Self {
        self.purpose = Some(purpose.into());
        self
    }

    /// Set member IDs
    pub fn with_members(mut self, member_ids: Vec<String>) -> Self {
        self.member_ids = Some(member_ids);
        self
    }

    /// Set last activity timestamp
    pub fn with_last_activity(mut self, timestamp: DateTime<Utc>) -> Self {
        self.last_activity_at = Some(timestamp);
        self
    }

    /// Mark as archived
    pub fn archived(mut self) -> Self {
        self.is_archived = true;
        self
    }

    /// Set metadata
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Check if this is a direct message channel
    pub fn is_direct_message(&self) -> bool {
        matches!(
            self.channel_type,
            ChannelType::DirectMessage | ChannelType::GroupMessage
        )
    }

    /// Check if this is a public channel
    pub fn is_public(&self) -> bool {
        self.channel_type == ChannelType::Public
    }
}

/// Unread information for a channel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelUnread {
    /// Channel ID
    pub channel_id: String,
    /// Team/workspace ID (if applicable)
    pub team_id: Option<String>,
    /// Number of unread messages
    pub msg_count: i64,
    /// Number of unread mentions
    pub mention_count: i64,
    /// Timestamp when the channel was last viewed (milliseconds since epoch)
    pub last_viewed_at: i64,
}

impl ChannelUnread {
    /// Create a new ChannelUnread instance
    pub fn new(channel_id: impl Into<String>) -> Self {
        ChannelUnread {
            channel_id: channel_id.into(),
            team_id: None,
            msg_count: 0,
            mention_count: 0,
            last_viewed_at: 0,
        }
    }

    /// Set team ID
    pub fn with_team(mut self, team_id: impl Into<String>) -> Self {
        self.team_id = Some(team_id.into());
        self
    }

    /// Set unread counts
    pub fn with_counts(mut self, msg_count: i64, mention_count: i64) -> Self {
        self.msg_count = msg_count;
        self.mention_count = mention_count;
        self
    }

    /// Set last viewed timestamp
    pub fn with_last_viewed(mut self, last_viewed_at: i64) -> Self {
        self.last_viewed_at = last_viewed_at;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_creation() {
        let channel = Channel::new("ch-1", "general", "General", ChannelType::Public);
        assert_eq!(channel.id, "ch-1");
        assert_eq!(channel.name, "general");
        assert_eq!(channel.display_name, "General");
        assert_eq!(channel.channel_type, ChannelType::Public);
        assert!(!channel.is_archived);
    }

    #[test]
    fn test_channel_builder() {
        let channel = Channel::new("ch-1", "team-chat", "Team Chat", ChannelType::Private)
            .with_topic("Team discussions")
            .with_purpose("Internal team communication")
            .with_members(vec!["user-1".to_string(), "user-2".to_string()]);

        assert_eq!(channel.topic, Some("Team discussions".to_string()));
        assert_eq!(channel.purpose, Some("Internal team communication".to_string()));
        assert_eq!(channel.member_ids.as_ref().unwrap().len(), 2);
    }

    #[test]
    fn test_direct_message_channel() {
        let dm = Channel::new("dm-1", "alice-bob", "Alice & Bob", ChannelType::DirectMessage);
        assert!(dm.is_direct_message());
        assert!(!dm.is_public());
    }

    #[test]
    fn test_public_channel() {
        let public = Channel::new("ch-1", "announcements", "Announcements", ChannelType::Public);
        assert!(public.is_public());
        assert!(!public.is_direct_message());
    }

    #[test]
    fn test_archived_channel() {
        let channel = Channel::new("ch-1", "old-project", "Old Project", ChannelType::Private)
            .archived();
        assert!(channel.is_archived);
    }

    #[test]
    fn test_channel_json_serialization() {
        let channel = Channel::new("ch-1", "general", "General", ChannelType::Public);
        let json = serde_json::to_string(&channel).unwrap();

        // Verify that the JSON contains "type" not "channel_type"
        assert!(json.contains(r#""type":"public"#));
        assert!(!json.contains("channel_type"));
    }

    #[test]
    fn test_channel_json_deserialization() {
        // Test that we can deserialize from JSON with "type" field
        let json = r#"{
            "id": "ch-123",
            "name": "test-channel",
            "display_name": "Test Channel",
            "type": "private",
            "topic": null,
            "purpose": null,
            "member_ids": null,
            "created_at": "2024-01-01T00:00:00Z",
            "last_activity_at": null,
            "is_archived": false,
            "metadata": null
        }"#;

        let channel: Channel = serde_json::from_str(json).unwrap();
        assert_eq!(channel.id, "ch-123");
        assert_eq!(channel.channel_type, ChannelType::Private);
    }
}
