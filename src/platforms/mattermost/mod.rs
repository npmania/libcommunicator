//! Mattermost platform adapter
//!
//! This module implements the communication layer for Mattermost servers.
//! The OpenAPI specification for the Mattermost API is available in
//! `api-spec.yaml` in this directory.

mod auth;
mod channels;
mod client;
mod convert;
mod platform_impl;
mod posts;
mod reactions;
mod status;
mod teams;
mod types;
mod users;
mod websocket;

pub use client::{MattermostClient, RateLimitInfo};
pub use convert::{status_string_to_user_status, user_status_to_status_string};
pub use platform_impl::MattermostPlatform;
pub use types::*;
