use chrono::{DateTime, Utc};

use crate::types::user::UserStatus;
use crate::types::{Attachment, Channel, ChannelType, Message, User};

use super::types::{FileInfo, MattermostChannel, MattermostPost, MattermostUser};

/// Convert a Mattermost timestamp (milliseconds since epoch) to DateTime<Utc>
fn timestamp_to_datetime(timestamp_ms: i64) -> DateTime<Utc> {
    DateTime::from_timestamp(timestamp_ms / 1000, ((timestamp_ms % 1000) * 1_000_000) as u32)
        .unwrap_or_else(|| Utc::now())
}

/// Convert Mattermost User to our internal User type
impl From<MattermostUser> for User {
    fn from(mm_user: MattermostUser) -> Self {
        // Determine display name from available fields
        let display_name = if !mm_user.first_name.is_empty() || !mm_user.last_name.is_empty() {
            format!("{} {}", mm_user.first_name, mm_user.last_name).trim().to_string()
        } else if !mm_user.nickname.is_empty() {
            mm_user.nickname.clone()
        } else {
            mm_user.username.clone()
        };

        // Create metadata with Mattermost-specific fields
        let metadata = serde_json::json!({
            "first_name": mm_user.first_name,
            "last_name": mm_user.last_name,
            "nickname": mm_user.nickname,
            "position": mm_user.position,
            "roles": mm_user.roles,
            "locale": mm_user.locale,
            "timezone": mm_user.timezone,
            "props": mm_user.props,
            "create_at": mm_user.create_at,
            "update_at": mm_user.update_at,
        });

        let mut user = User::new(mm_user.id, mm_user.username, display_name);

        if !mm_user.email.is_empty() {
            user = user.with_email(mm_user.email);
        }

        // Note: Mattermost doesn't provide avatar URL in the user object directly
        // You would typically construct it as: {server_url}/api/v4/users/{user_id}/image

        if mm_user.is_bot {
            user = user.as_bot();
        }

        user.with_metadata(metadata)
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

/// Convert Mattermost FileInfo to our internal Attachment type
impl From<FileInfo> for Attachment {
    fn from(file: FileInfo) -> Self {
        // Construct the file URL
        // In a real implementation, you'd need the server URL
        // Format: {server_url}/api/v4/files/{file_id}
        let url = format!("/api/v4/files/{}", file.id);

        let mut attachment = Attachment::new(
            file.id,
            file.name,
            file.mime_type,
            file.size as u64,
            url,
        );

        // Add thumbnail if available
        if file.has_preview_image {
            let thumbnail_url = format!("/api/v4/files/{}/thumbnail", file.post_id);
            attachment = attachment.with_thumbnail(thumbnail_url);
        }

        attachment
    }
}

/// Convert Mattermost Channel to our internal Channel type
impl From<MattermostChannel> for Channel {
    fn from(mm_channel: MattermostChannel) -> Self {
        // Map Mattermost channel type to our ChannelType
        let channel_type = match mm_channel.channel_type.as_str() {
            "O" => ChannelType::Public,
            "P" => ChannelType::Private,
            "D" => ChannelType::DirectMessage,
            "G" => ChannelType::GroupMessage,
            _ => ChannelType::Public, // Default to public if unknown
        };

        let created_at = timestamp_to_datetime(mm_channel.create_at);
        let last_activity_at = if mm_channel.last_post_at > 0 {
            Some(timestamp_to_datetime(mm_channel.last_post_at))
        } else {
            None
        };

        // Create metadata with Mattermost-specific fields
        let metadata = serde_json::json!({
            "team_id": mm_channel.team_id,
            "total_msg_count": mm_channel.total_msg_count,
            "creator_id": mm_channel.creator_id,
            "update_at": mm_channel.update_at,
            "delete_at": mm_channel.delete_at,
        });

        let mut channel = Channel::new(
            mm_channel.id,
            mm_channel.name,
            mm_channel.display_name,
            channel_type,
        );

        // Override the created_at with the actual timestamp
        channel.created_at = created_at;

        if let Some(last_activity) = last_activity_at {
            channel = channel.with_last_activity(last_activity);
        }

        if !mm_channel.header.is_empty() {
            channel = channel.with_topic(mm_channel.header);
        }

        if !mm_channel.purpose.is_empty() {
            channel = channel.with_purpose(mm_channel.purpose);
        }

        if mm_channel.delete_at > 0 {
            channel = channel.archived();
        }

        channel.with_metadata(metadata)
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
        let mm_channel = MattermostChannel {
            id: "ch123".to_string(),
            create_at: 1234567890000,
            update_at: 1234567890000,
            delete_at: 0,
            team_id: "team1".to_string(),
            channel_type: "O".to_string(),
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
}
