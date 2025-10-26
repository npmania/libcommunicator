//! Error handling for libcommunicator
//!
//! This module provides error types and FFI-compatible error handling mechanisms.

use std::fmt;
use std::sync::Mutex;

/// Result type used throughout the library
pub type Result<T> = std::result::Result<T, Error>;

/// Error codes for FFI
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    /// Operation succeeded
    Success = 0,
    /// Generic/unknown error
    Unknown = 1,
    /// Invalid argument provided
    InvalidArgument = 2,
    /// Null pointer was passed
    NullPointer = 3,
    /// Memory allocation failed
    OutOfMemory = 4,
    /// Invalid UTF-8 string
    InvalidUtf8 = 5,
    /// Network error
    NetworkError = 6,
    /// Authentication failed
    AuthenticationFailed = 7,
    /// Resource not found
    NotFound = 8,
    /// Permission denied
    PermissionDenied = 9,
    /// Timeout occurred
    Timeout = 10,
    /// Invalid state for operation
    InvalidState = 11,
    /// Feature not supported by this platform
    Unsupported = 12,
    /// Rate limit exceeded
    RateLimited = 13,
}

impl ErrorCode {
    pub fn as_str(&self) -> &'static str {
        match self {
            ErrorCode::Success => "Success",
            ErrorCode::Unknown => "Unknown error",
            ErrorCode::InvalidArgument => "Invalid argument",
            ErrorCode::NullPointer => "Null pointer",
            ErrorCode::OutOfMemory => "Out of memory",
            ErrorCode::InvalidUtf8 => "Invalid UTF-8 string",
            ErrorCode::NetworkError => "Network error",
            ErrorCode::AuthenticationFailed => "Authentication failed",
            ErrorCode::NotFound => "Not found",
            ErrorCode::PermissionDenied => "Permission denied",
            ErrorCode::Timeout => "Timeout",
            ErrorCode::InvalidState => "Invalid state",
            ErrorCode::Unsupported => "Feature not supported",
            ErrorCode::RateLimited => "Rate limit exceeded",
        }
    }
}

/// Internal error type
#[derive(Debug, Clone)]
pub struct Error {
    pub code: ErrorCode,
    pub message: String,
    /// Platform-specific error ID (e.g., Mattermost error ID like "api.user.login.invalid_credentials")
    pub(crate) mattermost_error_id: Option<String>,
    /// Request ID from server headers for debugging
    pub(crate) request_id: Option<String>,
    /// HTTP status code if this error came from an HTTP response
    pub(crate) http_status: Option<u16>,
}

impl Error {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Error {
            code,
            message: message.into(),
            mattermost_error_id: None,
            request_id: None,
            http_status: None,
        }
    }

    pub fn null_pointer() -> Self {
        Error::new(ErrorCode::NullPointer, "Null pointer provided")
    }

    pub fn invalid_utf8() -> Self {
        Error::new(ErrorCode::InvalidUtf8, "Invalid UTF-8 string")
    }

    pub fn invalid_argument(msg: impl Into<String>) -> Self {
        Error::new(ErrorCode::InvalidArgument, msg)
    }

    pub fn unsupported(msg: impl Into<String>) -> Self {
        Error::new(ErrorCode::Unsupported, msg)
    }

    /// Add Mattermost-specific error ID (builder pattern)
    pub fn with_mattermost_error_id(mut self, id: String) -> Self {
        self.mattermost_error_id = Some(id);
        self
    }

    /// Add request ID for debugging (builder pattern)
    pub fn with_request_id(mut self, id: String) -> Self {
        self.request_id = Some(id);
        self
    }

    /// Add HTTP status code (builder pattern)
    pub fn with_http_status(mut self, status: u16) -> Self {
        self.http_status = Some(status);
        self
    }

    /// Get the Mattermost error ID if available
    pub fn mattermost_error_id(&self) -> Option<&str> {
        self.mattermost_error_id.as_deref()
    }

    /// Get the request ID if available
    pub fn request_id(&self) -> Option<&str> {
        self.request_id.as_deref()
    }

    /// Get the HTTP status code if available
    pub fn http_status(&self) -> Option<u16> {
        self.http_status
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code.as_str(), self.message)
    }
}

impl std::error::Error for Error {}

// Thread-local error storage for FFI
lazy_static::lazy_static! {
    static ref LAST_ERROR: Mutex<Option<Error>> = Mutex::new(None);
}

/// Set the last error (called internally when FFI functions fail)
pub(crate) fn set_last_error(error: Error) {
    if let Ok(mut last) = LAST_ERROR.lock() {
        *last = Some(error);
    }
}

/// Clear the last error
pub(crate) fn clear_last_error() {
    if let Ok(mut last) = LAST_ERROR.lock() {
        *last = None;
    }
}

/// Get the last error (for FFI)
pub(crate) fn get_last_error() -> Option<Error> {
    LAST_ERROR.lock().ok()?.clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = Error::new(ErrorCode::NetworkError, "Connection failed");
        assert_eq!(err.code, ErrorCode::NetworkError);
        assert_eq!(err.message, "Connection failed");
    }

    #[test]
    fn test_error_storage() {
        clear_last_error();
        assert!(get_last_error().is_none());

        let err = Error::new(ErrorCode::InvalidArgument, "Test error");
        set_last_error(err.clone());

        let retrieved = get_last_error();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().code, ErrorCode::InvalidArgument);
    }

    #[test]
    fn test_error_with_additional_info() {
        let error = Error::new(ErrorCode::NotFound, "User not found")
            .with_mattermost_error_id("api.user.get.not_found".to_string())
            .with_request_id("abc123".to_string())
            .with_http_status(404);

        assert_eq!(error.code, ErrorCode::NotFound);
        assert_eq!(error.message, "User not found");
        assert_eq!(error.mattermost_error_id(), Some("api.user.get.not_found"));
        assert_eq!(error.request_id(), Some("abc123"));
        assert_eq!(error.http_status(), Some(404));
    }

    #[test]
    fn test_error_builder_pattern() {
        let error = Error::new(ErrorCode::AuthenticationFailed, "Login failed")
            .with_mattermost_error_id("api.user.login.invalid_credentials".to_string());

        assert_eq!(
            error.mattermost_error_id(),
            Some("api.user.login.invalid_credentials")
        );
        assert_eq!(error.request_id(), None);
        assert_eq!(error.http_status(), None);
    }

    #[test]
    fn test_error_without_additional_info() {
        let error = Error::new(ErrorCode::Unknown, "Generic error");

        assert_eq!(error.mattermost_error_id(), None);
        assert_eq!(error.request_id(), None);
        assert_eq!(error.http_status(), None);
    }
}
