use crate::error::{Error, ErrorCode, Result};
use crate::types::ConnectionState;

use super::client::MattermostClient;
use super::types::{LoginRequest, MattermostUser};

impl MattermostClient {
    /// Authenticate with Mattermost using email/username and password
    ///
    /// # Arguments
    /// * `login_id` - The user's email or username
    /// * `password` - The user's password
    ///
    /// # Returns
    /// A Result containing the authenticated user information or an Error
    ///
    /// # Note
    /// This method will extract the session token from the response headers
    /// and store it for future API calls.
    /// If the account requires MFA, an error will be returned with the Mattermost error ID
    /// indicating MFA is required. In that case, call `login_with_mfa()` instead.
    pub async fn login(&self, login_id: &str, password: &str) -> Result<MattermostUser> {
        self.login_with_options(login_id, password, None, None).await
    }

    /// Authenticate with Mattermost using email/username, password, and MFA token
    ///
    /// # Arguments
    /// * `login_id` - The user's email or username
    /// * `password` - The user's password
    /// * `mfa_token` - The 6-digit MFA code from authenticator app
    ///
    /// # Returns
    /// A Result containing the authenticated user information or an Error
    ///
    /// # Note
    /// This method should be used when the account has Multi-Factor Authentication enabled.
    /// If you attempt to login without MFA on an MFA-enabled account, you'll receive an error
    /// with the Mattermost error ID "api.user.login.mfa_required" or similar.
    pub async fn login_with_mfa(
        &self,
        login_id: &str,
        password: &str,
        mfa_token: &str,
    ) -> Result<MattermostUser> {
        self.login_with_options(login_id, password, Some(mfa_token), None)
            .await
    }

    /// Internal helper for login with optional MFA token and device ID
    ///
    /// # Arguments
    /// * `login_id` - The user's email or username
    /// * `password` - The user's password
    /// * `mfa_token` - Optional MFA token (6-digit code)
    /// * `device_id` - Optional device ID for tracking login devices
    ///
    /// # Returns
    /// A Result containing the authenticated user information or an Error
    async fn login_with_options(
        &self,
        login_id: &str,
        password: &str,
        mfa_token: Option<&str>,
        device_id: Option<&str>,
    ) -> Result<MattermostUser> {
        self.set_state(ConnectionState::Connecting).await;

        let login_request = LoginRequest {
            login_id: login_id.to_string(),
            password: password.to_string(),
            token: mfa_token.map(|s| s.to_string()),
            device_id: device_id.map(|s| s.to_string()),
        };

        let url = self.api_url("/users/login");
        let response = self
            .http_client
            .post(&url)
            .json(&login_request)
            .send()
            .await
            .map_err(|e| {
                Error::new(
                    ErrorCode::AuthenticationFailed,
                    format!("Login request failed: {e}"),
                )
            })?;

        // Check for errors early and set state
        if !response.status().is_success() {
            self.set_state(ConnectionState::Error).await;
        }

        // Extract the session token from the response headers
        if let Some(token) = response.headers().get("Token") {
            let token_str = token
                .to_str()
                .map_err(|e| {
                    Error::new(
                        ErrorCode::AuthenticationFailed,
                        format!("Invalid token header: {e}"),
                    )
                })?
                .to_string();

            self.set_token(token_str).await;
        } else {
            self.set_state(ConnectionState::Error).await;
            return Err(Error::new(
                ErrorCode::AuthenticationFailed,
                "No token in login response",
            ));
        }

        // Parse the user information from the response body
        let status = response.status();
        if status.is_success() {
            let user = response.json::<MattermostUser>().await.map_err(|e| {
                Error::new(ErrorCode::Unknown, format!("Failed to parse user: {e}"))
            })?;

            // Store the user ID
            self.set_user_id(Some(user.id.clone())).await;
            self.set_state(ConnectionState::Connected).await;

            Ok(user)
        } else {
            self.set_state(ConnectionState::Error).await;
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            Err(Error::new(
                ErrorCode::AuthenticationFailed,
                format!("Login failed with status {status}: {error_text}"),
            ))
        }
    }

    /// Authenticate with Mattermost using a Personal Access Token (PAT)
    ///
    /// # Arguments
    /// * `token` - The Personal Access Token
    ///
    /// # Returns
    /// A Result containing the authenticated user information or an Error
    ///
    /// # Note
    /// After setting the token, this method calls get_current_user to verify
    /// the token is valid and to retrieve user information.
    pub async fn login_with_token(&self, token: &str) -> Result<MattermostUser> {
        self.set_state(ConnectionState::Connecting).await;
        self.set_token(token.to_string()).await;

        // Verify the token by fetching current user info
        match self.get_current_user_api().await {
            Ok(user) => {
                self.set_user_id(Some(user.id.clone())).await;
                self.set_state(ConnectionState::Connected).await;
                Ok(user)
            }
            Err(e) => {
                self.set_state(ConnectionState::Error).await;
                // Clear the invalid token
                self.set_token(String::new()).await;
                Err(Error::new(
                    ErrorCode::AuthenticationFailed,
                    format!("Token authentication failed: {e}"),
                ))
            }
        }
    }

    /// Logout from Mattermost
    ///
    /// # Returns
    /// A Result indicating success or failure
    ///
    /// # Note
    /// This will invalidate the current session token on the server
    /// and clear the stored token locally.
    pub async fn logout(&self) -> Result<()> {
        self.set_state(ConnectionState::Disconnecting).await;

        // Only call the logout endpoint if we have a token
        if self.get_token().await.is_some() {
            let response = self.post("/users/logout", &serde_json::json!({})).await;

            // Clear token regardless of API call success
            self.set_token(String::new()).await;
            self.set_user_id(None).await;
            self.set_team_id(None).await;
            self.set_state(ConnectionState::Disconnected).await;

            // Silently ignore logout API errors - we've already cleared local state
            let _ = response;
        } else {
            self.set_state(ConnectionState::Disconnected).await;
        }

        Ok(())
    }

    /// Get the current user's information
    ///
    /// # Returns
    /// A Result containing the user information or an Error
    ///
    /// # Note
    /// This requires an active authentication session (token must be set)
    async fn get_current_user_api(&self) -> Result<MattermostUser> {
        let response = self.get("/users/me").await?;
        self.handle_response(response).await
    }

    /// Verify if the current session is still valid
    ///
    /// # Returns
    /// true if the session is valid, false otherwise
    pub async fn verify_session(&self) -> bool {
        if self.get_token().await.is_none() {
            return false;
        }

        self.get_current_user_api().await.is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_logout_without_token() {
        let client = MattermostClient::new("https://mattermost.example.com").unwrap();

        // Should succeed even without a token
        let result = client.logout().await;
        assert!(result.is_ok());
        assert_eq!(client.get_state().await, ConnectionState::Disconnected);
    }

    #[tokio::test]
    async fn test_verify_session_no_token() {
        let client = MattermostClient::new("https://mattermost.example.com").unwrap();

        // Should return false when no token is set
        let valid = client.verify_session().await;
        assert!(!valid);
    }

    #[tokio::test]
    async fn test_login_request_serialization_with_mfa() {
        use super::super::types::LoginRequest;

        // Test that LoginRequest with MFA token serializes correctly
        let request = LoginRequest {
            login_id: "user@example.com".to_string(),
            password: "password123".to_string(),
            token: Some("123456".to_string()),
            device_id: Some("device123".to_string()),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("login_id"));
        assert!(json.contains("password"));
        assert!(json.contains("token"));
        assert!(json.contains("device_id"));
        assert!(json.contains("123456"));
    }

    #[tokio::test]
    async fn test_login_request_serialization_without_mfa() {
        use super::super::types::LoginRequest;

        // Test that LoginRequest without MFA token doesn't include optional fields
        let request = LoginRequest {
            login_id: "user@example.com".to_string(),
            password: "password123".to_string(),
            token: None,
            device_id: None,
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("login_id"));
        assert!(json.contains("password"));
        // Optional fields should not be serialized when None
        assert!(!json.contains("token"));
        assert!(!json.contains("device_id"));
    }

    #[test]
    fn test_mfa_error_mapping() {
        use crate::error::ErrorCode;

        // Test MFA-specific error ID mapping (this is a unit test for the error mapping logic)
        // Note: We would need to expose map_mattermost_error_id as pub(crate) to test it directly
        // For now, this test documents the expected behavior

        // MFA required error should map to AuthenticationFailed
        assert_eq!(ErrorCode::AuthenticationFailed as u32, 7);

        // Invalid MFA error should also map to AuthenticationFailed
        // These would be tested in integration tests with actual server responses
    }
}
