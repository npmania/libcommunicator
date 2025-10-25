/// Platform-specific implementations for different chat services
///
/// Each platform module provides an adapter that implements the core
/// communication interface for that specific service.

mod platform_trait;

pub mod mattermost;

// Re-export platform trait and related types
pub use platform_trait::{Platform, PlatformConfig, PlatformEvent};
