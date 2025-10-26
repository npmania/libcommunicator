use crate::error::Result;

use super::client::MattermostClient;
use super::types::{Reaction, SaveReactionRequest};

impl MattermostClient {
    /// Add a reaction to a post
    ///
    /// # Arguments
    /// * `post_id` - The ID of the post to react to
    /// * `emoji_name` - The name of the emoji (e.g., "thumbsup", "smile")
    ///
    /// # Returns
    /// A Result containing the created reaction or an Error
    pub async fn add_reaction(&self, post_id: &str, emoji_name: &str) -> Result<Reaction> {
        // Get the current user ID from connection info
        let user_id = self.current_user_id().await?;

        let request = SaveReactionRequest {
            user_id,
            post_id: post_id.to_string(),
            emoji_name: emoji_name.to_string(),
        };

        let response = self.post("/reactions", &request).await?;
        self.handle_response(response).await
    }

    /// Remove a reaction from a post
    ///
    /// # Arguments
    /// * `post_id` - The ID of the post
    /// * `emoji_name` - The name of the emoji to remove
    ///
    /// # Returns
    /// A Result indicating success or failure
    pub async fn remove_reaction(&self, post_id: &str, emoji_name: &str) -> Result<()> {
        // Get the current user ID from connection info
        let user_id = self.current_user_id().await?;

        let endpoint = format!("/users/{user_id}/posts/{post_id}/reactions/{emoji_name}");
        let response = self.delete(&endpoint).await?;

        // Check response status
        if response.status().is_success() {
            Ok(())
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(crate::error::Error::new(
                crate::error::ErrorCode::Unknown,
                format!("Failed to remove reaction: {error_text}"),
            ))
        }
    }

    /// Get all reactions for a post
    ///
    /// # Arguments
    /// * `post_id` - The ID of the post
    ///
    /// # Returns
    /// A Result containing a vector of reactions or an Error
    pub async fn get_reactions(&self, post_id: &str) -> Result<Vec<Reaction>> {
        let endpoint = format!("/posts/{post_id}/reactions");
        let response = self.get(&endpoint).await?;
        self.handle_response(response).await
    }
}
