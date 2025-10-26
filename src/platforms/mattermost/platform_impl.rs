use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::error::{Error, ErrorCode, Result};
use crate::platforms::platform_trait::{Platform, PlatformConfig, PlatformEvent};
use crate::types::{Channel, ConnectionInfo, Message, PlatformCapabilities, Team, User};

use super::client::MattermostClient;
use super::websocket::WebSocketManager;

/// Wrapper struct that implements the Platform trait for Mattermost
pub struct MattermostPlatform {
    client: MattermostClient,
    connection_info: Option<ConnectionInfo>,
    websocket: Arc<Mutex<Option<WebSocketManager>>>,
    server_url: String,
    capabilities: PlatformCapabilities,
}

impl MattermostPlatform {
    /// Create a new Mattermost platform instance
    pub fn new(server_url: &str) -> Result<Self> {
        let client = MattermostClient::new(server_url)?;
        Ok(Self {
            client,
            connection_info: None,
            websocket: Arc::new(Mutex::new(None)),
            server_url: server_url.to_string(),
            capabilities: PlatformCapabilities::mattermost(),
        })
    }

    /// Get the underlying client (for accessing Mattermost-specific methods)
    pub fn client(&self) -> &MattermostClient {
        &self.client
    }

    /// Convert a Mattermost channel to our Channel type with proper DM/GM handling
    async fn convert_channel_with_context(
        &self,
        mm_channel: super::types::MattermostChannel,
        current_user_id: Option<&str>,
    ) -> Result<Channel> {
        use super::channels::get_dm_partner_id;
        use super::convert::ConversionContext;

        // Create conversion context with server URL and current user
        let mut ctx = ConversionContext::new(self.server_url.clone());
        if let Some(user_id) = current_user_id {
            ctx = ctx.with_current_user(user_id.to_string());
        }

        // Convert the channel with context
        let mut channel = mm_channel.to_channel_with_context(&ctx);

        // For DM channels, fetch the other user's name and use it as display name
        // Note: DM channel "name" field contains user IDs in format "user1id__user2id"
        if mm_channel.channel_type.is_direct() {
            if let Some(user_id) = current_user_id {
                // Check if this is a self-DM (saved messages) - both user IDs are the same
                if mm_channel.name == format!("{user_id}__{user_id}") {
                    // This is a DM with yourself
                    channel.display_name = "You (Saved Messages)".to_string();
                } else if let Some(partner_id) = get_dm_partner_id(&mm_channel.name, user_id) {
                    // Regular DM with another user - use the "name" field which contains user IDs
                    match self.client.get_user(&partner_id).await {
                        Ok(partner_user) => {
                            // Build display name from partner's information
                            let display_name = if !partner_user.first_name.is_empty() || !partner_user.last_name.is_empty() {
                                format!("{} {}", partner_user.first_name, partner_user.last_name).trim().to_string()
                            } else if !partner_user.nickname.is_empty() {
                                partner_user.nickname.clone()
                            } else {
                                partner_user.username.clone()
                            };
                            channel.display_name = display_name;
                        }
                        Err(_) => {
                            // Fall back to a generic name
                            channel.display_name = "Direct Message".to_string();
                        }
                    }
                }
            }
        }
        // For group channels, we could fetch all participants and build a name
        // For now, we'll use the existing display_name from the API
        else if mm_channel.channel_type.is_group() && (mm_channel.display_name.is_empty() || current_user_id.is_some()) {
            // Group channels may need similar treatment
            // This could be enhanced in the future to fetch all member names
            if channel.display_name.is_empty() {
                channel.display_name = "Group Message".to_string();
            }
        }

        Ok(channel)
    }
}

#[async_trait]
impl Platform for MattermostPlatform {
    fn capabilities(&self) -> &PlatformCapabilities {
        &self.capabilities
    }

    async fn connect(&mut self, config: PlatformConfig) -> Result<ConnectionInfo> {
        // Determine authentication method from credentials
        if let Some(token) = config.credentials.get("token") {
            // Use Personal Access Token or existing session token
            self.client.login_with_token(token).await?;
        } else if let (Some(login_id), Some(password)) = (
            config.credentials.get("login_id"),
            config.credentials.get("password"),
        ) {
            // Use email/username and password
            self.client.login(login_id, password).await?;
        } else {
            return Err(Error::new(
                ErrorCode::InvalidArgument,
                "Missing authentication credentials (provide 'token' or 'login_id'+'password')",
            ));
        }

        // Set team ID if provided
        if let Some(team_id) = config.team_id {
            self.client.set_team_id(Some(team_id)).await;
        }

        // Get the current user to build connection info
        let current_user = self.client.get_current_user().await?;

        // Get connection info
        let conn_info = self.client.connection_info(
            &self.server_url,
            &current_user.username
        ).await;
        self.connection_info = Some(conn_info.clone());

        Ok(conn_info)
    }

    async fn disconnect(&mut self) -> Result<()> {
        // Disconnect WebSocket if connected
        if let Some(ws) = self.websocket.lock().await.as_mut() {
            ws.disconnect().await;
        }

        // Logout from Mattermost
        self.client.logout().await?;

        self.connection_info = None;
        Ok(())
    }

    fn connection_info(&self) -> Option<&ConnectionInfo> {
        self.connection_info.as_ref()
    }

    async fn send_message(&self, channel_id: &str, text: &str) -> Result<Message> {
        let mm_post = self.client.send_message(channel_id, text).await?;
        Ok(mm_post.into())
    }

    async fn get_channels(&self) -> Result<Vec<Channel>> {
        // Get team ID from connection info or client state
        let team_id = self.client.get_team_id().await.ok_or_else(|| {
            Error::new(
                ErrorCode::InvalidState,
                "Team ID not set - call connect() with a team_id or set it manually",
            )
        })?;

        let mm_channels = self.client.get_channels_for_team(&team_id).await?;

        // Get current user ID for DM channel context
        let current_user_id = self.client.get_user_id().await;

        // Convert channels with proper DM handling
        let mut channels = Vec::new();
        for mm_channel in mm_channels {
            let channel = self.convert_channel_with_context(mm_channel, current_user_id.as_deref()).await?;
            channels.push(channel);
        }

        Ok(channels)
    }

    async fn get_channel(&self, channel_id: &str) -> Result<Channel> {
        let mm_channel = self.client.get_channel(channel_id).await?;
        let current_user_id = self.client.get_user_id().await;
        self.convert_channel_with_context(mm_channel, current_user_id.as_deref()).await
    }

    async fn get_messages(&self, channel_id: &str, limit: usize) -> Result<Vec<Message>> {
        let post_list = self.client.get_latest_posts(channel_id, limit as u32).await?;

        // Convert posts to messages in the correct order
        let mut messages: Vec<Message> = post_list
            .order
            .iter()
            .filter_map(|post_id| post_list.posts.get(post_id))
            .map(|post| post.clone().into())
            .collect();

        // Reverse to get most recent first
        messages.reverse();

        Ok(messages)
    }

    async fn get_channel_members(&self, channel_id: &str) -> Result<Vec<User>> {
        let mm_members = self.client.get_channel_members(channel_id).await?;

        // Fetch user details for each member
        let mut users = Vec::new();
        for member in mm_members {
            if let Ok(mm_user) = self.client.get_user(&member.user_id).await {
                users.push(mm_user.into());
                // Continue with other users even if one fails
            }
        }

        Ok(users)
    }

    async fn get_user(&self, user_id: &str) -> Result<User> {
        let mm_user = self.client.get_user(user_id).await?;
        Ok(mm_user.into())
    }

    async fn get_current_user(&self) -> Result<User> {
        let mm_user = self.client.get_current_user().await?;
        Ok(mm_user.into())
    }

    async fn create_direct_channel(&self, user_id: &str) -> Result<Channel> {
        let mm_channel = self.client.create_direct_channel(user_id).await?;
        let current_user_id = self.client.get_user_id().await;
        self.convert_channel_with_context(mm_channel, current_user_id.as_deref()).await
    }

    async fn get_teams(&self) -> Result<Vec<Team>> {
        let mm_teams = self.client.get_teams().await?;
        Ok(mm_teams.into_iter().map(|t| t.into()).collect())
    }

    async fn get_team(&self, team_id: &str) -> Result<Team> {
        let mm_team = self.client.get_team(team_id).await?;
        Ok(mm_team.into())
    }

    async fn set_status(&self, status: crate::types::user::UserStatus, custom_message: Option<&str>) -> Result<()> {
        let status_str = super::user_status_to_status_string(status);
        self.client.set_status(status_str).await?;

        // TODO: Mattermost supports custom status messages via a separate API endpoint
        // For now, we're ignoring the custom_message parameter
        // Future enhancement: call the custom status API if custom_message is provided
        let _ = custom_message;

        Ok(())
    }

    async fn get_user_status(&self, user_id: &str) -> Result<crate::types::user::UserStatus> {
        let mm_status = self.client.get_user_status(user_id).await?;
        Ok(super::status_string_to_user_status(&mm_status.status))
    }

    async fn send_typing_indicator(&self, channel_id: &str, parent_id: Option<&str>) -> Result<()> {
        let ws_lock = self.websocket.lock().await;
        if let Some(ws) = ws_lock.as_ref() {
            ws.send_typing_indicator(channel_id, parent_id).await
        } else {
            Err(Error::new(
                ErrorCode::InvalidState,
                "WebSocket not connected - cannot send typing indicator. Call subscribe_events() first.",
            ))
        }
    }

    async fn subscribe_events(&mut self) -> Result<()> {
        let token = self.client.get_token().await.ok_or_else(|| {
            Error::new(
                ErrorCode::InvalidState,
                "Not authenticated - cannot subscribe to events",
            )
        })?;

        // Use the stored server URL
        let server_url = &self.server_url;

        let mut ws_manager = WebSocketManager::new(server_url, token);
        ws_manager.connect().await?;

        let mut ws_lock = self.websocket.lock().await;
        *ws_lock = Some(ws_manager);

        Ok(())
    }

    async fn unsubscribe_events(&mut self) -> Result<()> {
        let mut ws_lock = self.websocket.lock().await;
        if let Some(ws) = ws_lock.as_mut() {
            ws.disconnect().await;
        }
        *ws_lock = None;
        Ok(())
    }

    async fn poll_event(&mut self) -> Result<Option<PlatformEvent>> {
        let ws_lock = self.websocket.lock().await;
        if let Some(ws) = ws_lock.as_ref() {
            // Poll from the WebSocket manager
            if let Some(event) = ws.poll_event().await {
                // Convert the internal event to PlatformEvent
                // Note: The websocket module already returns PlatformEvent
                return Ok(Some(event));
            }
        }
        Ok(None)
    }

    // ========================================================================
    // Extended Platform Methods Implementation
    // ========================================================================

    async fn send_reply(&self, channel_id: &str, text: &str, root_id: &str) -> Result<Message> {
        let mm_post = self.client.send_reply(channel_id, text, root_id).await?;
        Ok(mm_post.into())
    }

    async fn update_message(&self, message_id: &str, new_text: &str) -> Result<Message> {
        let mm_post = self.client.update_post(message_id, new_text).await?;
        Ok(mm_post.into())
    }

    async fn delete_message(&self, message_id: &str) -> Result<()> {
        self.client.delete_post(message_id).await
    }

    async fn get_message(&self, message_id: &str) -> Result<Message> {
        let mm_post = self.client.get_post(message_id).await?;
        Ok(mm_post.into())
    }

    async fn search_messages(&self, query: &str, limit: usize) -> Result<Vec<Message>> {
        // Get team ID from client state
        let team_id = self.client.get_team_id().await.ok_or_else(|| {
            Error::new(
                ErrorCode::InvalidState,
                "Team ID not set - call connect() with a team_id or set it manually",
            )
        })?;

        let post_list = self.client.search_posts(&team_id, query).await?;

        // Convert posts to messages, limited by the requested limit
        let mut messages: Vec<Message> = post_list
            .order
            .iter()
            .take(limit)
            .filter_map(|post_id| post_list.posts.get(post_id))
            .map(|post| post.clone().into())
            .collect();

        // Reverse to get most recent first
        messages.reverse();

        Ok(messages)
    }

    async fn get_messages_before(&self, channel_id: &str, before_id: &str, limit: usize) -> Result<Vec<Message>> {
        let post_list = self.client.get_posts_before(channel_id, before_id, limit as u32).await?;

        // Convert posts to messages in the correct order
        let mut messages: Vec<Message> = post_list
            .order
            .iter()
            .filter_map(|post_id| post_list.posts.get(post_id))
            .map(|post| post.clone().into())
            .collect();

        // Reverse to get most recent first
        messages.reverse();

        Ok(messages)
    }

    async fn get_messages_after(&self, channel_id: &str, after_id: &str, limit: usize) -> Result<Vec<Message>> {
        let post_list = self.client.get_posts_after(channel_id, after_id, limit as u32).await?;

        // Convert posts to messages in the correct order
        let mut messages: Vec<Message> = post_list
            .order
            .iter()
            .filter_map(|post_id| post_list.posts.get(post_id))
            .map(|post| post.clone().into())
            .collect();

        // Reverse to get most recent first
        messages.reverse();

        Ok(messages)
    }

    async fn add_reaction(&self, message_id: &str, emoji: &str) -> Result<()> {
        self.client.add_reaction(message_id, emoji).await?;
        Ok(())
    }

    async fn remove_reaction(&self, message_id: &str, emoji: &str) -> Result<()> {
        self.client.remove_reaction(message_id, emoji).await
    }

    async fn get_channel_by_name(&self, team_id: &str, channel_name: &str) -> Result<Channel> {
        let mm_channel = self.client.get_channel_by_name(team_id, channel_name).await?;
        let current_user_id = self.client.get_user_id().await;
        self.convert_channel_with_context(mm_channel, current_user_id.as_deref()).await
    }

    async fn create_group_channel(&self, user_ids: Vec<String>) -> Result<Channel> {
        let mm_channel = self.client.create_group_channel(user_ids).await?;
        let current_user_id = self.client.get_user_id().await;
        self.convert_channel_with_context(mm_channel, current_user_id.as_deref()).await
    }

    async fn add_channel_member(&self, channel_id: &str, user_id: &str) -> Result<()> {
        self.client.add_channel_member(channel_id, user_id).await?;
        Ok(())
    }

    async fn remove_channel_member(&self, channel_id: &str, user_id: &str) -> Result<()> {
        self.client.remove_channel_member(channel_id, user_id).await
    }

    async fn get_user_by_username(&self, username: &str) -> Result<User> {
        let mm_user = self.client.get_user_by_username(username).await?;
        Ok(mm_user.into())
    }

    async fn get_user_by_email(&self, email: &str) -> Result<User> {
        let mm_user = self.client.get_user_by_email(email).await?;
        Ok(mm_user.into())
    }

    async fn get_users_by_ids(&self, user_ids: Vec<String>) -> Result<Vec<User>> {
        let mm_users = self.client.get_users_by_ids(&user_ids).await?;
        Ok(mm_users.into_iter().map(|u| u.into()).collect())
    }

    async fn set_custom_status(&self, emoji: Option<&str>, text: &str, expires_at: Option<i64>) -> Result<()> {
        use super::types::CustomStatus;

        // Convert Unix timestamp (i64) to ISO 8601 string if provided
        let expires_at_str = expires_at.map(|ts| {
            // Convert Unix timestamp to ISO 8601 format
            // For simplicity, using a basic conversion
            use chrono::{DateTime, Utc};
            let datetime = DateTime::<Utc>::from_timestamp(ts, 0)
                .unwrap_or_else(Utc::now);
            datetime.to_rfc3339()
        });

        let custom_status = CustomStatus {
            emoji: emoji.map(|s| s.to_string()),
            text: Some(text.to_string()),
            duration: None,
            expires_at: expires_at_str,
        };

        self.client.set_custom_status(custom_status).await
    }

    async fn remove_custom_status(&self) -> Result<()> {
        self.client.remove_custom_status().await
    }

    async fn get_users_status(&self, user_ids: Vec<String>) -> Result<std::collections::HashMap<String, crate::types::user::UserStatus>> {
        let mm_statuses = self.client.get_users_status_by_ids(&user_ids).await?;

        let mut status_map = std::collections::HashMap::new();
        for status in mm_statuses {
            let user_status = super::status_string_to_user_status(&status.status);
            status_map.insert(status.user_id, user_status);
        }

        Ok(status_map)
    }

    async fn get_team_by_name(&self, team_name: &str) -> Result<Team> {
        let mm_team = self.client.get_team_by_name(team_name).await?;
        Ok(mm_team.into())
    }

    async fn set_team_id(&self, team_id: Option<String>) -> Result<()> {
        self.client.set_team_id(team_id).await;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_platform_creation() {
        let platform = MattermostPlatform::new("https://mattermost.example.com");
        assert!(platform.is_ok());
    }

    #[tokio::test]
    async fn test_invalid_url() {
        let platform = MattermostPlatform::new("not a url");
        assert!(platform.is_err());
    }

    #[test]
    fn test_platform_config() {
        let config = PlatformConfig::new("https://mattermost.example.com")
            .with_credential("login_id", "user@example.com")
            .with_credential("password", "password123")
            .with_team("team-abc");

        assert_eq!(config.server, "https://mattermost.example.com");
        assert!(config.credentials.contains_key("login_id"));
        assert_eq!(config.team_id, Some("team-abc".to_string()));
    }
}
