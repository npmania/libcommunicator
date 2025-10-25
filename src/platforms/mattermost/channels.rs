use crate::error::Result;

use super::client::MattermostClient;
use super::types::{ChannelMember, CreateDirectChannelRequest, CreateGroupChannelRequest, MattermostChannel};

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
}
