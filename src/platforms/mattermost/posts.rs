use crate::error::Result;

use super::client::MattermostClient;
use super::types::{CreatePostRequest, MattermostPost, PostList};

impl MattermostClient {
    /// Send a message (post) to a channel
    ///
    /// # Arguments
    /// * `channel_id` - The ID of the channel to send the message to
    /// * `message` - The message text to send
    ///
    /// # Returns
    /// A Result containing the created post or an Error
    pub async fn send_message(&self, channel_id: &str, message: &str) -> Result<MattermostPost> {
        let request = CreatePostRequest::new(channel_id.to_string(), message.to_string());

        let response = self.post("/posts", &request).await?;
        self.handle_response(response).await
    }

    /// Send a message as a reply to another post
    ///
    /// # Arguments
    /// * `channel_id` - The ID of the channel
    /// * `message` - The message text to send
    /// * `root_id` - The ID of the post to reply to
    ///
    /// # Returns
    /// A Result containing the created post or an Error
    pub async fn send_reply(&self, channel_id: &str, message: &str, root_id: &str) -> Result<MattermostPost> {
        let request = CreatePostRequest::new(channel_id.to_string(), message.to_string())
            .with_root_id(root_id.to_string());

        let response = self.post("/posts", &request).await?;
        self.handle_response(response).await
    }

    /// Get a specific post by ID
    ///
    /// # Arguments
    /// * `post_id` - The ID of the post to retrieve
    ///
    /// # Returns
    /// A Result containing the post or an Error
    pub async fn get_post(&self, post_id: &str) -> Result<MattermostPost> {
        let endpoint = format!("/posts/{}", post_id);
        let response = self.get(&endpoint).await?;
        self.handle_response(response).await
    }

    /// Get posts for a channel
    ///
    /// # Arguments
    /// * `channel_id` - The ID of the channel
    /// * `page` - Page number to retrieve (0-indexed)
    /// * `per_page` - Number of posts per page (default 60, max 200)
    ///
    /// # Returns
    /// A Result containing a PostList or an Error
    pub async fn get_posts_for_channel(
        &self,
        channel_id: &str,
        page: u32,
        per_page: u32,
    ) -> Result<PostList> {
        let endpoint = format!("/channels/{}/posts?page={}&per_page={}", channel_id, page, per_page);
        let response = self.get(&endpoint).await?;
        self.handle_response(response).await
    }

    /// Get the latest posts for a channel
    ///
    /// # Arguments
    /// * `channel_id` - The ID of the channel
    /// * `limit` - Number of posts to retrieve (default 60)
    ///
    /// # Returns
    /// A Result containing a PostList or an Error
    pub async fn get_latest_posts(&self, channel_id: &str, limit: u32) -> Result<PostList> {
        self.get_posts_for_channel(channel_id, 0, limit).await
    }

    /// Update a post
    ///
    /// # Arguments
    /// * `post_id` - The ID of the post to update
    /// * `message` - The new message text
    ///
    /// # Returns
    /// A Result containing the updated post or an Error
    pub async fn update_post(&self, post_id: &str, message: &str) -> Result<MattermostPost> {
        let body = serde_json::json!({
            "id": post_id,
            "message": message,
        });

        let endpoint = format!("/posts/{}", post_id);
        let response = self.put(&endpoint, &body).await?;
        self.handle_response(response).await
    }

    /// Delete a post
    ///
    /// # Arguments
    /// * `post_id` - The ID of the post to delete
    ///
    /// # Returns
    /// A Result indicating success or failure
    pub async fn delete_post(&self, post_id: &str) -> Result<()> {
        let endpoint = format!("/posts/{}", post_id);
        let response = self.delete(&endpoint).await?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(crate::error::Error::new(
                crate::error::ErrorCode::NetworkError,
                &format!("Failed to delete post: {}", response.status()),
            ))
        }
    }

    /// Search for posts in a team
    ///
    /// # Arguments
    /// * `team_id` - The ID of the team to search in
    /// * `terms` - The search terms
    ///
    /// # Returns
    /// A Result containing a PostList or an Error
    pub async fn search_posts(&self, team_id: &str, terms: &str) -> Result<PostList> {
        let body = serde_json::json!({
            "terms": terms,
            "is_or_search": false,
        });

        let endpoint = format!("/teams/{}/posts/search", team_id);
        let response = self.post(&endpoint, &body).await?;
        self.handle_response(response).await
    }

    /// Get posts created before a specific post (for pagination)
    ///
    /// # Arguments
    /// * `channel_id` - The ID of the channel
    /// * `post_id` - Get posts before this post ID
    /// * `per_page` - Number of posts to retrieve
    ///
    /// # Returns
    /// A Result containing a PostList or an Error
    pub async fn get_posts_before(
        &self,
        channel_id: &str,
        post_id: &str,
        per_page: u32,
    ) -> Result<PostList> {
        let endpoint = format!(
            "/channels/{}/posts?before={}&per_page={}",
            channel_id, post_id, per_page
        );
        let response = self.get(&endpoint).await?;
        self.handle_response(response).await
    }

    /// Get posts created after a specific post (for pagination)
    ///
    /// # Arguments
    /// * `channel_id` - The ID of the channel
    /// * `post_id` - Get posts after this post ID
    /// * `per_page` - Number of posts to retrieve
    ///
    /// # Returns
    /// A Result containing a PostList or an Error
    pub async fn get_posts_after(
        &self,
        channel_id: &str,
        post_id: &str,
        per_page: u32,
    ) -> Result<PostList> {
        let endpoint = format!(
            "/channels/{}/posts?after={}&per_page={}",
            channel_id, post_id, per_page
        );
        let response = self.get(&endpoint).await?;
        self.handle_response(response).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_post_endpoints() {
        let client = MattermostClient::new("https://mattermost.example.com").unwrap();

        // Test endpoint construction
        assert_eq!(
            client.api_url("/posts"),
            "https://mattermost.example.com/api/v4/posts"
        );
        assert_eq!(
            client.api_url("/posts/post123"),
            "https://mattermost.example.com/api/v4/posts/post123"
        );
        assert_eq!(
            client.api_url("/channels/channel123/posts?page=0&per_page=60"),
            "https://mattermost.example.com/api/v4/channels/channel123/posts?page=0&per_page=60"
        );
    }
}
