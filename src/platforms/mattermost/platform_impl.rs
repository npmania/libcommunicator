use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::error::{Error, ErrorCode, Result};
use crate::platforms::platform_trait::{Platform, PlatformConfig, PlatformEvent};
use crate::types::{
    Attachment, Channel, ConnectionInfo, Message, PlatformCapabilities, Team, User,
};

use super::client::MattermostClient;
use super::convert::ConversionContext;
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
                            let display_name = if !partner_user.first_name.is_empty()
                                || !partner_user.last_name.is_empty()
                            {
                                format!("{} {}", partner_user.first_name, partner_user.last_name)
                                    .trim()
                                    .to_string()
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
        else if mm_channel.channel_type.is_group()
            && (mm_channel.display_name.is_empty() || current_user_id.is_some())
        {
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
            // Check if MFA token is provided
            if let Some(mfa_token) = config.credentials.get("mfa_token") {
                // Use email/username, password, and MFA token
                self.client
                    .login_with_mfa(login_id, password, mfa_token)
                    .await?;
            } else {
                // Use email/username and password
                self.client.login(login_id, password).await?;
            }
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
        let conn_info = self
            .client
            .connection_info(&self.server_url, &current_user.username)
            .await;
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
            let channel = self
                .convert_channel_with_context(mm_channel, current_user_id.as_deref())
                .await?;
            channels.push(channel);
        }

        Ok(channels)
    }

    async fn get_channel(&self, channel_id: &str) -> Result<Channel> {
        let mm_channel = self.client.get_channel_cached(channel_id).await?;
        let current_user_id = self.client.get_user_id().await;
        self.convert_channel_with_context(mm_channel, current_user_id.as_deref())
            .await
    }

    async fn get_messages(&self, channel_id: &str, limit: usize) -> Result<Vec<Message>> {
        let post_list = self
            .client
            .get_latest_posts(channel_id, limit as u32)
            .await?;

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

        // Collect all user IDs
        let user_ids: Vec<String> = mm_members.iter().map(|m| m.user_id.clone()).collect();

        // Use batch cached fetch - this is MUCH more efficient than N individual calls
        // If users are cached, this makes zero API calls
        // Otherwise, it makes one batch API call for all uncached users
        let mm_users = self.client.get_users_by_ids_cached(&user_ids).await?;

        // Convert to User type
        Ok(mm_users.into_iter().map(|u| u.into()).collect())
    }

    async fn get_user(&self, user_id: &str) -> Result<User> {
        let mm_user = self.client.get_user_cached(user_id).await?;
        Ok(mm_user.into())
    }

    async fn get_current_user(&self) -> Result<User> {
        let mm_user = self.client.get_current_user().await?;
        Ok(mm_user.into())
    }

    async fn create_direct_channel(&self, user_id: &str) -> Result<Channel> {
        let mm_channel = self.client.create_direct_channel(user_id).await?;
        let current_user_id = self.client.get_user_id().await;
        self.convert_channel_with_context(mm_channel, current_user_id.as_deref())
            .await
    }

    async fn create_channel(
        &self,
        team_id: &str,
        name: &str,
        display_name: &str,
        is_private: bool,
    ) -> Result<Channel> {
        let mm_channel = self
            .client
            .create_channel(team_id, name, display_name, is_private)
            .await?;
        let current_user_id = self.client.get_user_id().await;
        self.convert_channel_with_context(mm_channel, current_user_id.as_deref())
            .await
    }

    async fn update_channel(
        &self,
        channel_id: &str,
        display_name: Option<&str>,
        purpose: Option<&str>,
        header: Option<&str>,
    ) -> Result<Channel> {
        let mm_channel = self
            .client
            .update_channel(channel_id, display_name, purpose, header)
            .await?;
        let current_user_id = self.client.get_user_id().await;
        self.convert_channel_with_context(mm_channel, current_user_id.as_deref())
            .await
    }

    async fn delete_channel(&self, channel_id: &str) -> Result<()> {
        self.client.delete_channel(channel_id).await
    }

    async fn get_teams(&self) -> Result<Vec<Team>> {
        let mm_teams = self.client.get_teams().await?;
        Ok(mm_teams.into_iter().map(|t| t.into()).collect())
    }

    async fn get_team(&self, team_id: &str) -> Result<Team> {
        let mm_team = self.client.get_team_cached(team_id).await?;
        Ok(mm_team.into())
    }

    async fn set_status(
        &self,
        status: crate::types::user::UserStatus,
        custom_message: Option<&str>,
    ) -> Result<()> {
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
                // Invalidate caches based on event type
                match &event {
                    // User events - invalidate user cache
                    PlatformEvent::UserUpdated { user_id } => {
                        self.client.invalidate_user_cache(user_id).await;
                    }
                    PlatformEvent::UserRoleUpdated { user_id } => {
                        self.client.invalidate_user_cache(user_id).await;
                    }

                    // Channel events - invalidate channel cache
                    PlatformEvent::ChannelCreated(channel) => {
                        self.client.invalidate_channel_cache(&channel.id).await;
                    }
                    PlatformEvent::ChannelUpdated(channel) => {
                        self.client.invalidate_channel_cache(&channel.id).await;
                    }
                    PlatformEvent::ChannelDeleted { channel_id } => {
                        self.client.invalidate_channel_cache(channel_id).await;
                    }

                    // Team events - clear team cache (structural changes)
                    PlatformEvent::AddedToTeam { team_id, .. } => {
                        self.client.invalidate_team_cache(team_id).await;
                    }
                    PlatformEvent::LeftTeam { team_id, .. } => {
                        self.client.invalidate_team_cache(team_id).await;
                    }

                    // Other events don't require cache invalidation
                    _ => {}
                }

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
        let team_id = self
            .client
            .get_team_id()
            .await
            .ok_or_else(|| Error::new(ErrorCode::InvalidArgument, "Team ID not set"))?;

        // Use advanced search with pagination
        let options = crate::platforms::mattermost::PostSearchOptions {
            is_or_search: false,
            include_deleted_channels: false,
            time_zone_offset: 0,
            page: 0,
            per_page: limit as u32,
        };

        let post_list = self
            .client
            .search_posts_advanced(&team_id, query, options)
            .await?;

        // Convert posts to messages
        let mut messages: Vec<Message> = post_list
            .order
            .iter()
            .filter_map(|post_id| post_list.posts.get(post_id))
            .map(|post| post.clone().into())
            .collect();

        // Limit to requested number
        messages.truncate(limit);

        Ok(messages)
    }

    async fn get_messages_before(
        &self,
        channel_id: &str,
        before_id: &str,
        limit: usize,
    ) -> Result<Vec<Message>> {
        let post_list = self
            .client
            .get_posts_before(channel_id, before_id, limit as u32)
            .await?;

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

    async fn get_messages_after(
        &self,
        channel_id: &str,
        after_id: &str,
        limit: usize,
    ) -> Result<Vec<Message>> {
        let post_list = self
            .client
            .get_posts_after(channel_id, after_id, limit as u32)
            .await?;

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

    async fn pin_post(&self, message_id: &str) -> Result<()> {
        self.client.pin_post(message_id).await
    }

    async fn unpin_post(&self, message_id: &str) -> Result<()> {
        self.client.unpin_post(message_id).await
    }

    async fn get_pinned_posts(&self, channel_id: &str) -> Result<Vec<Message>> {
        let mm_posts = self.client.get_pinned_posts(channel_id).await?;
        let messages: Vec<Message> = mm_posts.into_iter().map(|p| p.into()).collect();
        Ok(messages)
    }

    async fn get_emojis(&self, page: u32, per_page: u32) -> Result<Vec<crate::types::Emoji>> {
        let mm_emojis = self.client.get_emojis(page, per_page, "name").await?;
        Ok(mm_emojis.into_iter().map(|e| e.into()).collect())
    }

    async fn get_channel_by_name(&self, team_id: &str, channel_name: &str) -> Result<Channel> {
        let mm_channel = self
            .client
            .get_channel_by_name(team_id, channel_name)
            .await?;
        let current_user_id = self.client.get_user_id().await;
        self.convert_channel_with_context(mm_channel, current_user_id.as_deref())
            .await
    }

    async fn create_group_channel(&self, user_ids: Vec<String>) -> Result<Channel> {
        let mm_channel = self.client.create_group_channel(user_ids).await?;
        let current_user_id = self.client.get_user_id().await;
        self.convert_channel_with_context(mm_channel, current_user_id.as_deref())
            .await
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

    async fn set_custom_status(
        &self,
        emoji: Option<&str>,
        text: &str,
        expires_at: Option<i64>,
    ) -> Result<()> {
        use super::types::CustomStatus;

        // Convert Unix timestamp (i64) to ISO 8601 string if provided
        let expires_at_str = expires_at.map(|ts| {
            // Convert Unix timestamp to ISO 8601 format
            // For simplicity, using a basic conversion
            use chrono::{DateTime, Utc};
            let datetime = DateTime::<Utc>::from_timestamp(ts, 0).unwrap_or_else(Utc::now);
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

    async fn get_users_status(
        &self,
        user_ids: Vec<String>,
    ) -> Result<std::collections::HashMap<String, crate::types::user::UserStatus>> {
        let mm_statuses = self.client.get_users_status_by_ids(&user_ids).await?;

        let mut status_map = std::collections::HashMap::new();
        for status in mm_statuses {
            let user_status = super::status_string_to_user_status(&status.status);
            status_map.insert(status.user_id, user_status);
        }

        Ok(status_map)
    }

    async fn request_all_statuses(&self) -> Result<i64> {
        let ws_lock = self.websocket.lock().await;
        if let Some(ws) = ws_lock.as_ref() {
            ws.get_statuses().await
        } else {
            Err(Error::new(
                ErrorCode::InvalidState,
                "WebSocket not connected. Call subscribe_events first.",
            ))
        }
    }

    async fn request_users_statuses(&self, user_ids: Vec<String>) -> Result<i64> {
        let ws_lock = self.websocket.lock().await;
        if let Some(ws) = ws_lock.as_ref() {
            ws.get_statuses_by_ids(user_ids).await
        } else {
            Err(Error::new(
                ErrorCode::InvalidState,
                "WebSocket not connected. Call subscribe_events first.",
            ))
        }
    }

    async fn get_team_by_name(&self, team_name: &str) -> Result<Team> {
        let mm_team = self.client.get_team_by_name(team_name).await?;
        Ok(mm_team.into())
    }

    async fn set_team_id(&self, team_id: Option<String>) -> Result<()> {
        self.client.set_team_id(team_id).await;
        Ok(())
    }

    // ========================================================================
    // File Operations
    // ========================================================================

    async fn upload_file(&self, channel_id: &str, file_path: &std::path::Path) -> Result<String> {
        let file_info = self.client.upload_file(channel_id, file_path, None).await?;
        Ok(file_info.id)
    }

    async fn download_file(&self, file_id: &str) -> Result<Vec<u8>> {
        self.client.download_file(file_id).await
    }

    async fn get_file_metadata(&self, file_id: &str) -> Result<Attachment> {
        let file_info = self.client.get_file_info(file_id).await?;
        // Convert FileInfo to Attachment using context
        let ctx = ConversionContext {
            server_url: self.client.get_base_url().to_string(),
            current_user_id: self.client.get_user_id().await,
        };
        Ok(file_info.to_attachment_with_context(&ctx))
    }

    async fn get_file_thumbnail(&self, file_id: &str) -> Result<Vec<u8>> {
        self.client.get_file_thumbnail(file_id).await
    }

    // ========================================================================
    // Thread Operations
    // ========================================================================

    async fn get_thread(&self, post_id: &str) -> Result<Vec<Message>> {
        let post_list = self.client.get_thread(post_id).await?;

        // Convert posts to messages
        let mut messages = Vec::new();
        for post_id in &post_list.order {
            if let Some(post) = post_list.posts.get(post_id) {
                messages.push(post.clone().into());
            }
        }

        Ok(messages)
    }

    async fn follow_thread(&self, thread_id: &str) -> Result<()> {
        let user_id = "me"; // Use "me" to refer to current user
        let team_id = self
            .client
            .get_team_id()
            .await
            .ok_or_else(|| Error::new(ErrorCode::InvalidArgument, "Team ID not set"))?;

        self.client
            .follow_thread(user_id, &team_id, thread_id)
            .await
    }

    async fn unfollow_thread(&self, thread_id: &str) -> Result<()> {
        let user_id = "me"; // Use "me" to refer to current user
        let team_id = self
            .client
            .get_team_id()
            .await
            .ok_or_else(|| Error::new(ErrorCode::InvalidArgument, "Team ID not set"))?;

        self.client
            .unfollow_thread(user_id, &team_id, thread_id)
            .await
    }

    async fn mark_thread_read(&self, thread_id: &str) -> Result<()> {
        let user_id = "me"; // Use "me" to refer to current user
        let team_id = self
            .client
            .get_team_id()
            .await
            .ok_or_else(|| Error::new(ErrorCode::InvalidArgument, "Team ID not set"))?;

        // Use current timestamp
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;

        self.client
            .mark_thread_as_read(user_id, &team_id, thread_id, timestamp)
            .await
    }

    async fn mark_thread_unread(&self, thread_id: &str, post_id: &str) -> Result<()> {
        let user_id = "me"; // Use "me" to refer to current user
        let team_id = self
            .client
            .get_team_id()
            .await
            .ok_or_else(|| Error::new(ErrorCode::InvalidArgument, "Team ID not set"))?;

        self.client
            .mark_thread_as_unread(user_id, &team_id, thread_id, post_id)
            .await
    }

    async fn search_users(&self, query: &str, limit: usize) -> Result<Vec<User>> {
        let team_id = self
            .client
            .get_team_id()
            .await
            .ok_or_else(|| Error::new(ErrorCode::InvalidArgument, "Team ID not set"))?;

        let request = crate::platforms::mattermost::UserSearchRequest::new(query.to_string())
            .with_team_id(team_id)
            .with_limit(limit as u32);

        let mm_users = self.client.search_users(&request).await?;
        Ok(mm_users.into_iter().map(|u| u.into()).collect())
    }

    async fn autocomplete_users(
        &self,
        channel_id: &str,
        query: &str,
        limit: usize,
    ) -> Result<Vec<User>> {
        let team_id = self
            .client
            .get_team_id()
            .await
            .ok_or_else(|| Error::new(ErrorCode::InvalidArgument, "Team ID not set"))?;

        let mm_users = self
            .client
            .autocomplete_users(&team_id, channel_id, query, Some(limit as u32))
            .await?;

        Ok(mm_users.into_iter().map(|u| u.into()).collect())
    }

    async fn search_channels(&self, query: &str, limit: usize) -> Result<Vec<Channel>> {
        let team_id = self
            .client
            .get_team_id()
            .await
            .ok_or_else(|| Error::new(ErrorCode::InvalidArgument, "Team ID not set"))?;

        let request = crate::platforms::mattermost::ChannelSearchRequest::new(query.to_string());

        let mm_channels = self.client.search_channels(&team_id, &request).await?;

        // Limit results
        let limited: Vec<_> = mm_channels.into_iter().take(limit).collect();

        // Convert channels with proper DM handling
        let current_user_id = self.client.get_user_id().await;
        let mut channels = Vec::new();
        for mm_channel in limited {
            let channel = self
                .convert_channel_with_context(mm_channel, current_user_id.as_deref())
                .await?;
            channels.push(channel);
        }

        Ok(channels)
    }

    async fn autocomplete_channels(&self, query: &str, limit: usize) -> Result<Vec<Channel>> {
        let team_id = self
            .client
            .get_team_id()
            .await
            .ok_or_else(|| Error::new(ErrorCode::InvalidArgument, "Team ID not set"))?;

        let mm_channels = self.client.autocomplete_channels(&team_id, query).await?;

        // Limit results
        let limited: Vec<_> = mm_channels.into_iter().take(limit).collect();

        // Convert channels with proper DM handling
        let current_user_id = self.client.get_user_id().await;
        let mut channels = Vec::new();
        for mm_channel in limited {
            let channel = self
                .convert_channel_with_context(mm_channel, current_user_id.as_deref())
                .await?;
            channels.push(channel);
        }

        Ok(channels)
    }

    // ========================================================================
    // User Preferences and Notifications
    // ========================================================================

    async fn get_user_preferences(&self, user_id: &str) -> Result<String> {
        let prefs = self.client.get_user_preferences(user_id).await?;
        serde_json::to_string(&prefs).map_err(|e| {
            Error::new(
                ErrorCode::Unknown,
                format!("Failed to serialize preferences: {e}"),
            )
        })
    }

    async fn set_user_preferences(&self, user_id: &str, preferences_json: &str) -> Result<()> {
        let prefs: Vec<super::types::UserPreference> = serde_json::from_str(preferences_json)
            .map_err(|e| {
                Error::new(
                    ErrorCode::InvalidArgument,
                    format!("Failed to parse preferences JSON: {e}"),
                )
            })?;

        self.client.set_user_preferences(user_id, &prefs).await
    }

    async fn mute_channel(&self, channel_id: &str) -> Result<()> {
        let user_id = self
            .client
            .get_user_id()
            .await
            .ok_or_else(|| Error::new(ErrorCode::InvalidState, "User not authenticated"))?;

        self.client.mute_channel(channel_id, &user_id).await
    }

    async fn unmute_channel(&self, channel_id: &str) -> Result<()> {
        let user_id = self
            .client
            .get_user_id()
            .await
            .ok_or_else(|| Error::new(ErrorCode::InvalidState, "User not authenticated"))?;

        self.client.unmute_channel(channel_id, &user_id).await
    }

    async fn update_channel_notify_props(
        &self,
        channel_id: &str,
        notify_props_json: &str,
    ) -> Result<()> {
        let user_id = self
            .client
            .get_user_id()
            .await
            .ok_or_else(|| Error::new(ErrorCode::InvalidState, "User not authenticated"))?;

        let props: super::types::ChannelNotifyProps = serde_json::from_str(notify_props_json)
            .map_err(|e| {
                Error::new(
                    ErrorCode::InvalidArgument,
                    format!("Failed to parse notify props JSON: {e}"),
                )
            })?;

        self.client
            .update_channel_notify_props(channel_id, &user_id, &props)
            .await
    }

    async fn view_channel(&self, channel_id: &str) -> Result<()> {
        self.client.view_channel(channel_id, None).await?;
        Ok(())
    }

    async fn get_channel_unread(&self, channel_id: &str) -> Result<crate::types::ChannelUnread> {
        let mm_unread = self.client.get_channel_unread(channel_id).await?;

        Ok(crate::types::ChannelUnread {
            channel_id: mm_unread.channel_id,
            team_id: Some(mm_unread.team_id),
            msg_count: mm_unread.msg_count,
            mention_count: mm_unread.mention_count,
            last_viewed_at: mm_unread.last_viewed_at,
        })
    }

    async fn get_team_unreads(&self, team_id: &str) -> Result<Vec<crate::types::ChannelUnread>> {
        let mm_unreads = self.client.get_team_unreads(team_id).await?;

        Ok(mm_unreads
            .into_iter()
            .map(|mm_unread| crate::types::ChannelUnread {
                channel_id: mm_unread.channel_id,
                team_id: Some(mm_unread.team_id),
                msg_count: mm_unread.msg_count,
                mention_count: mm_unread.mention_count,
                last_viewed_at: mm_unread.last_viewed_at,
            })
            .collect())
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
