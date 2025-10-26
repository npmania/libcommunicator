//! Context and opaque handle management
//!
//! This module demonstrates the opaque handle pattern for FFI.
//! Rust objects are boxed and passed as opaque pointers to C,
//! then converted back when needed.

use crate::error::{Error, ErrorCode, Result};
use std::collections::HashMap;
use std::os::raw::c_void;

/// Log levels for callbacks
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Debug = 0,
    Info = 1,
    Warning = 2,
    Error = 3,
}

/// Callback function type for logging
/// Parameters: level, message, user_data
pub type LogCallback = extern "C" fn(LogLevel, *const std::os::raw::c_char, *mut c_void);

/// A communication context that manages connections to platforms
///
/// This is a Rust struct that will be exposed as an opaque handle through FFI
pub struct Context {
    /// User-defined identifier for this context
    pub id: String,
    /// Configuration options
    pub config: HashMap<String, String>,
    /// Internal state
    initialized: bool,
    /// Optional log callback
    log_callback: Option<LogCallback>,
    /// User data passed to callbacks
    user_data: *mut c_void,
}

impl Context {
    /// Create a new context
    pub fn new(id: impl Into<String>) -> Self {
        Context {
            id: id.into(),
            config: HashMap::new(),
            initialized: false,
            log_callback: None,
            user_data: std::ptr::null_mut(),
        }
    }

    /// Set a log callback
    pub fn set_log_callback(&mut self, callback: LogCallback, user_data: *mut c_void) {
        self.log_callback = Some(callback);
        self.user_data = user_data;
    }

    /// Clear the log callback
    pub fn clear_log_callback(&mut self) {
        self.log_callback = None;
        self.user_data = std::ptr::null_mut();
    }

    /// Log a message (internal helper)
    pub(crate) fn log(&self, level: LogLevel, message: &str) {
        if let Some(callback) = self.log_callback {
            if let Ok(c_string) = std::ffi::CString::new(message) {
                callback(level, c_string.as_ptr(), self.user_data);
            }
        }
    }

    /// Initialize the context
    pub fn initialize(&mut self) -> Result<()> {
        if self.initialized {
            return Err(Error::new(
                ErrorCode::InvalidState,
                "Context already initialized",
            ));
        }
        self.log(
            LogLevel::Info,
            &format!("Initializing context '{}'", self.id),
        );
        self.initialized = true;
        self.log(LogLevel::Info, "Context initialized successfully");
        Ok(())
    }

    /// Check if the context is initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Set a configuration value
    pub fn set_config(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.config.insert(key.into(), value.into());
    }

    /// Get a configuration value
    pub fn get_config(&self, key: &str) -> Option<&String> {
        self.config.get(key)
    }

    /// Shutdown the context
    pub fn shutdown(&mut self) -> Result<()> {
        if !self.initialized {
            return Err(Error::new(
                ErrorCode::InvalidState,
                "Context not initialized",
            ));
        }
        self.log(LogLevel::Info, "Shutting down context");
        self.initialized = false;
        self.config.clear();
        self.log(LogLevel::Info, "Context shutdown complete");
        Ok(())
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        // Ensure cleanup happens even if shutdown wasn't called
        if self.initialized {
            let _ = self.shutdown();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_lifecycle() {
        let mut ctx = Context::new("test");
        assert!(!ctx.is_initialized());

        ctx.initialize().unwrap();
        assert!(ctx.is_initialized());

        ctx.set_config("key", "value");
        assert_eq!(ctx.get_config("key").unwrap(), "value");

        ctx.shutdown().unwrap();
        assert!(!ctx.is_initialized());
    }

    #[test]
    fn test_double_initialize() {
        let mut ctx = Context::new("test");
        ctx.initialize().unwrap();
        assert!(ctx.initialize().is_err());
    }
}
