use crate::error::Result;
use crate::types::ChannelType;

use super::client::MattermostClient;
use super::types::{ChannelMember, CreateDirectChannelRequest, CreateGroupChannelRequest, MattermostChannel};

/// Parse a direct message channel ID to extract participant user IDs
///
/// Mattermost DM channel IDs use the format: `{lower_user_id}__{higher_user_id}`
/// where user IDs are sorted alphabetically.
///
/// # Arguments
/// * `channel_id` - The channel ID to parse
///
/// # Returns
/// A tuple of (user_id_1, user_id_2) if the ID is a valid DM format, None otherwise
pub fn parse_dm_channel_id(channel_id: &str) -> Option<(String, String)> {
    if channel_id.contains("__") {
        let parts: Vec<&str> = channel_id.split("__").collect();
        if parts.len() == 2 {
            return Some((parts[0].to_string(), parts[1].to_string()));
        }
    }
    None
}

/// Get the other user's ID in a DM channel
///
/// # Arguments
/// * `channel_id` - The DM channel ID
/// * `current_user_id` - The current user's ID
///
/// # Returns
/// The other user's ID if this is a DM channel with the current user, None otherwise
pub fn get_dm_partner_id(channel_id: &str, current_user_id: &str) -> Option<String> {
    if let Some((user1, user2)) = parse_dm_channel_id(channel_id) {
        if user1 == current_user_id {
            return Some(user2);
        } else if user2 == current_user_id {
            return Some(user1);
        }
    }
    None
}

/// Detect channel type from ID format
///
/// Direct message channels have the format `user_id__user_id` (2 parts)
/// Group message channels have more than 2 parts separated by `__`
///
/// # Arguments
/// * `channel_id` - The channel ID to analyze
///
/// # Returns
/// The detected channel type, or None if the type cannot be determined from the ID
pub fn detect_channel_type_from_id(channel_id: &str) -> Option<ChannelType> {
    if channel_id.contains("__") {
        let parts: Vec<&str> = channel_id.split("__").collect();
        if parts.len() == 2 {
            Some(ChannelType::DirectMessage)
        } else if parts.len() > 2 {
            Some(ChannelType::GroupMessage)
        } else {
            None
        }
    } else {
        None // Use API-provided type
    }
}

impl MattermostClient {
    /// Get all channels for the current user in a specific team
    ///
    /// # Arguments
    /// * `team_id` - The ID of the team to get channels from
    ///
    /// # Returns
    /// A Result containing a list of channels or an Error
    pub async fn get_channels_for_team(&self, team_id: &str) -> Result<Vec<MattermostChannel>> {
        let endpoint = format!("/users/me/teams/{}/channels", team_id);
        let response = self.get(&endpoint).await?;
        self.handle_response(response).await
    }

    /// Get a channel by ID
    ///
    /// # Arguments
    /// * `channel_id` - The ID of the channel to retrieve
    ///
    /// # Returns
    /// A Result containing the channel information or an Error
    pub async fn get_channel(&self, channel_id: &str) -> Result<MattermostChannel> {
        let endpoint = format!("/channels/{}", channel_id);
        let response = self.get(&endpoint).await?;
        self.handle_response(response).await
    }

    /// Get a channel by name in a team
    ///
    /// # Arguments
    /// * `team_id` - The ID of the team
    /// * `channel_name` - The name of the channel
    ///
    /// # Returns
    /// A Result containing the channel information or an Error
    pub async fn get_channel_by_name(&self, team_id: &str, channel_name: &str) -> Result<MattermostChannel> {
        let endpoint = format!("/teams/{}/channels/name/{}", team_id, channel_name);
        let response = self.get(&endpoint).await?;
        self.handle_response(response).await
    }

    /// Create a direct message channel with another user
    ///
    /// # Arguments
    /// * `user_id` - The ID of the user to create a DM with
    ///
    /// # Returns
    /// A Result containing the created channel or an Error
    pub async fn create_direct_channel(&self, user_id: &str) -> Result<MattermostChannel> {
        let current_user_id = self.get_user_id().await.ok_or_else(|| {
            crate::error::Error::new(
                crate::error::ErrorCode::InvalidState,
                "User ID not set - ensure you're authenticated",
            )
        })?;

        let request = CreateDirectChannelRequest {
            user_ids: vec![current_user_id, user_id.to_string()],
        };

        let response = self.post("/channels/direct", &request).await?;
        self.handle_response(response).await
    }

    /// Create a group message channel with multiple users
    ///
    /// # Arguments
    /// * `user_ids` - A list of user IDs to include in the group message
    ///
    /// # Returns
    /// A Result containing the created channel or an Error
    pub async fn create_group_channel(&self, user_ids: Vec<String>) -> Result<MattermostChannel> {
        let request = CreateGroupChannelRequest { user_ids };

        let response = self.post("/channels/group", &request).await?;
        self.handle_response(response).await
    }

    /// Get the members of a channel
    ///
    /// # Arguments
    /// * `channel_id` - The ID of the channel
    ///
    /// # Returns
    /// A Result containing a list of channel members or an Error
    pub async fn get_channel_members(&self, channel_id: &str) -> Result<Vec<ChannelMember>> {
        let endpoint = format!("/channels/{}/members", channel_id);
        let response = self.get(&endpoint).await?;
        self.handle_response(response).await
    }

    /// Get a specific channel member
    ///
    /// # Arguments
    /// * `channel_id` - The ID of the channel
    /// * `user_id` - The ID of the user
    ///
    /// # Returns
    /// A Result containing the channel member information or an Error
    pub async fn get_channel_member(&self, channel_id: &str, user_id: &str) -> Result<ChannelMember> {
        let endpoint = format!("/channels/{}/members/{}", channel_id, user_id);
        let response = self.get(&endpoint).await?;
        self.handle_response(response).await
    }

    /// Add a user to a channel
    ///
    /// # Arguments
    /// * `channel_id` - The ID of the channel
    /// * `user_id` - The ID of the user to add
    ///
    /// # Returns
    /// A Result containing the channel member information or an Error
    pub async fn add_channel_member(&self, channel_id: &str, user_id: &str) -> Result<ChannelMember> {
        let body = serde_json::json!({
            "user_id": user_id,
        });

        let endpoint = format!("/channels/{}/members", channel_id);
        let response = self.post(&endpoint, &body).await?;
        self.handle_response(response).await
    }

    /// Remove a user from a channel
    ///
    /// # Arguments
    /// * `channel_id` - The ID of the channel
    /// * `user_id` - The ID of the user to remove
    ///
    /// # Returns
    /// A Result indicating success or failure
    pub async fn remove_channel_member(&self, channel_id: &str, user_id: &str) -> Result<()> {
        let endpoint = format!("/channels/{}/members/{}", channel_id, user_id);
        let response = self.delete(&endpoint).await?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(crate::error::Error::new(
                crate::error::ErrorCode::NetworkError,
                &format!("Failed to remove channel member: {}", response.status()),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_endpoints() {
        let client = MattermostClient::new("https://mattermost.example.com").unwrap();

        // Test endpoint construction
        assert_eq!(
            client.api_url("/users/me/teams/team123/channels"),
            "https://mattermost.example.com/api/v4/users/me/teams/team123/channels"
        );
        assert_eq!(
            client.api_url("/channels/channel123"),
            "https://mattermost.example.com/api/v4/channels/channel123"
        );
    }

    #[test]
    fn test_parse_dm_channel_id() {
        // Valid DM channel ID
        let dm_id = "t1pn9rb63fnpjrqibgriijcx4r__xei6dqz8xfgm7kqzddjziyofyo";
        let result = parse_dm_channel_id(dm_id);
        assert!(result.is_some());
        let (user1, user2) = result.unwrap();
        assert_eq!(user1, "t1pn9rb63fnpjrqibgriijcx4r");
        assert_eq!(user2, "xei6dqz8xfgm7kqzddjziyofyo");

        // Regular channel ID (not a DM)
        let regular_id = "channel123abc";
        assert!(parse_dm_channel_id(regular_id).is_none());

        // Invalid format (too many parts)
        let invalid_id = "user1__user2__user3";
        assert!(parse_dm_channel_id(invalid_id).is_none());
    }

    #[test]
    fn test_get_dm_partner_id() {
        let dm_id = "abc123__xyz789";

        // Current user is first in the ID
        let partner = get_dm_partner_id(dm_id, "abc123");
        assert_eq!(partner, Some("xyz789".to_string()));

        // Current user is second in the ID
        let partner = get_dm_partner_id(dm_id, "xyz789");
        assert_eq!(partner, Some("abc123".to_string()));

        // Current user is not in the channel
        let partner = get_dm_partner_id(dm_id, "other_user");
        assert!(partner.is_none());

        // Regular channel ID
        let partner = get_dm_partner_id("regular_channel", "abc123");
        assert!(partner.is_none());
    }

    #[test]
    fn test_detect_channel_type_from_id() {
        // Direct message (2 users)
        let dm_id = "user1__user2";
        assert_eq!(
            detect_channel_type_from_id(dm_id),
            Some(ChannelType::DirectMessage)
        );

        // Group message (3+ users)
        let gm_id = "user1__user2__user3";
        assert_eq!(
            detect_channel_type_from_id(gm_id),
            Some(ChannelType::GroupMessage)
        );

        // Regular channel
        let regular_id = "townSquare123";
        assert!(detect_channel_type_from_id(regular_id).is_none());
    }
}
