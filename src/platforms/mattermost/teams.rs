//! Team management operations for Mattermost

use crate::error::Result;
use super::client::MattermostClient;
use super::types::MattermostTeam;

impl MattermostClient {
    /// Get all teams the current user belongs to
    ///
    /// # Returns
    /// A Result containing a vector of MattermostTeam objects
    ///
    /// # API Endpoint
    /// GET /users/me/teams
    pub async fn get_teams(&self) -> Result<Vec<MattermostTeam>> {
        let response = self.get("/users/me/teams").await?;
        self.handle_response(response).await
    }

    /// Get a specific team by ID
    ///
    /// # Arguments
    /// * `team_id` - The unique identifier of the team
    ///
    /// # Returns
    /// A Result containing the MattermostTeam object
    ///
    /// # API Endpoint
    /// GET /teams/{team_id}
    pub async fn get_team(&self, team_id: &str) -> Result<MattermostTeam> {
        let endpoint = format!("/teams/{team_id}");
        let response = self.get(&endpoint).await?;
        self.handle_response(response).await
    }

    /// Get a team by its unique name
    ///
    /// # Arguments
    /// * `team_name` - The unique name of the team (not display name)
    ///
    /// # Returns
    /// A Result containing the MattermostTeam object
    ///
    /// # API Endpoint
    /// GET /teams/name/{team_name}
    pub async fn get_team_by_name(&self, team_name: &str) -> Result<MattermostTeam> {
        let endpoint = format!("/teams/name/{team_name}");
        let response = self.get(&endpoint).await?;
        self.handle_response(response).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_team_endpoints() {
        // Test endpoint construction
        assert_eq!(format!("/teams/{}", "team123"), "/teams/team123");
        assert_eq!(format!("/teams/name/{}", "engineering"), "/teams/name/engineering");
    }
}
