//! Message types for chat communications

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Represents a chat message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Unique identifier for this message
    pub id: String,
    /// The message text/content
    pub text: String,
    /// User ID of the sender
    pub sender_id: String,
    /// Channel/conversation ID where this message was sent
    pub channel_id: String,
    /// When the message was created
    pub created_at: DateTime<Utc>,
    /// When the message was last edited (if applicable)
    pub edited_at: Option<DateTime<Utc>>,
    /// Optional attachments (files, images, etc.)
    pub attachments: Vec<Attachment>,
    /// Optional metadata (platform-specific)
    pub metadata: Option<serde_json::Value>,
}

impl Message {
    /// Create a new message
    pub fn new(
        id: impl Into<String>,
        text: impl Into<String>,
        sender_id: impl Into<String>,
        channel_id: impl Into<String>,
    ) -> Self {
        Message {
            id: id.into(),
            text: text.into(),
            sender_id: sender_id.into(),
            channel_id: channel_id.into(),
            created_at: Utc::now(),
            edited_at: None,
            attachments: Vec::new(),
            metadata: None,
        }
    }

    /// Add an attachment to this message
    pub fn with_attachment(mut self, attachment: Attachment) -> Self {
        self.attachments.push(attachment);
        self
    }

    /// Set metadata for this message
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

/// Represents a file or media attachment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    /// Unique identifier for this attachment
    pub id: String,
    /// Filename
    pub filename: String,
    /// MIME type (e.g., "image/png", "application/pdf")
    pub mime_type: String,
    /// Size in bytes
    pub size: u64,
    /// URL to access the file
    pub url: String,
    /// Optional thumbnail URL (for images/videos)
    pub thumbnail_url: Option<String>,
}

impl Attachment {
    /// Create a new attachment
    pub fn new(
        id: impl Into<String>,
        filename: impl Into<String>,
        mime_type: impl Into<String>,
        size: u64,
        url: impl Into<String>,
    ) -> Self {
        Attachment {
            id: id.into(),
            filename: filename.into(),
            mime_type: mime_type.into(),
            size,
            url: url.into(),
            thumbnail_url: None,
        }
    }

    /// Set thumbnail URL
    pub fn with_thumbnail(mut self, thumbnail_url: impl Into<String>) -> Self {
        self.thumbnail_url = Some(thumbnail_url.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_creation() {
        let msg = Message::new("msg-1", "Hello, world!", "user-1", "channel-1");
        assert_eq!(msg.id, "msg-1");
        assert_eq!(msg.text, "Hello, world!");
        assert_eq!(msg.sender_id, "user-1");
        assert_eq!(msg.channel_id, "channel-1");
        assert!(msg.attachments.is_empty());
        assert!(msg.metadata.is_none());
    }

    #[test]
    fn test_attachment_creation() {
        let attachment = Attachment::new(
            "att-1",
            "document.pdf",
            "application/pdf",
            1024,
            "https://example.com/file.pdf",
        );
        assert_eq!(attachment.id, "att-1");
        assert_eq!(attachment.filename, "document.pdf");
        assert_eq!(attachment.size, 1024);
    }

    #[test]
    fn test_message_with_attachment() {
        let attachment = Attachment::new(
            "att-1",
            "image.png",
            "image/png",
            2048,
            "https://example.com/image.png",
        );
        let msg = Message::new("msg-1", "Check this out", "user-1", "channel-1")
            .with_attachment(attachment);
        assert_eq!(msg.attachments.len(), 1);
        assert_eq!(msg.attachments[0].filename, "image.png");
    }
}
