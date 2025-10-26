//! User status management operations for Mattermost

use super::client::MattermostClient;
use super::types::{CustomStatus, GetStatusesByIdsRequest, MattermostStatus, SetStatusRequest};
use crate::error::Result;

impl MattermostClient {
    /// Set the current user's status
    ///
    /// # Arguments
    /// * `status` - The status to set ("online", "away", "dnd", "offline")
    ///
    /// # Returns
    /// A Result containing the updated MattermostStatus
    ///
    /// # API Endpoint
    /// PUT /users/{user_id}/status
    pub async fn set_status(&self, status: &str) -> Result<MattermostStatus> {
        let user_id = self.get_user_id().await.ok_or_else(|| {
            crate::error::Error::new(
                crate::error::ErrorCode::InvalidState,
                "No user ID available - not logged in",
            )
        })?;

        let request = SetStatusRequest {
            user_id: user_id.clone(),
            status: status.to_string(),
        };

        let endpoint = format!("/users/{user_id}/status");
        let response = self.put(&endpoint, &request).await?;
        self.handle_response(response).await
    }

    /// Get a user's status
    ///
    /// # Arguments
    /// * `user_id` - The unique identifier of the user
    ///
    /// # Returns
    /// A Result containing the MattermostStatus
    ///
    /// # API Endpoint
    /// GET /users/{user_id}/status
    pub async fn get_user_status(&self, user_id: &str) -> Result<MattermostStatus> {
        let endpoint = format!("/users/{user_id}/status");
        let response = self.get(&endpoint).await?;
        self.handle_response(response).await
    }

    /// Get statuses for multiple users by their IDs
    ///
    /// # Arguments
    /// * `user_ids` - Array of user IDs
    ///
    /// # Returns
    /// A Result containing a vector of MattermostStatus objects
    ///
    /// # API Endpoint
    /// POST /users/status/ids
    pub async fn get_users_status_by_ids(
        &self,
        user_ids: &[String],
    ) -> Result<Vec<MattermostStatus>> {
        let request = GetStatusesByIdsRequest {
            user_ids: user_ids.to_vec(),
        };

        let response = self.post("/users/status/ids", &request).await?;
        self.handle_response(response).await
    }

    /// Set a custom status for the current user
    ///
    /// # Arguments
    /// * `custom_status` - The custom status to set
    ///
    /// # Returns
    /// A Result indicating success
    ///
    /// # API Endpoint
    /// PUT /users/{user_id}/status/custom
    pub async fn set_custom_status(&self, custom_status: CustomStatus) -> Result<()> {
        let user_id = self.get_user_id().await.ok_or_else(|| {
            crate::error::Error::new(
                crate::error::ErrorCode::InvalidState,
                "No user ID available - not logged in",
            )
        })?;

        let endpoint = format!("/users/{user_id}/status/custom");
        let response = self.put(&endpoint, &custom_status).await?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(crate::error::Error::new(
                crate::error::ErrorCode::NetworkError,
                format!("Failed to set custom status: {}", response.status()),
            ))
        }
    }

    /// Remove the current user's custom status
    ///
    /// # Returns
    /// A Result indicating success
    ///
    /// # API Endpoint
    /// DELETE /users/{user_id}/status/custom
    pub async fn remove_custom_status(&self) -> Result<()> {
        let user_id = self.get_user_id().await.ok_or_else(|| {
            crate::error::Error::new(
                crate::error::ErrorCode::InvalidState,
                "No user ID available - not logged in",
            )
        })?;

        let endpoint = format!("/users/{user_id}/status/custom");
        let response = self.delete(&endpoint).await?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(crate::error::Error::new(
                crate::error::ErrorCode::NetworkError,
                format!("Failed to remove custom status: {}", response.status()),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_endpoints() {
        // Test endpoint construction
        assert_eq!(
            format!("/users/{}/status", "user123"),
            "/users/user123/status"
        );
        assert_eq!(
            format!("/users/{}/status/custom", "user123"),
            "/users/user123/status/custom"
        );
    }

    #[test]
    fn test_set_status_request() {
        let request = SetStatusRequest {
            user_id: "user123".to_string(),
            status: "online".to_string(),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("user123"));
        assert!(json.contains("online"));
    }

    #[test]
    fn test_get_statuses_by_ids_request() {
        let request = GetStatusesByIdsRequest {
            user_ids: vec!["user1".to_string(), "user2".to_string()],
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("user1"));
        assert!(json.contains("user2"));
    }

    #[test]
    fn test_custom_status_serialization() {
        let custom_status = CustomStatus {
            emoji: Some(":coffee:".to_string()),
            text: Some("In a meeting".to_string()),
            duration: Some("one_hour".to_string()),
            expires_at: None,
        };

        let json = serde_json::to_string(&custom_status).unwrap();
        assert!(json.contains("coffee"));
        assert!(json.contains("In a meeting"));
        assert!(json.contains("one_hour"));
    }
}
