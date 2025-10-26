//! Emoji types for custom emojis across platforms

use serde::{Deserialize, Serialize};

/// A custom emoji
///
/// Represents a custom emoji that has been uploaded to the platform.
/// This does not include standard Unicode emojis, which are available by default.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Emoji {
    /// Unique identifier for the emoji
    pub id: String,

    /// Name of the emoji (without colons, e.g., "thumbsup" not ":thumbsup:")
    pub name: String,

    /// User ID of the person who created/uploaded this emoji
    pub creator_id: String,

    /// Timestamp when the emoji was created (Unix timestamp in milliseconds)
    pub created_at: i64,
}

impl Emoji {
    /// Create a new Emoji
    pub fn new(id: String, name: String, creator_id: String, created_at: i64) -> Self {
        Self {
            id,
            name,
            creator_id,
            created_at,
        }
    }

    /// Get the emoji name with colons (e.g., ":thumbsup:")
    pub fn name_with_colons(&self) -> String {
        format!(":{}:", self.name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emoji_creation() {
        let emoji = Emoji::new(
            "emoji123".to_string(),
            "parrot".to_string(),
            "user456".to_string(),
            1234567890000,
        );

        assert_eq!(emoji.id, "emoji123");
        assert_eq!(emoji.name, "parrot");
        assert_eq!(emoji.creator_id, "user456");
        assert_eq!(emoji.created_at, 1234567890000);
    }

    #[test]
    fn test_name_with_colons() {
        let emoji = Emoji::new(
            "emoji123".to_string(),
            "parrot".to_string(),
            "user456".to_string(),
            1234567890000,
        );

        assert_eq!(emoji.name_with_colons(), ":parrot:");
    }
}
