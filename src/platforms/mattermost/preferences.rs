use crate::error::Result;

use super::client::MattermostClient;
use super::types::{ChannelNotifyProps, DeletePreferencesRequest, UserPreference};

impl MattermostClient {
    // ========================================================================
    // User Preferences
    // ========================================================================

    /// Get all preferences for a user
    ///
    /// # Arguments
    /// * `user_id` - The ID of the user to get preferences for
    ///
    /// # Returns
    /// A Result containing a list of user preferences or an Error
    ///
    /// # API Endpoint
    /// GET /users/{user_id}/preferences
    pub async fn get_user_preferences(&self, user_id: &str) -> Result<Vec<UserPreference>> {
        let endpoint = format!("/users/{user_id}/preferences");
        let response = self.get(&endpoint).await?;
        self.handle_response(response).await
    }

    /// Set/update user preferences
    ///
    /// # Arguments
    /// * `user_id` - The ID of the user to set preferences for
    /// * `preferences` - A list of preferences to set
    ///
    /// # Returns
    /// A Result indicating success or failure
    ///
    /// # API Endpoint
    /// PUT /users/{user_id}/preferences
    pub async fn set_user_preferences(
        &self,
        user_id: &str,
        preferences: &[UserPreference],
    ) -> Result<()> {
        let endpoint = format!("/users/{user_id}/preferences");
        let response = self.put(&endpoint, &preferences).await?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(crate::error::Error::new(
                crate::error::ErrorCode::NetworkError,
                format!("Failed to set user preferences: {}", response.status()),
            ))
        }
    }

    /// Delete user preferences
    ///
    /// # Arguments
    /// * `user_id` - The ID of the user to delete preferences for
    /// * `preferences` - A list of preferences to delete
    ///
    /// # Returns
    /// A Result indicating success or failure
    ///
    /// # API Endpoint
    /// POST /users/{user_id}/preferences/delete
    pub async fn delete_user_preferences(
        &self,
        user_id: &str,
        preferences: &[UserPreference],
    ) -> Result<()> {
        let request = DeletePreferencesRequest {
            preferences: preferences.to_vec(),
        };

        let endpoint = format!("/users/{user_id}/preferences/delete");
        let response = self.post(&endpoint, &request).await?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(crate::error::Error::new(
                crate::error::ErrorCode::NetworkError,
                format!("Failed to delete user preferences: {}", response.status()),
            ))
        }
    }

    /// Get a specific preference category for a user
    ///
    /// # Arguments
    /// * `user_id` - The ID of the user
    /// * `category` - The preference category to retrieve
    ///
    /// # Returns
    /// A Result containing a list of preferences in that category or an Error
    ///
    /// # API Endpoint
    /// GET /users/{user_id}/preferences/{category}
    pub async fn get_user_preferences_by_category(
        &self,
        user_id: &str,
        category: &str,
    ) -> Result<Vec<UserPreference>> {
        let endpoint = format!("/users/{user_id}/preferences/{category}");
        let response = self.get(&endpoint).await?;
        self.handle_response(response).await
    }

    /// Get a specific preference for a user
    ///
    /// # Arguments
    /// * `user_id` - The ID of the user
    /// * `category` - The preference category
    /// * `name` - The preference name
    ///
    /// # Returns
    /// A Result containing the specific preference or an Error
    ///
    /// # API Endpoint
    /// GET /users/{user_id}/preferences/{category}/name/{name}
    pub async fn get_user_preference(
        &self,
        user_id: &str,
        category: &str,
        name: &str,
    ) -> Result<UserPreference> {
        let endpoint = format!("/users/{user_id}/preferences/{category}/name/{name}");
        let response = self.get(&endpoint).await?;
        self.handle_response(response).await
    }

    // ========================================================================
    // Channel Notifications
    // ========================================================================
    //
    // Note: To get current notification properties for a channel member,
    // use the existing get_channel_member() function from channels.rs,
    // which returns a ChannelMember struct containing notify_props.

    /// Update notification properties for a channel
    ///
    /// # Arguments
    /// * `channel_id` - The ID of the channel
    /// * `user_id` - The ID of the user
    /// * `notify_props` - The notification properties to update
    ///
    /// # Returns
    /// A Result indicating success or failure
    ///
    /// # API Endpoint
    /// PUT /channels/{channel_id}/members/{user_id}/notify_props
    pub async fn update_channel_notify_props(
        &self,
        channel_id: &str,
        user_id: &str,
        notify_props: &ChannelNotifyProps,
    ) -> Result<()> {
        let endpoint = format!("/channels/{channel_id}/members/{user_id}/notify_props");
        let response = self.put(&endpoint, &notify_props).await?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(crate::error::Error::new(
                crate::error::ErrorCode::NetworkError,
                format!(
                    "Failed to update channel notify props: {}",
                    response.status()
                ),
            ))
        }
    }

    /// Mute a channel for a user
    ///
    /// This is a convenience method that sets the notification properties
    /// to mute all notifications for the specified channel.
    ///
    /// # Arguments
    /// * `channel_id` - The ID of the channel to mute
    /// * `user_id` - The ID of the user
    ///
    /// # Returns
    /// A Result indicating success or failure
    pub async fn mute_channel(&self, channel_id: &str, user_id: &str) -> Result<()> {
        let muted_props = ChannelNotifyProps::muted();
        self.update_channel_notify_props(channel_id, user_id, &muted_props)
            .await
    }

    /// Unmute a channel for a user
    ///
    /// This is a convenience method that restores the default notification
    /// properties for the specified channel.
    ///
    /// # Arguments
    /// * `channel_id` - The ID of the channel to unmute
    /// * `user_id` - The ID of the user
    ///
    /// # Returns
    /// A Result indicating success or failure
    pub async fn unmute_channel(&self, channel_id: &str, user_id: &str) -> Result<()> {
        let unmuted_props = ChannelNotifyProps::unmuted();
        self.update_channel_notify_props(channel_id, user_id, &unmuted_props)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preference_endpoints() {
        let client = MattermostClient::new("https://mattermost.example.com").unwrap();

        // Test preference endpoints
        assert_eq!(
            client.api_url("/users/user123/preferences"),
            "https://mattermost.example.com/api/v4/users/user123/preferences"
        );
        assert_eq!(
            client.api_url("/users/user123/preferences/delete"),
            "https://mattermost.example.com/api/v4/users/user123/preferences/delete"
        );
        assert_eq!(
            client.api_url("/users/user123/preferences/notifications"),
            "https://mattermost.example.com/api/v4/users/user123/preferences/notifications"
        );
        assert_eq!(
            client.api_url("/users/user123/preferences/notifications/name/email"),
            "https://mattermost.example.com/api/v4/users/user123/preferences/notifications/name/email"
        );
    }

    #[test]
    fn test_channel_notify_endpoints() {
        let client = MattermostClient::new("https://mattermost.example.com").unwrap();

        // Test channel member endpoints
        assert_eq!(
            client.api_url("/channels/channel123/members/user456"),
            "https://mattermost.example.com/api/v4/channels/channel123/members/user456"
        );
        assert_eq!(
            client.api_url("/channels/channel123/members/user456/notify_props"),
            "https://mattermost.example.com/api/v4/channels/channel123/members/user456/notify_props"
        );
    }
}
