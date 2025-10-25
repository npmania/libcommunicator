//! Core types for libcommunicator
//!
//! This module contains platform-agnostic types used across all platform adapters.

pub mod channel;
pub mod connection;
pub mod message;
pub mod user;

// Re-export for convenience
pub use channel::{Channel, ChannelType};
pub use connection::{ConnectionInfo, ConnectionState};
pub use message::{Attachment, Message};
pub use user::User;
