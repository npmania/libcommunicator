use futures::{stream::{SplitSink, SplitStream}, SinkExt, StreamExt};
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};

use crate::error::{Error, ErrorCode, Result};
use crate::platforms::platform_trait::PlatformEvent;

use super::types::{MattermostChannel, MattermostPost, WebSocketAuthChallenge, WebSocketAuthData, WebSocketEvent};

/// Type alias for the WebSocket write half
type WsWriter = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>;
/// Type alias for the WebSocket read half
type WsReader = SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;

/// WebSocket connection state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    /// Not connected
    Disconnected,
    /// Attempting to connect
    Connecting,
    /// Successfully connected and authenticated
    Connected,
    /// Attempting to reconnect after disconnection
    Reconnecting,
    /// Shutting down gracefully
    ShuttingDown,
}

/// Configuration for WebSocket connection
#[derive(Debug, Clone)]
pub struct WebSocketConfig {
    /// Maximum number of events to queue (default: 1000)
    /// When full, oldest events are dropped
    pub max_queue_size: usize,
    /// Ping interval in seconds (default: 30)
    /// Sends ping to keep connection alive
    pub ping_interval_secs: u64,
}

impl Default for WebSocketConfig {
    fn default() -> Self {
        Self {
            max_queue_size: 1000,
            ping_interval_secs: 30,
        }
    }
}

/// WebSocket connection manager for Mattermost
pub struct WebSocketManager {
    /// URL for the WebSocket connection
    ws_url: String,
    /// Authentication token
    token: String,
    /// Configuration
    config: WebSocketConfig,
    /// Event sender (for internal use)
    event_tx: mpsc::Sender<PlatformEvent>,
    /// Event receiver for polling events
    event_rx: Arc<Mutex<mpsc::Receiver<PlatformEvent>>>,
    /// WebSocket write half for sending messages
    ws_writer: Arc<Mutex<Option<WsWriter>>>,
    /// Shutdown signal sender
    shutdown_tx: Option<mpsc::Sender<()>>,
    /// Sequence number for WebSocket messages
    seq_number: Arc<Mutex<i64>>,
    /// Last received sequence number for gap detection
    last_received_seq: Arc<Mutex<i64>>,
    /// Current connection state
    connection_state: Arc<Mutex<ConnectionState>>,
}

impl WebSocketManager {
    /// Create a new WebSocket manager with default configuration
    ///
    /// # Arguments
    /// * `base_url` - The base URL of the Mattermost server
    /// * `token` - Authentication token for WebSocket authentication
    pub fn new(base_url: &str, token: String) -> Self {
        Self::with_config(base_url, token, WebSocketConfig::default())
    }

    /// Create a new WebSocket manager with custom configuration
    ///
    /// # Arguments
    /// * `base_url` - The base URL of the Mattermost server
    /// * `token` - Authentication token for WebSocket authentication
    /// * `config` - WebSocket configuration
    pub fn with_config(base_url: &str, token: String, config: WebSocketConfig) -> Self {
        // Convert HTTP(S) URL to WebSocket URL
        let ws_url = base_url
            .replace("https://", "wss://")
            .replace("http://", "ws://");
        let ws_url = format!("{}/api/v4/websocket", ws_url);

        // Create bounded channel for events with configured size
        let (event_tx, event_rx) = mpsc::channel(config.max_queue_size);

        Self {
            ws_url,
            token,
            config,
            event_tx,
            event_rx: Arc::new(Mutex::new(event_rx)),
            ws_writer: Arc::new(Mutex::new(None)),
            shutdown_tx: None,
            seq_number: Arc::new(Mutex::new(1)),
            last_received_seq: Arc::new(Mutex::new(0)),
            connection_state: Arc::new(Mutex::new(ConnectionState::Disconnected)),
        }
    }

    /// Send typing indicator to a channel
    ///
    /// # Arguments
    /// * `channel_id` - The channel to send typing indicator to
    pub async fn send_typing_indicator(&self, channel_id: &str) -> Result<()> {
        let action = serde_json::json!({
            "action": "user_typing",
            "seq": self.next_seq().await,
            "data": {
                "channel_id": channel_id,
            }
        });

        self.send_ws_message(Message::Text(action.to_string())).await
    }

    /// Get the current connection state
    pub async fn get_connection_state(&self) -> ConnectionState {
        *self.connection_state.lock().await
    }

    /// Set the connection state
    async fn set_connection_state(&self, state: ConnectionState) {
        *self.connection_state.lock().await = state;
    }

    /// Send a WebSocket message
    ///
    /// # Arguments
    /// * `message` - The message to send
    ///
    /// # Returns
    /// Result indicating success or failure
    async fn send_ws_message(&self, message: Message) -> Result<()> {
        let mut writer = self.ws_writer.lock().await;
        if let Some(ws) = writer.as_mut() {
            ws.send(message)
                .await
                .map_err(|e| Error::new(ErrorCode::NetworkError, &format!("Failed to send WebSocket message: {}", e)))?;
            Ok(())
        } else {
            Err(Error::new(ErrorCode::InvalidState, "WebSocket not connected"))
        }
    }

    /// Get next sequence number for WebSocket messages
    async fn next_seq(&self) -> i64 {
        let mut seq_num = self.seq_number.lock().await;
        let current = *seq_num;
        *seq_num += 1;
        current
    }

    /// Connect to the Mattermost WebSocket and start receiving events
    ///
    /// # Returns
    /// A Result indicating success or failure
    pub async fn connect(&mut self) -> Result<()> {
        self.set_connection_state(ConnectionState::Connecting).await;

        let (ws_stream, _) = connect_async(&self.ws_url)
            .await
            .map_err(|e| {
                // Set state back to disconnected on failure
                let state = self.connection_state.clone();
                tokio::spawn(async move {
                    *state.lock().await = ConnectionState::Disconnected;
                });
                Error::new(ErrorCode::NetworkError, &format!("WebSocket connection failed: {}", e))
            })?;

        let (mut write, read) = ws_stream.split();

        // Send authentication challenge
        let seq = {
            let mut seq_num = self.seq_number.lock().await;
            let current = *seq_num;
            *seq_num += 1;
            current
        };

        let auth_challenge = WebSocketAuthChallenge {
            seq,
            action: "authentication_challenge".to_string(),
            data: WebSocketAuthData {
                token: self.token.clone(),
            },
        };

        let auth_msg = serde_json::to_string(&auth_challenge)
            .map_err(|e| Error::new(ErrorCode::Unknown, &format!("Failed to serialize auth: {}", e)))?;

        write
            .send(Message::Text(auth_msg))
            .await
            .map_err(|e| Error::new(ErrorCode::NetworkError, &format!("Failed to send auth: {}", e)))?;

        // Store the write half for bidirectional communication
        *self.ws_writer.lock().await = Some(write);

        // Mark as connected after successful authentication
        self.set_connection_state(ConnectionState::Connected).await;

        // Create shutdown channel
        let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);
        self.shutdown_tx = Some(shutdown_tx);

        // Clone references for the spawned task
        let event_tx = self.event_tx.clone();
        let connection_state = Arc::clone(&self.connection_state);
        let ws_writer = Arc::clone(&self.ws_writer);
        let last_received_seq = Arc::clone(&self.last_received_seq);
        let ping_interval = std::time::Duration::from_secs(self.config.ping_interval_secs);

        // Spawn a task to handle incoming messages
        tokio::spawn(async move {
            let mut read = read;  // Make read mutable for the task
            let mut ping_timer = tokio::time::interval(ping_interval);
            ping_timer.tick().await;  // Skip first immediate tick

            loop {
                tokio::select! {
                    // Handle incoming WebSocket messages
                    msg = read.next() => {
                        match msg {
                            Some(Ok(Message::Text(text))) => {
                                if let Err(e) = Self::handle_message(text, &event_tx, &last_received_seq).await {
                                    eprintln!("Error handling WebSocket message: {}", e);
                                }
                            }
                            Some(Ok(Message::Ping(data))) => {
                                // Respond to ping with pong
                                if let Some(writer) = ws_writer.lock().await.as_mut() {
                                    if let Err(e) = writer.send(Message::Pong(data)).await {
                                        eprintln!("Failed to send pong: {}", e);
                                        *connection_state.lock().await = ConnectionState::Disconnected;
                                        *ws_writer.lock().await = None;
                                        break;
                                    }
                                }
                            }
                            Some(Ok(Message::Pong(_))) => {
                                // Pong received - connection is alive
                            }
                            Some(Ok(Message::Close(_))) => {
                                println!("WebSocket closed by server");
                                *connection_state.lock().await = ConnectionState::Disconnected;
                                *ws_writer.lock().await = None;
                                break;
                            }
                            Some(Err(e)) => {
                                eprintln!("WebSocket error: {}", e);
                                *connection_state.lock().await = ConnectionState::Disconnected;
                                *ws_writer.lock().await = None;
                                break;
                            }
                            None => {
                                println!("WebSocket stream ended");
                                *connection_state.lock().await = ConnectionState::Disconnected;
                                *ws_writer.lock().await = None;
                                break;
                            }
                            _ => {}
                        }
                    }
                    // Send periodic ping to keep connection alive
                    _ = ping_timer.tick() => {
                        if let Some(writer) = ws_writer.lock().await.as_mut() {
                            if let Err(e) = writer.send(Message::Ping(vec![])).await {
                                eprintln!("Failed to send ping: {}", e);
                                *connection_state.lock().await = ConnectionState::Disconnected;
                                *ws_writer.lock().await = None;
                                break;
                            }
                        }
                    }
                    // Handle shutdown signal
                    _ = shutdown_rx.recv() => {
                        println!("WebSocket shutdown requested");
                        *connection_state.lock().await = ConnectionState::ShuttingDown;
                        *ws_writer.lock().await = None;
                        break;
                    }
                }
            }

            // Ensure we're marked as disconnected and writer is cleared when task exits
            *connection_state.lock().await = ConnectionState::Disconnected;
            *ws_writer.lock().await = None;
        });

        Ok(())
    }

    /// Handle an incoming WebSocket message
    async fn handle_message(
        text: String,
        event_tx: &mpsc::Sender<PlatformEvent>,
        last_received_seq: &Arc<Mutex<i64>>,
    ) -> Result<()> {
        let ws_event: WebSocketEvent = serde_json::from_str(&text)
            .map_err(|e| {
                // Log raw message snippet for debugging (first 200 chars)
                let snippet = &text[..text.len().min(200)];
                eprintln!("Failed to parse WebSocket event: {} | Raw: {}", e, snippet);
                Error::new(ErrorCode::Unknown, &format!("Failed to parse WebSocket event: {}", e))
            })?;

        // Check for sequence gaps
        if ws_event.seq > 0 {
            let mut last_seq = last_received_seq.lock().await;
            let expected_seq = *last_seq + 1;
            if *last_seq > 0 && ws_event.seq > expected_seq {
                eprintln!(
                    "WARNING: WebSocket sequence gap detected! Expected {}, got {}. {} events may have been missed.",
                    expected_seq,
                    ws_event.seq,
                    ws_event.seq - expected_seq
                );
            }
            *last_seq = ws_event.seq;
        }

        // Convert WebSocket event to PlatformEvent
        if let Some(platform_event) = Self::convert_event(ws_event) {
            // Try to send event to channel
            // If full, log warning and drop the event (non-blocking)
            match event_tx.try_send(platform_event) {
                Ok(_) => {} // Event sent successfully
                Err(mpsc::error::TrySendError::Full(event)) => {
                    eprintln!("WARNING: Event queue is full, dropping event: {:?}", event);
                }
                Err(mpsc::error::TrySendError::Closed(_)) => {
                    // Receiver dropped, silently ignore
                }
            }
        }

        Ok(())
    }

    /// Convert a Mattermost WebSocket event to a PlatformEvent
    fn convert_event(ws_event: WebSocketEvent) -> Option<PlatformEvent> {
        match ws_event.event.as_str() {
            "posted" => {
                // Extract and deserialize the post data from the event
                // Note: The "post" field is a JSON-encoded string, not a nested object
                if let Some(post_data) = ws_event.data.get("post") {
                    // Get the string value directly (it's already JSON-encoded)
                    if let Some(post_str) = post_data.as_str() {
                        if let Ok(post) = serde_json::from_str::<MattermostPost>(post_str) {
                            let message = post.into();
                            return Some(PlatformEvent::MessagePosted(message));
                        } else {
                            eprintln!("Failed to deserialize post JSON: {}", post_str);
                        }
                    } else {
                        eprintln!("Post data is not a string: {:?}", post_data);
                    }
                }
                eprintln!("Failed to parse 'posted' event data");
                None
            }
            "post_edited" => {
                // Extract and deserialize the post data for the edited message
                // Note: The "post" field is a JSON-encoded string, not a nested object
                if let Some(post_data) = ws_event.data.get("post") {
                    // Get the string value directly (it's already JSON-encoded)
                    if let Some(post_str) = post_data.as_str() {
                        if let Ok(post) = serde_json::from_str::<MattermostPost>(post_str) {
                            let message = post.into();
                            return Some(PlatformEvent::MessageUpdated(message));
                        } else {
                            eprintln!("Failed to deserialize post JSON: {}", post_str);
                        }
                    } else {
                        eprintln!("Post data is not a string: {:?}", post_data);
                    }
                }
                eprintln!("Failed to parse 'post_edited' event data");
                None
            }
            "post_deleted" => {
                // Extract the post ID from the post data
                // Note: The "post" field is a JSON-encoded string containing the full post object
                let post_id = if let Some(post_data) = ws_event.data.get("post") {
                    if let Some(post_str) = post_data.as_str() {
                        // Parse the post to extract the ID
                        if let Ok(post) = serde_json::from_str::<MattermostPost>(post_str) {
                            post.id
                        } else {
                            eprintln!("Failed to deserialize post JSON for deletion: {}", post_str);
                            String::new()
                        }
                    } else {
                        eprintln!("Post data is not a string: {:?}", post_data);
                        String::new()
                    }
                } else {
                    String::new()
                };

                Some(PlatformEvent::MessageDeleted {
                    message_id: post_id,
                    channel_id: ws_event.broadcast.channel_id,
                })
            }
            "typing" => Some(PlatformEvent::UserTyping {
                user_id: ws_event.data.get("user_id")
                    .and_then(|u| u.as_str())
                    .unwrap_or("")
                    .to_string(),
                channel_id: ws_event.broadcast.channel_id,
            }),
            "user_added" => Some(PlatformEvent::UserJoinedChannel {
                user_id: ws_event.data.get("user_id")
                    .and_then(|u| u.as_str())
                    .unwrap_or("")
                    .to_string(),
                channel_id: ws_event.broadcast.channel_id,
            }),
            "user_removed" => Some(PlatformEvent::UserLeftChannel {
                user_id: ws_event.data.get("user_id")
                    .and_then(|u| u.as_str())
                    .unwrap_or("")
                    .to_string(),
                channel_id: ws_event.broadcast.channel_id,
            }),
            "channel_created" => {
                // Extract and deserialize the channel data from the event
                if let Some(channel_data) = ws_event.data.get("channel") {
                    if let Ok(channel_str) = serde_json::to_string(channel_data) {
                        if let Ok(channel) = serde_json::from_str::<MattermostChannel>(&channel_str) {
                            let channel = channel.into();
                            return Some(PlatformEvent::ChannelCreated(channel));
                        }
                    }
                }
                // Fallback: if we can't parse the full channel, at least notify about the channel ID
                if !ws_event.broadcast.channel_id.is_empty() {
                    eprintln!("Failed to parse 'channel_created' event data, but channel ID available: {}",
                              ws_event.broadcast.channel_id);
                }
                None
            }
            "channel_deleted" => {
                Some(PlatformEvent::ChannelDeleted {
                    channel_id: ws_event.broadcast.channel_id,
                })
            }
            "channel_updated" => {
                // Extract and deserialize the channel data from the event
                if let Some(channel_data) = ws_event.data.get("channel") {
                    if let Ok(channel_str) = serde_json::to_string(channel_data) {
                        if let Ok(channel) = serde_json::from_str::<MattermostChannel>(&channel_str) {
                            let channel = channel.into();
                            return Some(PlatformEvent::ChannelUpdated(channel));
                        }
                    }
                }
                eprintln!("Failed to parse 'channel_updated' event data");
                None
            }
            "status_change" => {
                let user_id = ws_event.data.get("user_id")
                    .and_then(|u| u.as_str())
                    .unwrap_or("")
                    .to_string();
                let status_str = ws_event.data.get("status")
                    .and_then(|s| s.as_str())
                    .unwrap_or("offline");

                use crate::types::user::UserStatus;
                let status = match status_str {
                    "online" => UserStatus::Online,
                    "away" => UserStatus::Away,
                    "dnd" | "do_not_disturb" => UserStatus::DoNotDisturb,
                    "offline" => UserStatus::Offline,
                    _ => UserStatus::Unknown,
                };

                Some(PlatformEvent::UserStatusChanged { user_id, status })
            }
            "hello" => {
                // Connection established event - can be ignored or logged
                None
            }
            "reaction_added" | "reaction_removed" => {
                // Emoji reactions - log for now (full implementation would require Reaction type in platform_trait)
                println!("Reaction event: {} in channel {}", ws_event.event, ws_event.broadcast.channel_id);
                None
            }
            "direct_added" | "group_added" => {
                // New DM/GM channel created - log for now
                println!("New channel event: {} - channel {}", ws_event.event, ws_event.broadcast.channel_id);
                None
            }
            "preference_changed" | "preferences_changed" => {
                // User preferences changed - log for now
                println!("Preference changed event");
                None
            }
            "ephemeral_message" => {
                // Ephemeral message (temporary, usually bot responses) - log for now
                println!("Ephemeral message received");
                None
            }
            "new_user" | "user_updated" | "user_role_updated" => {
                // User events - log for now
                println!("User event: {}", ws_event.event);
                None
            }
            "channel_viewed" => {
                // User viewed channel - log for now
                println!("Channel viewed: {}", ws_event.broadcast.channel_id);
                None
            }
            _ => {
                // Unknown event type - log for visibility
                println!("Unknown/unhandled WebSocket event: {}", ws_event.event);
                None
            }
        }
    }

    /// Poll for the next event from the event queue
    ///
    /// # Returns
    /// An Option containing the next PlatformEvent, or None if the queue is empty
    pub async fn poll_event(&self) -> Option<PlatformEvent> {
        let mut rx = self.event_rx.lock().await;
        rx.try_recv().ok()
    }

    /// Disconnect from the WebSocket
    pub async fn disconnect(&mut self) {
        self.set_connection_state(ConnectionState::ShuttingDown).await;
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(()).await;
        }
        // State will be set to Disconnected by the spawned task
    }

}

impl Drop for WebSocketManager {
    fn drop(&mut self) {
        // Note: We can't use async in Drop, so we just drop the shutdown_tx
        // which will signal the task to stop
        self.shutdown_tx.take();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ws_url_conversion() {
        let manager = WebSocketManager::new("https://mattermost.example.com", "token".to_string());
        assert_eq!(manager.ws_url, "wss://mattermost.example.com/api/v4/websocket");

        let manager2 = WebSocketManager::new("http://localhost:8065", "token".to_string());
        assert_eq!(manager2.ws_url, "ws://localhost:8065/api/v4/websocket");
    }

    #[tokio::test]
    async fn test_event_queue() {
        let manager = WebSocketManager::new("https://mattermost.example.com", "token".to_string());

        // Initially empty - poll should return None
        assert!(manager.poll_event().await.is_none());

        // Send an event through the channel
        manager.event_tx.send(PlatformEvent::MessageDeleted {
            message_id: "msg123".to_string(),
            channel_id: "ch456".to_string(),
        }).await.unwrap();

        // Poll event
        let event = manager.poll_event().await;
        assert!(event.is_some());

        // Queue should be empty again
        assert!(manager.poll_event().await.is_none());
    }

    #[tokio::test]
    async fn test_event_queue_overflow() {
        // Create manager with small queue size
        let config = WebSocketConfig {
            max_queue_size: 2,
            ping_interval_secs: 30,
        };
        let manager = WebSocketManager::with_config(
            "https://mattermost.example.com",
            "token".to_string(),
            config,
        );

        // Fill the queue
        manager.event_tx.send(PlatformEvent::MessageDeleted {
            message_id: "msg1".to_string(),
            channel_id: "ch1".to_string(),
        }).await.unwrap();

        manager.event_tx.send(PlatformEvent::MessageDeleted {
            message_id: "msg2".to_string(),
            channel_id: "ch2".to_string(),
        }).await.unwrap();

        // Queue is now full, try_send should fail
        let result = manager.event_tx.try_send(PlatformEvent::MessageDeleted {
            message_id: "msg3".to_string(),
            channel_id: "ch3".to_string(),
        });

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), mpsc::error::TrySendError::Full(_)));

        // But we should still be able to receive the first two
        assert!(manager.poll_event().await.is_some());
        assert!(manager.poll_event().await.is_some());
        assert!(manager.poll_event().await.is_none());
    }

    #[test]
    fn test_parse_posted_event() {
        // Real data from Mattermost WebSocket
        let json = r#"{"event": "posted", "data": {"channel_display_name":"@jay","channel_name":"t1pn9rb63fnpjrqibgriijcx4r__xei6dqz8xfgm7kqzddjziyofyo","channel_type":"D","post":"{\"id\":\"a4aurxyyc3yruntz4zfmdw75nr\",\"create_at\":1761422860825,\"update_at\":1761422860825,\"edit_at\":0,\"delete_at\":0,\"is_pinned\":false,\"user_id\":\"t1pn9rb63fnpjrqibgriijcx4r\",\"channel_id\":\"4ckrmjaeeb8mbpodbmo6bknpge\",\"root_id\":\"\",\"original_id\":\"\",\"message\":\"aweff\",\"type\":\"\",\"props\":{\"disable_group_highlight\":true},\"hashtags\":\"\",\"file_ids\":[],\"pending_post_id\":\"t1pn9rb63fnpjrqibgriijcx4r:1761422860771\",\"remote_id\":\"\",\"reply_count\":0,\"last_reply_at\":0,\"participants\":null,\"metadata\":{}}","sender_name":"@jay","set_online":true,"team_id":""}, "broadcast": {"omit_users":null,"user_id":"","channel_id":"4ckrmjaeeb8mbpodbmo6bknpge","team_id":"","connection_id":"","omit_connection_id":""}, "seq": 35}"#;

        let ws_event: WebSocketEvent = serde_json::from_str(json).expect("Failed to parse WebSocket event");
        let platform_event = WebSocketManager::convert_event(ws_event);

        assert!(platform_event.is_some(), "Should successfully parse posted event");
        if let Some(PlatformEvent::MessagePosted(msg)) = platform_event {
            assert_eq!(msg.id, "a4aurxyyc3yruntz4zfmdw75nr");
            assert_eq!(msg.text, "aweff");
            assert_eq!(msg.channel_id, "4ckrmjaeeb8mbpodbmo6bknpge");
            assert_eq!(msg.sender_id, "t1pn9rb63fnpjrqibgriijcx4r");
        } else {
            panic!("Expected MessagePosted event");
        }
    }

    #[test]
    fn test_parse_post_edited_event() {
        // Real data from Mattermost WebSocket
        let json = r#"{"event": "post_edited", "data": {"post":"{\"id\":\"a4aurxyyc3yruntz4zfmdw75nr\",\"create_at\":1761422860825,\"update_at\":1761422988059,\"edit_at\":1761422988059,\"delete_at\":0,\"is_pinned\":false,\"user_id\":\"t1pn9rb63fnpjrqibgriijcx4r\",\"channel_id\":\"4ckrmjaeeb8mbpodbmo6bknpge\",\"root_id\":\"\",\"original_id\":\"\",\"message\":\"awe\",\"type\":\"\",\"props\":{\"disable_group_highlight\":true},\"hashtags\":\"\",\"file_ids\":[],\"pending_post_id\":\"\",\"remote_id\":\"\",\"reply_count\":0,\"last_reply_at\":0,\"participants\":null,\"metadata\":{}}"}, "broadcast": {"omit_users":null,"user_id":"","channel_id":"4ckrmjaeeb8mbpodbmo6bknpge","team_id":"","connection_id":"","omit_connection_id":""}, "seq": 37}"#;

        let ws_event: WebSocketEvent = serde_json::from_str(json).expect("Failed to parse WebSocket event");
        let platform_event = WebSocketManager::convert_event(ws_event);

        assert!(platform_event.is_some(), "Should successfully parse post_edited event");
        if let Some(PlatformEvent::MessageUpdated(msg)) = platform_event {
            assert_eq!(msg.id, "a4aurxyyc3yruntz4zfmdw75nr");
            assert_eq!(msg.text, "awe");
            assert_eq!(msg.channel_id, "4ckrmjaeeb8mbpodbmo6bknpge");
            assert_eq!(msg.sender_id, "t1pn9rb63fnpjrqibgriijcx4r");
        } else {
            panic!("Expected MessageUpdated event");
        }
    }

    #[test]
    fn test_parse_post_deleted_event() {
        // Real data from Mattermost WebSocket
        let json = r#"{"event": "post_deleted", "data": {"post":"{\"id\":\"a4aurxyyc3yruntz4zfmdw75nr\",\"create_at\":1761422860825,\"update_at\":1761422988059,\"edit_at\":1761422988059,\"delete_at\":0,\"is_pinned\":false,\"user_id\":\"t1pn9rb63fnpjrqibgriijcx4r\",\"channel_id\":\"4ckrmjaeeb8mbpodbmo6bknpge\",\"root_id\":\"\",\"original_id\":\"\",\"message\":\"awe\",\"type\":\"\",\"props\":{\"disable_group_highlight\":true},\"hashtags\":\"\",\"file_ids\":[],\"pending_post_id\":\"\",\"remote_id\":\"\",\"reply_count\":0,\"last_reply_at\":0,\"participants\":null}"}, "broadcast": {"omit_users":null,"user_id":"","channel_id":"4ckrmjaeeb8mbpodbmo6bknpge","team_id":"","connection_id":"","omit_connection_id":"","contains_sanitized_data":true}, "seq": 38}"#;

        let ws_event: WebSocketEvent = serde_json::from_str(json).expect("Failed to parse WebSocket event");
        let platform_event = WebSocketManager::convert_event(ws_event);

        assert!(platform_event.is_some(), "Should successfully parse post_deleted event");
        if let Some(PlatformEvent::MessageDeleted { message_id, channel_id }) = platform_event {
            assert_eq!(message_id, "a4aurxyyc3yruntz4zfmdw75nr");
            assert_eq!(channel_id, "4ckrmjaeeb8mbpodbmo6bknpge");
        } else {
            panic!("Expected MessageDeleted event");
        }
    }

    #[tokio::test]
    async fn test_connection_state() {
        let manager = WebSocketManager::new("https://mattermost.example.com", "token".to_string());

        // Should start in Disconnected state
        assert_eq!(manager.get_connection_state().await, ConnectionState::Disconnected);

        // State should change to Connecting when connect is called (will fail, but state changes)
        // Note: This test would need a mock server for full testing
    }
}
