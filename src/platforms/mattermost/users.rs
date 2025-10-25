use crate::error::Result;

use super::client::MattermostClient;
use super::types::MattermostUser;

impl MattermostClient {
    /// Get a user by ID
    ///
    /// # Arguments
    /// * `user_id` - The ID of the user to retrieve
    ///
    /// # Returns
    /// A Result containing the user information or an Error
    pub async fn get_user(&self, user_id: &str) -> Result<MattermostUser> {
        let endpoint = format!("/users/{user_id}");
        let response = self.get(&endpoint).await?;
        self.handle_response(response).await
    }

    /// Get the current authenticated user
    ///
    /// # Returns
    /// A Result containing the current user's information or an Error
    pub async fn get_current_user(&self) -> Result<MattermostUser> {
        let response = self.get("/users/me").await?;
        self.handle_response(response).await
    }

    /// Get a user by username
    ///
    /// # Arguments
    /// * `username` - The username to search for
    ///
    /// # Returns
    /// A Result containing the user information or an Error
    pub async fn get_user_by_username(&self, username: &str) -> Result<MattermostUser> {
        let endpoint = format!("/users/username/{username}");
        let response = self.get(&endpoint).await?;
        self.handle_response(response).await
    }

    /// Get a user by email
    ///
    /// # Arguments
    /// * `email` - The email address to search for
    ///
    /// # Returns
    /// A Result containing the user information or an Error
    pub async fn get_user_by_email(&self, email: &str) -> Result<MattermostUser> {
        let endpoint = format!("/users/email/{email}");
        let response = self.get(&endpoint).await?;
        self.handle_response(response).await
    }

    /// Get multiple users by their IDs
    ///
    /// # Arguments
    /// * `user_ids` - A list of user IDs to retrieve
    ///
    /// # Returns
    /// A Result containing a list of users or an Error
    pub async fn get_users_by_ids(&self, user_ids: &[String]) -> Result<Vec<MattermostUser>> {
        let response = self.post("/users/ids", &user_ids).await?;
        self.handle_response(response).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_endpoints() {
        let client = MattermostClient::new("https://mattermost.example.com").unwrap();

        // Test endpoint construction
        assert_eq!(
            client.api_url("/users/user123"),
            "https://mattermost.example.com/api/v4/users/user123"
        );
        assert_eq!(
            client.api_url("/users/me"),
            "https://mattermost.example.com/api/v4/users/me"
        );
    }
}
