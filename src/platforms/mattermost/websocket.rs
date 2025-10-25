use futures::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::error::{Error, ErrorCode, Result};
use crate::platforms::platform_trait::PlatformEvent;

use super::types::{MattermostChannel, MattermostPost, WebSocketAuthChallenge, WebSocketAuthData, WebSocketEvent};

/// WebSocket connection manager for Mattermost
pub struct WebSocketManager {
    /// URL for the WebSocket connection
    ws_url: String,
    /// Authentication token
    token: String,
    /// Event queue for received events
    event_queue: Arc<Mutex<Vec<PlatformEvent>>>,
    /// Shutdown signal sender
    shutdown_tx: Option<mpsc::Sender<()>>,
    /// Sequence number for WebSocket messages
    seq_number: Arc<Mutex<i64>>,
}

impl WebSocketManager {
    /// Create a new WebSocket manager
    ///
    /// # Arguments
    /// * `base_url` - The base URL of the Mattermost server
    /// * `token` - Authentication token for WebSocket authentication
    pub fn new(base_url: &str, token: String) -> Self {
        // Convert HTTP(S) URL to WebSocket URL
        let ws_url = base_url
            .replace("https://", "wss://")
            .replace("http://", "ws://");
        let ws_url = format!("{}/api/v4/websocket", ws_url);

        Self {
            ws_url,
            token,
            event_queue: Arc::new(Mutex::new(Vec::new())),
            shutdown_tx: None,
            seq_number: Arc::new(Mutex::new(1)),
        }
    }

    /// Connect to the Mattermost WebSocket and start receiving events
    ///
    /// # Returns
    /// A Result indicating success or failure
    pub async fn connect(&mut self) -> Result<()> {
        let (ws_stream, _) = connect_async(&self.ws_url)
            .await
            .map_err(|e| Error::new(ErrorCode::NetworkError, &format!("WebSocket connection failed: {}", e)))?;

        let (mut write, mut read) = ws_stream.split();

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

        // Create shutdown channel
        let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);
        self.shutdown_tx = Some(shutdown_tx);

        // Clone Arc references for the spawned task
        let event_queue = Arc::clone(&self.event_queue);

        // Spawn a task to handle incoming messages
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    // Handle incoming WebSocket messages
                    msg = read.next() => {
                        match msg {
                            Some(Ok(Message::Text(text))) => {
                                if let Err(e) = Self::handle_message(text, &event_queue).await {
                                    eprintln!("Error handling WebSocket message: {}", e);
                                }
                            }
                            Some(Ok(Message::Close(_))) => {
                                println!("WebSocket closed by server");
                                break;
                            }
                            Some(Err(e)) => {
                                eprintln!("WebSocket error: {}", e);
                                break;
                            }
                            None => {
                                println!("WebSocket stream ended");
                                break;
                            }
                            _ => {}
                        }
                    }
                    // Handle shutdown signal
                    _ = shutdown_rx.recv() => {
                        println!("WebSocket shutdown requested");
                        break;
                    }
                }
            }
        });

        Ok(())
    }

    /// Handle an incoming WebSocket message
    async fn handle_message(text: String, event_queue: &Arc<Mutex<Vec<PlatformEvent>>>) -> Result<()> {
        let ws_event: WebSocketEvent = serde_json::from_str(&text)
            .map_err(|e| Error::new(ErrorCode::Unknown, &format!("Failed to parse WebSocket event: {}", e)))?;

        // Convert WebSocket event to PlatformEvent
        if let Some(platform_event) = Self::convert_event(ws_event) {
            let mut queue = event_queue.lock().await;
            queue.push(platform_event);
        }

        Ok(())
    }

    /// Convert a Mattermost WebSocket event to a PlatformEvent
    fn convert_event(ws_event: WebSocketEvent) -> Option<PlatformEvent> {
        match ws_event.event.as_str() {
            "posted" => {
                // Extract and deserialize the post data from the event
                if let Some(post_data) = ws_event.data.get("post") {
                    if let Ok(post_str) = serde_json::to_string(post_data) {
                        if let Ok(post) = serde_json::from_str::<MattermostPost>(&post_str) {
                            let message = post.into();
                            return Some(PlatformEvent::MessagePosted(message));
                        }
                    }
                }
                eprintln!("Failed to parse 'posted' event data");
                None
            }
            "post_edited" => {
                // Extract and deserialize the post data for the edited message
                if let Some(post_data) = ws_event.data.get("post") {
                    if let Ok(post_str) = serde_json::to_string(post_data) {
                        if let Ok(post) = serde_json::from_str::<MattermostPost>(&post_str) {
                            let message = post.into();
                            return Some(PlatformEvent::MessageUpdated(message));
                        }
                    }
                }
                eprintln!("Failed to parse 'post_edited' event data");
                None
            }
            "post_deleted" => {
                let post_id = ws_event.data.get("post")
                    .and_then(|p| p.as_str())
                    .unwrap_or("")
                    .to_string();

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
            _ => {
                // Unknown event type
                println!("Unknown WebSocket event: {}", ws_event.event);
                None
            }
        }
    }

    /// Poll for the next event from the event queue
    ///
    /// # Returns
    /// An Option containing the next PlatformEvent, or None if the queue is empty
    pub async fn poll_event(&self) -> Option<PlatformEvent> {
        let mut queue = self.event_queue.lock().await;
        if !queue.is_empty() {
            Some(queue.remove(0))
        } else {
            None
        }
    }

    /// Disconnect from the WebSocket
    pub async fn disconnect(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(()).await;
        }
    }

    /// Check if there are any events in the queue
    pub async fn has_events(&self) -> bool {
        let queue = self.event_queue.lock().await;
        !queue.is_empty()
    }

    /// Get the number of events in the queue
    pub async fn event_count(&self) -> usize {
        let queue = self.event_queue.lock().await;
        queue.len()
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

        // Initially empty
        assert!(!manager.has_events().await);
        assert_eq!(manager.event_count().await, 0);

        // Add an event manually
        {
            let mut queue = manager.event_queue.lock().await;
            queue.push(PlatformEvent::MessageDeleted {
                message_id: "msg123".to_string(),
                channel_id: "ch456".to_string(),
            });
        }

        assert!(manager.has_events().await);
        assert_eq!(manager.event_count().await, 1);

        // Poll event
        let event = manager.poll_event().await;
        assert!(event.is_some());
        assert!(!manager.has_events().await);
    }
}
