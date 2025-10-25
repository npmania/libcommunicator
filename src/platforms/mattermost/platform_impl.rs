use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::error::{Error, ErrorCode, Result};
use crate::platforms::platform_trait::{Platform, PlatformConfig, PlatformEvent};
use crate::types::{Channel, ConnectionInfo, Message, User};

use super::client::MattermostClient;
use super::websocket::WebSocketManager;

/// Wrapper struct that implements the Platform trait for Mattermost
pub struct MattermostPlatform {
    client: MattermostClient,
    connection_info: Option<ConnectionInfo>,
    websocket: Arc<Mutex<Option<WebSocketManager>>>,
    server_url: String,
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
        })
    }

    /// Get the underlying client (for accessing Mattermost-specific methods)
    pub fn client(&self) -> &MattermostClient {
        &self.client
    }
}

#[async_trait]
impl Platform for MattermostPlatform {
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
        Ok(mm_channels.into_iter().map(|ch| ch.into()).collect())
    }

    async fn get_channel(&self, channel_id: &str) -> Result<Channel> {
        let mm_channel = self.client.get_channel(channel_id).await?;
        Ok(mm_channel.into())
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
            match self.client.get_user(&member.user_id).await {
                Ok(mm_user) => users.push(mm_user.into()),
                Err(e) => {
                    eprintln!("Failed to fetch user {}: {}", member.user_id, e);
                    // Continue with other users even if one fails
                }
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
        Ok(mm_channel.into())
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

        let mut ws_manager = WebSocketManager::new(&server_url, token);
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
