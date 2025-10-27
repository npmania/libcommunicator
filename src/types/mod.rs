//! Core types for libcommunicator
//!
//! This module contains platform-agnostic types used across all platform adapters.

pub mod capabilities;
pub mod channel;
pub mod connection;
pub mod emoji;
pub mod message;
pub mod team;
pub mod user;

// Re-export for convenience
pub use capabilities::PlatformCapabilities;
pub use channel::{Channel, ChannelType, ChannelUnread};
pub use connection::{ConnectionInfo, ConnectionState};
pub use emoji::Emoji;
pub use message::{Attachment, Message};
pub use team::{Team, TeamType, TeamUnread};
pub use user::User;
