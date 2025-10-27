use crate::error::Result;

use super::client::MattermostClient;
use super::types::{
    ChannelMember, ChannelUnreadInfo, ChannelViewRequest, ChannelViewResponse,
    CreateDirectChannelRequest, CreateGroupChannelRequest, MattermostChannel, PostList, TeamUnread,
};

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

impl MattermostClient {
    /// Get all channels for the current user in a specific team
    ///
    /// # Arguments
    /// * `team_id` - The ID of the team to get channels from
    ///
    /// # Returns
    /// A Result containing a list of channels or an Error
    pub async fn get_channels_for_team(&self, team_id: &str) -> Result<Vec<MattermostChannel>> {
        let endpoint = format!("/users/me/teams/{team_id}/channels");
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
        let endpoint = format!("/channels/{channel_id}");
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
    pub async fn get_channel_by_name(
        &self,
        team_id: &str,
        channel_name: &str,
    ) -> Result<MattermostChannel> {
        let endpoint = format!("/teams/{team_id}/channels/name/{channel_name}");
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
        let endpoint = format!("/channels/{channel_id}/members");
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
    pub async fn get_channel_member(
        &self,
        channel_id: &str,
        user_id: &str,
    ) -> Result<ChannelMember> {
        let endpoint = format!("/channels/{channel_id}/members/{user_id}");
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
    pub async fn add_channel_member(
        &self,
        channel_id: &str,
        user_id: &str,
    ) -> Result<ChannelMember> {
        let body = serde_json::json!({
            "user_id": user_id,
        });

        let endpoint = format!("/channels/{channel_id}/members");
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
        let endpoint = format!("/channels/{channel_id}/members/{user_id}");
        let response = self.delete(&endpoint).await?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(crate::error::Error::new(
                crate::error::ErrorCode::NetworkError,
                format!("Failed to remove channel member: {}", response.status()),
            ))
        }
    }

    // ========================================================================
    // Channel Read State Management
    // ========================================================================

    /// Mark a channel as viewed (read) by the current user
    ///
    /// This updates the last_viewed_at timestamp for the channel and clears
    /// unread counts for the user.
    ///
    /// # Arguments
    /// * `channel_id` - The ID of the channel to mark as viewed
    /// * `prev_channel_id` - Optional ID of the previous channel (for tracking channel switches)
    ///
    /// # Returns
    /// A Result containing the view response with updated timestamps or an Error
    pub async fn view_channel(
        &self,
        channel_id: &str,
        prev_channel_id: Option<String>,
    ) -> Result<ChannelViewResponse> {
        let user_id = self.get_user_id().await.ok_or_else(|| {
            crate::error::Error::new(
                crate::error::ErrorCode::InvalidState,
                "User ID not set - ensure you're authenticated",
            )
        })?;

        let mut request = ChannelViewRequest::new(channel_id.to_string());
        if let Some(prev) = prev_channel_id {
            request = request.with_prev_channel(prev);
        }

        let endpoint = format!("/channels/members/{user_id}/view");
        let response = self.post(&endpoint, &request).await?;
        self.handle_response(response).await
    }

    /// Get unread message information for a specific channel
    ///
    /// Returns the number of unread messages and mentions for the current user
    /// in the specified channel.
    ///
    /// # Arguments
    /// * `channel_id` - The ID of the channel to get unread info for
    ///
    /// # Returns
    /// A Result containing unread information (msg_count, mention_count, last_viewed_at) or an Error
    pub async fn get_channel_unread(&self, channel_id: &str) -> Result<ChannelUnreadInfo> {
        let user_id = self.get_user_id().await.ok_or_else(|| {
            crate::error::Error::new(
                crate::error::ErrorCode::InvalidState,
                "User ID not set - ensure you're authenticated",
            )
        })?;

        let endpoint = format!("/users/{user_id}/channels/{channel_id}/unread");
        let response = self.get(&endpoint).await?;
        self.handle_response(response).await
    }

    /// Get unread counts for all channels in a specific team
    ///
    /// Returns unread message and mention counts for each channel the current
    /// user is a member of in the specified team.
    ///
    /// # Arguments
    /// * `team_id` - The ID of the team to get unread counts for
    ///
    /// # Returns
    /// A Result containing a list of channel unread information or an Error
    pub async fn get_team_unreads(&self, team_id: &str) -> Result<Vec<ChannelUnreadInfo>> {
        let user_id = self.get_user_id().await.ok_or_else(|| {
            crate::error::Error::new(
                crate::error::ErrorCode::InvalidState,
                "User ID not set - ensure you're authenticated",
            )
        })?;

        let endpoint = format!("/users/{user_id}/teams/{team_id}/channels/members");
        let response = self.get(&endpoint).await?;

        // The API returns ChannelMember objects, which we need to convert to ChannelUnreadInfo
        let members: Vec<ChannelMember> = self.handle_response(response).await?;

        Ok(members
            .into_iter()
            .map(|m| ChannelUnreadInfo {
                team_id: team_id.to_string(),
                channel_id: m.channel_id,
                msg_count: m.msg_count,
                mention_count: m.mention_count,
                last_viewed_at: m.last_viewed_at,
            })
            .collect())
    }

    /// Get unread counts across all teams
    ///
    /// Returns a summary of unread message and mention counts for each team
    /// the current user is a member of.
    ///
    /// # Arguments
    /// None (uses the current authenticated user)
    ///
    /// # Returns
    /// A Result containing a list of team unread summaries or an Error
    pub async fn get_all_unreads(&self) -> Result<Vec<TeamUnread>> {
        let user_id = self.get_user_id().await.ok_or_else(|| {
            crate::error::Error::new(
                crate::error::ErrorCode::InvalidState,
                "User ID not set - ensure you're authenticated",
            )
        })?;

        let endpoint = format!("/users/{user_id}/teams/unread");
        let response = self.get(&endpoint).await?;
        self.handle_response(response).await
    }

    /// Get unread posts for a specific channel for the current user
    ///
    /// Returns posts that are unread by the user, starting from their last viewed position.
    ///
    /// # Arguments
    /// * `channel_id` - The ID of the channel to get unread posts for
    /// * `limit_after` - Optional limit on the number of posts to retrieve after last viewed (default: 60)
    ///
    /// # Returns
    /// A Result containing a PostList with unread posts or an Error
    pub async fn get_unread_posts(
        &self,
        channel_id: &str,
        limit_after: Option<i32>,
    ) -> Result<PostList> {
        let user_id = self.get_user_id().await.ok_or_else(|| {
            crate::error::Error::new(
                crate::error::ErrorCode::InvalidState,
                "User ID not set - ensure you're authenticated",
            )
        })?;

        let mut endpoint = format!("/users/{user_id}/channels/{channel_id}/posts/unread");
        if let Some(limit) = limit_after {
            endpoint.push_str(&format!("?limit_after={limit}"));
        }

        let response = self.get(&endpoint).await?;
        self.handle_response(response).await
    }

    /// Get posts around the first unread message in a channel
    ///
    /// Returns posts before and after the first unread message, useful for
    /// implementing "jump to first unread" functionality.
    ///
    /// # Arguments
    /// * `channel_id` - The ID of the channel
    /// * `limit_before` - Optional number of posts to retrieve before the unread marker (default: 60)
    /// * `limit_after` - Optional number of posts to retrieve after the unread marker (default: 60)
    ///
    /// # Returns
    /// A Result containing a PostList with posts around the first unread or an Error
    pub async fn get_posts_around_unread(
        &self,
        channel_id: &str,
        limit_before: Option<i32>,
        limit_after: Option<i32>,
    ) -> Result<PostList> {
        let mut params = Vec::new();

        if let Some(before) = limit_before {
            params.push(format!("limit_before={before}"));
        }
        if let Some(after) = limit_after {
            params.push(format!("limit_after={after}"));
        }

        let query_string = if params.is_empty() {
            String::new()
        } else {
            format!("?{}", params.join("&"))
        };

        let endpoint = format!("/channels/{channel_id}/posts/unread{query_string}");
        let response = self.get(&endpoint).await?;
        self.handle_response(response).await
    }

    // ========================================================================
    // Channel CRUD Operations
    // ========================================================================

    /// Create a new channel (public or private)
    ///
    /// # Arguments
    /// * `team_id` - The ID of the team to create the channel in
    /// * `name` - The channel name (lowercase, no spaces, URL-friendly)
    /// * `display_name` - The display name shown in the UI
    /// * `is_private` - Whether to create a private channel (true) or public channel (false)
    ///
    /// # Returns
    /// A Result containing the created channel or an Error
    pub async fn create_channel(
        &self,
        team_id: &str,
        name: &str,
        display_name: &str,
        is_private: bool,
    ) -> Result<MattermostChannel> {
        let channel_type = if is_private { "P" } else { "O" };

        let body = serde_json::json!({
            "team_id": team_id,
            "name": name,
            "display_name": display_name,
            "type": channel_type,
        });

        let response = self.post("/channels", &body).await?;
        self.handle_response(response).await
    }

    /// Update a channel's properties
    ///
    /// # Arguments
    /// * `channel_id` - The ID of the channel to update
    /// * `display_name` - Optional new display name (pass None to keep unchanged)
    /// * `purpose` - Optional new purpose (pass None to keep unchanged)
    /// * `header` - Optional new header (pass None to keep unchanged)
    ///
    /// # Returns
    /// A Result containing the updated channel or an Error
    pub async fn update_channel(
        &self,
        channel_id: &str,
        display_name: Option<&str>,
        purpose: Option<&str>,
        header: Option<&str>,
    ) -> Result<MattermostChannel> {
        // First, get the current channel to build the update request
        let mut channel = self.get_channel(channel_id).await?;

        // Update only the fields that were provided
        if let Some(name) = display_name {
            channel.display_name = name.to_string();
        }
        if let Some(p) = purpose {
            channel.purpose = p.to_string();
        }
        if let Some(h) = header {
            channel.header = h.to_string();
        }

        let endpoint = format!("/channels/{channel_id}");
        let response = self.put(&endpoint, &channel).await?;
        self.handle_response(response).await
    }

    /// Delete (archive) a channel
    ///
    /// # Arguments
    /// * `channel_id` - The ID of the channel to delete
    ///
    /// # Returns
    /// A Result indicating success or failure
    pub async fn delete_channel(&self, channel_id: &str) -> Result<()> {
        let endpoint = format!("/channels/{channel_id}");
        let response = self.delete(&endpoint).await?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(crate::error::Error::new(
                crate::error::ErrorCode::NetworkError,
                format!("Failed to delete channel: {}", response.status()),
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
    fn test_self_dm_detection() {
        // Self-DM (both IDs are the same)
        let self_dm = "t1pn9rb63fnpjrqibgriijcx4r__t1pn9rb63fnpjrqibgriijcx4r";
        let user_id = "t1pn9rb63fnpjrqibgriijcx4r";

        // When you query your own ID, you get yourself back
        let partner = get_dm_partner_id(self_dm, user_id);
        assert_eq!(partner, Some(user_id.to_string()));
    }

    #[test]
    fn test_real_dm_channel_names() {
        // Test with actual Mattermost DM channel name formats

        // Self-DM
        let self_dm_name = "t1pn9rb63fnpjrqibgriijcx4r__t1pn9rb63fnpjrqibgriijcx4r";
        let user_id = "t1pn9rb63fnpjrqibgriijcx4r";
        assert_eq!(
            get_dm_partner_id(self_dm_name, user_id),
            Some(user_id.to_string())
        );

        // Regular DM
        let dm_name = "t1pn9rb63fnpjrqibgriijcx4r__xei6dqz8xfgm7kqzddjziyofyo";
        assert_eq!(
            get_dm_partner_id(dm_name, "t1pn9rb63fnpjrqibgriijcx4r"),
            Some("xei6dqz8xfgm7kqzddjziyofyo".to_string())
        );
        assert_eq!(
            get_dm_partner_id(dm_name, "xei6dqz8xfgm7kqzddjziyofyo"),
            Some("t1pn9rb63fnpjrqibgriijcx4r".to_string())
        );
    }
}
