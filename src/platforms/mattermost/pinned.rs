use crate::error::Result;

use super::client::MattermostClient;
use super::types::{MattermostPost, PostList};

impl MattermostClient {
    /// Pin a post to its channel
    ///
    /// # Arguments
    /// * `post_id` - The ID of the post to pin
    ///
    /// # Returns
    /// A Result indicating success or failure
    ///
    /// # Notes
    /// Requires `read_channel` permission for the channel the post is in
    pub async fn pin_post(&self, post_id: &str) -> Result<()> {
        let endpoint = format!("/posts/{post_id}/pin");
        let response = self.post(&endpoint, &serde_json::json!({})).await?;

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
                format!("Failed to pin post: {error_text}"),
            ))
        }
    }

    /// Unpin a post from its channel
    ///
    /// # Arguments
    /// * `post_id` - The ID of the post to unpin
    ///
    /// # Returns
    /// A Result indicating success or failure
    ///
    /// # Notes
    /// Requires `read_channel` permission for the channel the post is in
    pub async fn unpin_post(&self, post_id: &str) -> Result<()> {
        let endpoint = format!("/posts/{post_id}/unpin");
        let response = self.post(&endpoint, &serde_json::json!({})).await?;

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
                format!("Failed to unpin post: {error_text}"),
            ))
        }
    }

    /// Get all pinned posts for a channel
    ///
    /// # Arguments
    /// * `channel_id` - The ID of the channel
    ///
    /// # Returns
    /// A Result containing a vector of pinned posts or an Error
    pub async fn get_pinned_posts(&self, channel_id: &str) -> Result<Vec<MattermostPost>> {
        let endpoint = format!("/channels/{channel_id}/pinned");
        let response = self.get(&endpoint).await?;
        let post_list: PostList = self.handle_response(response).await?;

        // Convert PostList to Vec<MattermostPost>
        // The order field contains the post IDs in order
        let mut posts = Vec::new();
        for post_id in &post_list.order {
            if let Some(post) = post_list.posts.get(post_id) {
                posts.push(post.clone());
            }
        }

        Ok(posts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pinned_endpoints() {
        let client = MattermostClient::new("https://mattermost.example.com").unwrap();

        // Test pin endpoint construction
        assert_eq!(
            client.api_url("/posts/post123/pin"),
            "https://mattermost.example.com/api/v4/posts/post123/pin"
        );

        // Test unpin endpoint construction
        assert_eq!(
            client.api_url("/posts/post123/unpin"),
            "https://mattermost.example.com/api/v4/posts/post123/unpin"
        );

        // Test get pinned posts endpoint construction
        assert_eq!(
            client.api_url("/channels/channel123/pinned"),
            "https://mattermost.example.com/api/v4/channels/channel123/pinned"
        );
    }
}
