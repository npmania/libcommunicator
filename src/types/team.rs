//! Team/workspace types for chat platforms
//!
//! This module defines organizational units that group channels/conversations.
//! Different platforms call this concept by different names:
//! - Mattermost: Team
//! - Slack: Workspace
//! - Discord: Guild/Server
//! - Microsoft Teams: Team
//! - Matrix: Space
//!
//! Not all platforms have this concept (e.g., IRC, basic Telegram).
//! Check PlatformCapabilities.has_workspaces before using team-related methods.

use serde::{Deserialize, Serialize};

/// Represents a team/workspace/guild on a chat platform
///
/// This is a generic organizational container that groups channels together.
/// The exact semantics depend on the platform.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Team {
    /// Unique identifier for this team
    pub id: String,
    /// Team name (unique identifier, often used in URLs)
    pub name: String,
    /// Display name (what users see)
    pub display_name: String,
    /// Team description
    pub description: Option<String>,
    /// Team type (e.g., "O" for open, "I" for invite-only)
    pub team_type: TeamType,
    /// Email domain for automatic team membership
    pub allowed_domains: Option<String>,
    /// Whether users can invite others
    pub allow_open_invite: bool,
    /// Optional metadata (platform-specific)
    pub metadata: Option<serde_json::Value>,
}

/// Team type/visibility
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[derive(Default)]
pub enum TeamType {
    /// Open team - anyone can join
    Open,
    /// Invite-only team
    #[default]
    Invite,
}

impl Team {
    /// Create a new team
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        display_name: impl Into<String>,
    ) -> Self {
        Team {
            id: id.into(),
            name: name.into(),
            display_name: display_name.into(),
            description: None,
            team_type: TeamType::Invite,
            allowed_domains: None,
            allow_open_invite: false,
            metadata: None,
        }
    }

    /// Set description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set team type
    pub fn with_team_type(mut self, team_type: TeamType) -> Self {
        self.team_type = team_type;
        self
    }

    /// Set allowed domains
    pub fn with_allowed_domains(mut self, domains: impl Into<String>) -> Self {
        self.allowed_domains = Some(domains.into());
        self
    }

    /// Set allow open invite
    pub fn with_open_invite(mut self, allow: bool) -> Self {
        self.allow_open_invite = allow;
        self
    }

    /// Set metadata
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_team_creation() {
        let team = Team::new("team-1", "engineering", "Engineering Team");
        assert_eq!(team.id, "team-1");
        assert_eq!(team.name, "engineering");
        assert_eq!(team.display_name, "Engineering Team");
        assert!(team.description.is_none());
        assert_eq!(team.team_type, TeamType::Invite);
        assert!(!team.allow_open_invite);
    }

    #[test]
    fn test_team_builder() {
        let team = Team::new("team-2", "sales", "Sales Team")
            .with_description("Our awesome sales team")
            .with_team_type(TeamType::Open)
            .with_allowed_domains("example.com")
            .with_open_invite(true);

        assert_eq!(team.description, Some("Our awesome sales team".to_string()));
        assert_eq!(team.team_type, TeamType::Open);
        assert_eq!(team.allowed_domains, Some("example.com".to_string()));
        assert!(team.allow_open_invite);
    }

    #[test]
    fn test_team_type_default() {
        let team_type = TeamType::default();
        assert_eq!(team_type, TeamType::Invite);
    }
}
