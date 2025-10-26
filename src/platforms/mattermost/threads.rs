use crate::error::Result;

use super::client::MattermostClient;
use super::types::{PostList, UserThread, UserThreads};

impl MattermostClient {
    /// Get a thread and all its replies
    ///
    /// Fetches a post and all posts in the same thread (replies).
    ///
    /// # Arguments
    /// * `post_id` - ID of any post in the thread (typically the root post)
    ///
    /// # Returns
    /// A Result containing a PostList with the thread posts
    ///
    /// # API Endpoint
    /// `GET /api/v4/posts/{post_id}/thread`
    pub async fn get_thread(&self, post_id: &str) -> Result<PostList> {
        let endpoint = format!("/posts/{post_id}/thread");
        let response = self.get(&endpoint).await?;
        self.handle_response(response).await
    }

    /// Get all threads that a user is following
    ///
    /// Retrieves threads that the user has participated in or is following.
    ///
    /// # Arguments
    /// * `user_id` - The user ID (can be "me" for current user)
    /// * `team_id` - The team ID
    /// * `since` - Optional timestamp to filter threads (Unix epoch in milliseconds)
    /// * `deleted` - Include deleted threads (for mobile sync)
    /// * `unread` - Filter to unread threads only
    /// * `threads_only` - Only return threads (exclude other posts)
    /// * `page` - Page number (0-indexed)
    /// * `per_page` - Number of threads per page
    ///
    /// # Returns
    /// A Result containing UserThreads response
    ///
    /// # API Endpoint
    /// `GET /api/v4/users/{user_id}/teams/{team_id}/threads`
    pub async fn get_user_threads(
        &self,
        user_id: &str,
        team_id: &str,
        since: Option<i64>,
        deleted: bool,
        unread: bool,
        threads_only: bool,
        page: u32,
        per_page: u32,
    ) -> Result<UserThreads> {
        let mut endpoint = format!(
            "/users/{user_id}/teams/{team_id}/threads?page={page}&perPage={per_page}"
        );

        if let Some(since_ts) = since {
            endpoint.push_str(&format!("&since={since_ts}"));
        }
        if deleted {
            endpoint.push_str("&deleted=true");
        }
        if unread {
            endpoint.push_str("&unread=true");
        }
        if threads_only {
            endpoint.push_str("&threadsOnly=true");
        }

        let response = self.get(&endpoint).await?;
        self.handle_response(response).await
    }

    /// Get a specific thread that the user is following
    ///
    /// # Arguments
    /// * `user_id` - The user ID (can be "me" for current user)
    /// * `team_id` - The team ID
    /// * `thread_id` - The thread ID (root post ID)
    ///
    /// # Returns
    /// A Result containing the UserThread
    ///
    /// # API Endpoint
    /// `GET /api/v4/users/{user_id}/teams/{team_id}/threads/{thread_id}`
    pub async fn get_user_thread(
        &self,
        user_id: &str,
        team_id: &str,
        thread_id: &str,
    ) -> Result<UserThread> {
        let endpoint = format!("/users/{user_id}/teams/{team_id}/threads/{thread_id}");
        let response = self.get(&endpoint).await?;
        self.handle_response(response).await
    }

    /// Start following a thread
    ///
    /// Makes the user follow a thread to receive notifications for new replies.
    ///
    /// # Arguments
    /// * `user_id` - The user ID (can be "me" for current user)
    /// * `team_id` - The team ID
    /// * `thread_id` - The thread ID (root post ID) to follow
    ///
    /// # Returns
    /// Result indicating success or failure
    ///
    /// # API Endpoint
    /// `PUT /api/v4/users/{user_id}/teams/{team_id}/threads/{thread_id}/following`
    ///
    /// # Minimum Server Version
    /// 5.29
    pub async fn follow_thread(
        &self,
        user_id: &str,
        team_id: &str,
        thread_id: &str,
    ) -> Result<()> {
        let endpoint = format!("/users/{user_id}/teams/{team_id}/threads/{thread_id}/following");
        let response = self.put(&endpoint, &serde_json::json!({})).await?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(crate::error::Error::new(
                crate::error::ErrorCode::NetworkError,
                format!("Failed to follow thread: {}", response.status()),
            ))
        }
    }

    /// Stop following a thread
    ///
    /// Makes the user unfollow a thread to stop receiving notifications.
    ///
    /// # Arguments
    /// * `user_id` - The user ID (can be "me" for current user)
    /// * `team_id` - The team ID
    /// * `thread_id` - The thread ID (root post ID) to unfollow
    ///
    /// # Returns
    /// Result indicating success or failure
    ///
    /// # API Endpoint
    /// `DELETE /api/v4/users/{user_id}/teams/{team_id}/threads/{thread_id}/following`
    ///
    /// # Minimum Server Version
    /// 5.29
    pub async fn unfollow_thread(
        &self,
        user_id: &str,
        team_id: &str,
        thread_id: &str,
    ) -> Result<()> {
        let endpoint = format!("/users/{user_id}/teams/{team_id}/threads/{thread_id}/following");
        let response = self.delete(&endpoint).await?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(crate::error::Error::new(
                crate::error::ErrorCode::NetworkError,
                format!("Failed to unfollow thread: {}", response.status()),
            ))
        }
    }

    /// Mark a thread as read up to a specific timestamp
    ///
    /// Updates the user's "last read" state for a thread.
    ///
    /// # Arguments
    /// * `user_id` - The user ID (can be "me" for current user)
    /// * `team_id` - The team ID
    /// * `thread_id` - The thread ID (root post ID)
    /// * `timestamp` - Unix timestamp in milliseconds to mark as read up to
    ///
    /// # Returns
    /// Result indicating success or failure
    ///
    /// # API Endpoint
    /// `PUT /api/v4/users/{user_id}/teams/{team_id}/threads/{thread_id}/read/{timestamp}`
    ///
    /// # Minimum Server Version
    /// 5.29
    pub async fn mark_thread_as_read(
        &self,
        user_id: &str,
        team_id: &str,
        thread_id: &str,
        timestamp: i64,
    ) -> Result<()> {
        let endpoint =
            format!("/users/{user_id}/teams/{team_id}/threads/{thread_id}/read/{timestamp}");
        let response = self.put(&endpoint, &serde_json::json!({})).await?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(crate::error::Error::new(
                crate::error::ErrorCode::NetworkError,
                format!("Failed to mark thread as read: {}", response.status()),
            ))
        }
    }

    /// Mark a thread as unread based on a specific post
    ///
    /// Marks a thread as unread starting from a specific post in the thread.
    ///
    /// # Arguments
    /// * `user_id` - The user ID (can be "me" for current user)
    /// * `team_id` - The team ID
    /// * `thread_id` - The thread ID (root post ID)
    /// * `post_id` - The post ID to mark as unread from
    ///
    /// # Returns
    /// Result indicating success or failure
    ///
    /// # API Endpoint
    /// `POST /api/v4/users/{user_id}/teams/{team_id}/threads/{thread_id}/set_unread/{post_id}`
    ///
    /// # Minimum Server Version
    /// 5.29
    pub async fn mark_thread_as_unread(
        &self,
        user_id: &str,
        team_id: &str,
        thread_id: &str,
        post_id: &str,
    ) -> Result<()> {
        let endpoint = format!(
            "/users/{user_id}/teams/{team_id}/threads/{thread_id}/set_unread/{post_id}"
        );
        let response = self.post(&endpoint, &serde_json::json!({})).await?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(crate::error::Error::new(
                crate::error::ErrorCode::NetworkError,
                format!("Failed to mark thread as unread: {}", response.status()),
            ))
        }
    }

    /// Mark all followed threads as read
    ///
    /// Marks all threads that the user is following as read.
    ///
    /// # Arguments
    /// * `user_id` - The user ID (can be "me" for current user)
    /// * `team_id` - The team ID
    ///
    /// # Returns
    /// Result indicating success or failure
    ///
    /// # API Endpoint
    /// `PUT /api/v4/users/{user_id}/teams/{team_id}/threads/read`
    ///
    /// # Minimum Server Version
    /// 5.29
    pub async fn mark_all_threads_as_read(&self, user_id: &str, team_id: &str) -> Result<()> {
        let endpoint = format!("/users/{user_id}/teams/{team_id}/threads/read");
        let response = self.put(&endpoint, &serde_json::json!({})).await?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(crate::error::Error::new(
                crate::error::ErrorCode::NetworkError,
                format!("Failed to mark all threads as read: {}", response.status()),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thread_endpoints() {
        let client = MattermostClient::new("https://mattermost.example.com").unwrap();

        // Test endpoint construction
        assert_eq!(
            client.api_url("/posts/post123/thread"),
            "https://mattermost.example.com/api/v4/posts/post123/thread"
        );

        assert_eq!(
            client.api_url("/users/me/teams/team123/threads?page=0&perPage=60"),
            "https://mattermost.example.com/api/v4/users/me/teams/team123/threads?page=0&perPage=60"
        );

        assert_eq!(
            client.api_url("/users/me/teams/team123/threads/thread123/following"),
            "https://mattermost.example.com/api/v4/users/me/teams/team123/threads/thread123/following"
        );

        assert_eq!(
            client.api_url("/users/me/teams/team123/threads/thread123/read/1234567890"),
            "https://mattermost.example.com/api/v4/users/me/teams/team123/threads/thread123/read/1234567890"
        );
    }
}
