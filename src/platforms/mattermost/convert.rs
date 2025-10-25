use chrono::{DateTime, Utc};

use crate::types::user::UserStatus;
use crate::types::{Attachment, Channel, ChannelType, Message, Team, TeamType, User};

use super::channels::get_dm_partner_id;
use super::types::{FileInfo, MattermostChannel, MattermostPost, MattermostTeam, MattermostUser};

/// Context for converting Mattermost types to generic types
/// Provides necessary information like server URL and current user ID
#[derive(Clone)]
pub struct ConversionContext {
    pub server_url: String,
    pub current_user_id: Option<String>,
}

impl ConversionContext {
    pub fn new(server_url: String) -> Self {
        Self {
            server_url,
            current_user_id: None,
        }
    }

    pub fn with_current_user(mut self, user_id: String) -> Self {
        self.current_user_id = Some(user_id);
        self
    }
}

/// Convert a Mattermost timestamp (milliseconds since epoch) to DateTime<Utc>
fn timestamp_to_datetime(timestamp_ms: i64) -> DateTime<Utc> {
    DateTime::from_timestamp(timestamp_ms / 1000, ((timestamp_ms % 1000) * 1_000_000) as u32)
        .unwrap_or_else(|| Utc::now())
}

impl MattermostUser {
    /// Convert to User with context for proper URL construction
    pub fn to_user_with_context(&self, ctx: &ConversionContext) -> User {
        // Determine display name from available fields
        let display_name = if !self.first_name.is_empty() || !self.last_name.is_empty() {
            format!("{} {}", self.first_name, self.last_name).trim().to_string()
        } else if !self.nickname.is_empty() {
            self.nickname.clone()
        } else {
            self.username.clone()
        };

        // Create metadata with Mattermost-specific fields
        let metadata = serde_json::json!({
            "first_name": self.first_name,
            "last_name": self.last_name,
            "nickname": self.nickname,
            "position": self.position,
            "roles": self.roles,
            "locale": self.locale,
            "timezone": self.timezone,
            "props": self.props,
            "create_at": self.create_at,
            "update_at": self.update_at,
        });

        let mut user = User::new(self.id.clone(), self.username.clone(), display_name);

        if !self.email.is_empty() {
            user = user.with_email(self.email.clone());
        }

        // Construct avatar URL with server context
        let avatar_url = format!("{}/api/v4/users/{}/image", ctx.server_url, self.id);
        user = user.with_avatar(avatar_url);

        if self.is_bot {
            user = user.as_bot();
        }

        user.with_metadata(metadata)
    }
}

/// Convert Mattermost User to our internal User type (without context)
impl From<MattermostUser> for User {
    fn from(mm_user: MattermostUser) -> Self {
        // Use a basic context with empty server URL for backwards compatibility
        let ctx = ConversionContext::new(String::new());
        mm_user.to_user_with_context(&ctx)
    }
}

/// Convert Mattermost Post to our internal Message type
impl From<MattermostPost> for Message {
    fn from(mm_post: MattermostPost) -> Self {
        let created_at = timestamp_to_datetime(mm_post.create_at);
        let edited_at = if mm_post.edit_at > 0 {
            Some(timestamp_to_datetime(mm_post.edit_at))
        } else {
            None
        };

        // Convert file attachments
        let attachments: Vec<Attachment> = mm_post
            .metadata
            .files
            .into_iter()
            .map(|file| file.into())
            .collect();

        // Create metadata with Mattermost-specific fields
        let metadata = serde_json::json!({
            "root_id": mm_post.root_id,
            "parent_id": mm_post.parent_id,
            "post_type": mm_post.post_type,
            "props": mm_post.props,
            "hashtags": mm_post.hashtags,
            "update_at": mm_post.update_at,
            "delete_at": mm_post.delete_at,
        });

        let mut message = Message::new(
            mm_post.id,
            mm_post.message,
            mm_post.user_id,
            mm_post.channel_id,
        );

        // Override the created_at with the actual timestamp
        message.created_at = created_at;
        message.edited_at = edited_at;
        message.attachments = attachments;
        message = message.with_metadata(metadata);

        message
    }
}

impl FileInfo {
    /// Convert to Attachment with context for proper URL construction
    pub fn to_attachment_with_context(&self, ctx: &ConversionContext) -> Attachment {
        // Construct the full file URL with server context
        let url = format!("{}/api/v4/files/{}", ctx.server_url, self.id);

        let mut attachment = Attachment::new(
            self.id.clone(),
            self.name.clone(),
            self.mime_type.clone(),
            self.size as u64,
            url,
        );

        // Add thumbnail if available
        if self.has_preview_image {
            let thumbnail_url = format!("{}/api/v4/files/{}/thumbnail", ctx.server_url, self.id);
            attachment = attachment.with_thumbnail(thumbnail_url);
        }

        attachment
    }
}

/// Convert Mattermost FileInfo to our internal Attachment type (without context)
impl From<FileInfo> for Attachment {
    fn from(file: FileInfo) -> Self {
        // Use a basic context with empty server URL for backwards compatibility
        let ctx = ConversionContext::new(String::new());
        file.to_attachment_with_context(&ctx)
    }
}

impl MattermostChannel {
    /// Convert to Channel with context for better DM display names
    pub fn to_channel_with_context(&self, ctx: &ConversionContext) -> Channel {
        use super::types::MattermostChannelType;

        // Map Mattermost channel type to our ChannelType
        let channel_type = match self.channel_type {
            MattermostChannelType::Open => ChannelType::Public,
            MattermostChannelType::Private => ChannelType::Private,
            MattermostChannelType::Direct => ChannelType::DirectMessage,
            MattermostChannelType::Group => ChannelType::GroupMessage,
        };

        let created_at = timestamp_to_datetime(self.create_at);
        let last_activity_at = if self.last_post_at > 0 {
            Some(timestamp_to_datetime(self.last_post_at))
        } else {
            None
        };

        // Create metadata with Mattermost-specific fields
        let mut metadata = serde_json::json!({
            "team_id": self.team_id,
            "total_msg_count": self.total_msg_count,
            "creator_id": self.creator_id,
            "update_at": self.update_at,
            "delete_at": self.delete_at,
        });

        // For DM channels, try to extract partner user ID from the "name" field
        // Note: DM channel "name" field contains user IDs in format "user1id__user2id"
        if self.channel_type.is_direct() {
            if let Some(ref user_id) = ctx.current_user_id {
                if let Some(partner_id) = get_dm_partner_id(&self.name, user_id) {
                    metadata["dm_partner_id"] = serde_json::json!(partner_id);
                }
            }
        }

        let mut channel = Channel::new(
            self.id.clone(),
            self.name.clone(),
            self.display_name.clone(),
            channel_type,
        );

        // Override the created_at with the actual timestamp
        channel.created_at = created_at;

        if let Some(last_activity) = last_activity_at {
            channel = channel.with_last_activity(last_activity);
        }

        if !self.header.is_empty() {
            channel = channel.with_topic(self.header.clone());
        }

        if !self.purpose.is_empty() {
            channel = channel.with_purpose(self.purpose.clone());
        }

        if self.delete_at > 0 {
            channel = channel.archived();
        }

        channel.with_metadata(metadata)
    }
}

/// Convert Mattermost Channel to our internal Channel type (without context)
impl From<MattermostChannel> for Channel {
    fn from(mm_channel: MattermostChannel) -> Self {
        // Use a basic context with empty server URL for backwards compatibility
        let ctx = ConversionContext::new(String::new());
        mm_channel.to_channel_with_context(&ctx)
    }
}

/// Convert Mattermost Team to our internal Team type
impl From<MattermostTeam> for Team {
    fn from(mm_team: MattermostTeam) -> Self {
        // Map Mattermost team type ("O" or "I") to TeamType enum
        let team_type = match mm_team.team_type.as_str() {
            "O" => TeamType::Open,
            "I" => TeamType::Invite,
            _ => TeamType::Invite, // Default to invite-only
        };

        // Create metadata with Mattermost-specific fields
        let metadata = serde_json::json!({
            "company_name": mm_team.company_name,
            "email": mm_team.email,
            "invite_id": mm_team.invite_id,
            "create_at": mm_team.create_at,
            "update_at": mm_team.update_at,
            "delete_at": mm_team.delete_at,
        });

        let description = if mm_team.description.is_empty() {
            None
        } else {
            Some(mm_team.description)
        };

        let allowed_domains = if mm_team.allowed_domains.is_empty() {
            None
        } else {
            Some(mm_team.allowed_domains)
        };

        Team {
            id: mm_team.id,
            name: mm_team.name,
            display_name: mm_team.display_name,
            description,
            team_type,
            allowed_domains,
            allow_open_invite: mm_team.allow_open_invite,
            metadata: Some(metadata),
        }
    }
}

/// Helper function to convert a status string to UserStatus
pub fn status_string_to_user_status(status: &str) -> UserStatus {
    match status {
        "online" => UserStatus::Online,
        "away" => UserStatus::Away,
        "dnd" | "do_not_disturb" => UserStatus::DoNotDisturb,
        "offline" => UserStatus::Offline,
        _ => UserStatus::Unknown,
    }
}

/// Helper function to convert UserStatus to a status string
pub fn user_status_to_status_string(status: UserStatus) -> &'static str {
    match status {
        UserStatus::Online => "online",
        UserStatus::Away => "away",
        UserStatus::DoNotDisturb => "dnd",
        UserStatus::Offline => "offline",
        UserStatus::Unknown => "offline",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_conversion() {
        let mm_user = MattermostUser {
            id: "user123".to_string(),
            username: "alice".to_string(),
            email: "alice@example.com".to_string(),
            first_name: "Alice".to_string(),
            last_name: "Smith".to_string(),
            nickname: "".to_string(),
            position: "Developer".to_string(),
            roles: "system_user".to_string(),
            locale: "en".to_string(),
            timezone: Default::default(),
            props: Default::default(),
            is_bot: false,
            create_at: 1234567890000,
            update_at: 1234567890000,
            delete_at: 0,
        };

        let user: User = mm_user.into();
        assert_eq!(user.id, "user123");
        assert_eq!(user.username, "alice");
        assert_eq!(user.display_name, "Alice Smith");
        assert_eq!(user.email, Some("alice@example.com".to_string()));
        assert!(!user.is_bot);
    }

    #[test]
    fn test_channel_type_conversion() {
        use crate::platforms::mattermost::types::MattermostChannelType;

        let mm_channel = MattermostChannel {
            id: "ch123".to_string(),
            create_at: 1234567890000,
            update_at: 1234567890000,
            delete_at: 0,
            team_id: "team1".to_string(),
            channel_type: MattermostChannelType::Open,
            display_name: "General".to_string(),
            name: "general".to_string(),
            header: "Welcome!".to_string(),
            purpose: "General discussion".to_string(),
            last_post_at: 0,
            total_msg_count: 42,
            creator_id: "user1".to_string(),
        };

        let channel: Channel = mm_channel.into();
        assert_eq!(channel.id, "ch123");
        assert_eq!(channel.name, "general");
        assert_eq!(channel.channel_type, ChannelType::Public);
        assert_eq!(channel.topic, Some("Welcome!".to_string()));
        assert_eq!(channel.purpose, Some("General discussion".to_string()));
    }

    #[test]
    fn test_status_string_conversion() {
        assert_eq!(status_string_to_user_status("online"), UserStatus::Online);
        assert_eq!(status_string_to_user_status("away"), UserStatus::Away);
        assert_eq!(status_string_to_user_status("dnd"), UserStatus::DoNotDisturb);
        assert_eq!(status_string_to_user_status("offline"), UserStatus::Offline);
        assert_eq!(status_string_to_user_status("unknown"), UserStatus::Unknown);
    }

    #[test]
    fn test_timestamp_conversion() {
        let timestamp_ms = 1234567890000i64;
        let dt = timestamp_to_datetime(timestamp_ms);
        assert_eq!(dt.timestamp(), 1234567890);
    }

    #[test]
    fn test_team_conversion() {
        let mm_team = MattermostTeam {
            id: "team123".to_string(),
            create_at: 1234567890000,
            update_at: 1234567890000,
            delete_at: 0,
            display_name: "Engineering Team".to_string(),
            name: "engineering".to_string(),
            description: "Our engineering team".to_string(),
            email: "eng@example.com".to_string(),
            team_type: "O".to_string(),
            company_name: "ACME Inc".to_string(),
            allowed_domains: "example.com".to_string(),
            invite_id: "inv123".to_string(),
            allow_open_invite: true,
        };

        let team: Team = mm_team.into();
        assert_eq!(team.id, "team123");
        assert_eq!(team.name, "engineering");
        assert_eq!(team.display_name, "Engineering Team");
        assert_eq!(team.description, Some("Our engineering team".to_string()));
        assert_eq!(team.team_type, TeamType::Open);
        assert_eq!(team.allowed_domains, Some("example.com".to_string()));
        assert!(team.allow_open_invite);
    }

    #[test]
    fn test_team_conversion_invite_only() {
        let mm_team = MattermostTeam {
            id: "team456".to_string(),
            create_at: 1234567890000,
            update_at: 1234567890000,
            delete_at: 0,
            display_name: "Private Team".to_string(),
            name: "private".to_string(),
            description: String::new(),
            email: String::new(),
            team_type: "I".to_string(),
            company_name: String::new(),
            allowed_domains: String::new(),
            invite_id: String::new(),
            allow_open_invite: false,
        };

        let team: Team = mm_team.into();
        assert_eq!(team.team_type, TeamType::Invite);
        assert_eq!(team.description, None);
        assert_eq!(team.allowed_domains, None);
        assert!(!team.allow_open_invite);
    }
}
