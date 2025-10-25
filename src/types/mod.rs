//! Core types for libcommunicator
//!
//! This module contains platform-agnostic types used across all platform adapters.

pub mod capabilities;
pub mod channel;
pub mod connection;
pub mod message;
pub mod team;
pub mod user;

// Re-export for convenience
pub use capabilities::PlatformCapabilities;
pub use channel::{Channel, ChannelType};
pub use connection::{ConnectionInfo, ConnectionState};
pub use message::{Attachment, Message};
pub use team::{Team, TeamType};
pub use user::User;
