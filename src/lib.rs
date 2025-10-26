use std::ffi::CString;
use std::os::raw::{c_char, c_void};

// Core modules
pub mod context;
pub mod error;
pub mod platforms;
pub mod runtime;
pub mod types;

// Re-exports for convenience
pub use context::{Context, LogCallback, LogLevel};
pub use error::{Error, ErrorCode, Result};
pub use platforms::{Platform, PlatformConfig, PlatformEvent};
pub use types::{
    Attachment, Channel, ChannelType, ConnectionInfo, ConnectionState, Emoji, Message, Team,
    TeamType, User,
};

// Library version information
pub const VERSION_MAJOR: u32 = 0;
pub const VERSION_MINOR: u32 = 1;
pub const VERSION_PATCH: u32 = 0;
pub const VERSION_STRING: &str = concat!(
    env!("CARGO_PKG_VERSION"),
    " (libcommunicator)"
);

/// FFI function: Free a string allocated by this library
#[no_mangle]
///
/// # Safety
/// This function is unsafe because it deals with raw pointers from C.
/// The caller must ensure all pointer arguments are valid.
pub unsafe extern "C" fn communicator_free_string(s: *mut c_char) {
    if !s.is_null() {
        unsafe {
            let _ = CString::from_raw(s);
        }
    }
}

// ============================================================================
// Library Initialization Pattern
// ============================================================================

/// FFI function: Initialize the library
/// This should be called once before using any other library functions
/// Returns ErrorCode indicating success or failure
#[no_mangle]
///
/// # Safety
/// This function is unsafe because it deals with raw pointers from C.
/// The caller must ensure all pointer arguments are valid.
pub unsafe extern "C" fn communicator_init() -> ErrorCode {
    error::clear_last_error();

    // Initialize the async runtime
    match runtime::init_runtime() {
        Ok(()) => ErrorCode::Success,
        Err(e) => {
            let code = e.code;
            error::set_last_error(e);
            code
        }
    }
}

/// FFI function: Cleanup the library
/// This should be called once when done using the library
/// Frees any global resources allocated by the library
#[no_mangle]
///
/// # Safety
/// This function is unsafe because it deals with raw pointers from C.
/// The caller must ensure all pointer arguments are valid.
pub unsafe extern "C" fn communicator_cleanup() {
    error::clear_last_error();

    // Shutdown the async runtime
    runtime::shutdown_runtime();
}

// ============================================================================
// Version Information
// ============================================================================

/// FFI function: Get the library version string
/// Returns a static string, do NOT free this pointer
#[no_mangle]
///
/// # Safety
/// This function is unsafe because it deals with raw pointers from C.
/// The caller must ensure all pointer arguments are valid.
pub unsafe extern "C" fn communicator_version() -> *const c_char {
    concat!(env!("CARGO_PKG_VERSION"), " (libcommunicator)\0").as_ptr() as *const c_char
}

/// FFI function: Get the major version number
#[no_mangle]
///
/// # Safety
/// This function is unsafe because it deals with raw pointers from C.
/// The caller must ensure all pointer arguments are valid.
pub unsafe extern "C" fn communicator_version_major() -> u32 {
    VERSION_MAJOR
}

/// FFI function: Get the minor version number
#[no_mangle]
///
/// # Safety
/// This function is unsafe because it deals with raw pointers from C.
/// The caller must ensure all pointer arguments are valid.
pub unsafe extern "C" fn communicator_version_minor() -> u32 {
    VERSION_MINOR
}

/// FFI function: Get the patch version number
#[no_mangle]
///
/// # Safety
/// This function is unsafe because it deals with raw pointers from C.
/// The caller must ensure all pointer arguments are valid.
pub unsafe extern "C" fn communicator_version_patch() -> u32 {
    VERSION_PATCH
}

// ============================================================================
// Error Handling FFI
// ============================================================================

/// FFI function: Get the error code of the last error
/// Returns ErrorCode::Success (0) if no error has occurred
#[no_mangle]
///
/// # Safety
/// This function is unsafe because it deals with raw pointers from C.
/// The caller must ensure all pointer arguments are valid.
pub unsafe extern "C" fn communicator_last_error_code() -> ErrorCode {
    error::get_last_error()
        .map(|e| e.code)
        .unwrap_or(ErrorCode::Success)
}

/// FFI function: Get the error message of the last error
/// Returns a dynamically allocated string that must be freed with communicator_free_string()
/// Returns NULL if no error has occurred
#[no_mangle]
///
/// # Safety
/// This function is unsafe because it deals with raw pointers from C.
/// The caller must ensure all pointer arguments are valid.
pub unsafe extern "C" fn communicator_last_error_message() -> *mut c_char {
    let error = match error::get_last_error() {
        Some(e) => e,
        None => return std::ptr::null_mut(),
    };

    match CString::new(error.message) {
        Ok(c_string) => c_string.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

/// FFI function: Get a human-readable description of an error code
/// Returns a static string, do NOT free this pointer
#[no_mangle]
///
/// # Safety
/// This function is unsafe because it deals with raw pointers from C.
/// The caller must ensure all pointer arguments are valid.
pub unsafe extern "C" fn communicator_error_code_string(code: ErrorCode) -> *const c_char {
    let s = match code {
        ErrorCode::Success => "Success\0",
        ErrorCode::Unknown => "Unknown error\0",
        ErrorCode::InvalidArgument => "Invalid argument\0",
        ErrorCode::NullPointer => "Null pointer\0",
        ErrorCode::OutOfMemory => "Out of memory\0",
        ErrorCode::InvalidUtf8 => "Invalid UTF-8 string\0",
        ErrorCode::NetworkError => "Network error\0",
        ErrorCode::AuthenticationFailed => "Authentication failed\0",
        ErrorCode::NotFound => "Not found\0",
        ErrorCode::PermissionDenied => "Permission denied\0",
        ErrorCode::Timeout => "Timeout\0",
        ErrorCode::InvalidState => "Invalid state\0",
        ErrorCode::Unsupported => "Feature not supported\0",
        ErrorCode::RateLimited => "Rate limit exceeded\0",
    };
    s.as_ptr() as *const c_char
}

/// FFI function: Clear the last error
#[no_mangle]
///
/// # Safety
/// This function is unsafe because it deals with raw pointers from C.
/// The caller must ensure all pointer arguments are valid.
pub unsafe extern "C" fn communicator_clear_error() {
    error::clear_last_error();
}

// ============================================================================
// Opaque Handle Pattern - Context Management
// ============================================================================

/// Opaque handle to a Context object
/// This is a pointer to a Rust-managed object
pub type ContextHandle = *mut Context;

/// FFI function: Create a new context
/// Returns an opaque handle to the context
/// The handle must be freed with communicator_context_destroy()
/// Returns NULL on error
#[no_mangle]
///
/// # Safety
/// This function is unsafe because it deals with raw pointers from C.
/// The caller must ensure all pointer arguments are valid.
pub unsafe extern "C" fn communicator_context_create(id: *const c_char) -> ContextHandle {
    error::clear_last_error();

    if id.is_null() {
        error::set_last_error(Error::null_pointer());
        return std::ptr::null_mut();
    }

    let id_str = {
        match std::ffi::CStr::from_ptr(id).to_str() {
            Ok(s) => s,
            Err(_) => {
                error::set_last_error(Error::invalid_utf8());
                return std::ptr::null_mut();
            }
        }
    };

    let context = Box::new(Context::new(id_str));
    Box::into_raw(context)
}

/// FFI function: Initialize a context
/// Returns ErrorCode indicating success or failure
#[no_mangle]
///
/// # Safety
/// This function is unsafe because it deals with raw pointers from C.
/// The caller must ensure all pointer arguments are valid.
pub unsafe extern "C" fn communicator_context_initialize(handle: ContextHandle) -> ErrorCode {
    error::clear_last_error();

    if handle.is_null() {
        error::set_last_error(Error::null_pointer());
        return ErrorCode::NullPointer;
    }

    let context = &mut *handle;

    match context.initialize() {
        Ok(()) => ErrorCode::Success,
        Err(e) => {
            let code = e.code;
            error::set_last_error(e);
            code
        }
    }
}

/// FFI function: Check if a context is initialized
/// Returns 1 if initialized, 0 if not, -1 on error
#[no_mangle]
///
/// # Safety
/// This function is unsafe because it deals with raw pointers from C.
/// The caller must ensure all pointer arguments are valid.
pub unsafe extern "C" fn communicator_context_is_initialized(handle: ContextHandle) -> i32 {
    error::clear_last_error();

    if handle.is_null() {
        error::set_last_error(Error::null_pointer());
        return -1;
    }

    let context = &*handle;
    if context.is_initialized() { 1 } else { 0 }
}

/// FFI function: Set a configuration value on a context
/// Returns ErrorCode indicating success or failure
#[no_mangle]
///
/// # Safety
/// This function is unsafe because it deals with raw pointers from C.
/// The caller must ensure all pointer arguments are valid.
pub unsafe extern "C" fn communicator_context_set_config(
    handle: ContextHandle,
    key: *const c_char,
    value: *const c_char,
) -> ErrorCode {
    error::clear_last_error();

    if handle.is_null() || key.is_null() || value.is_null() {
        error::set_last_error(Error::null_pointer());
        return ErrorCode::NullPointer;
    }

    let key_str = {
        match std::ffi::CStr::from_ptr(key).to_str() {
            Ok(s) => s,
            Err(_) => {
                error::set_last_error(Error::invalid_utf8());
                return ErrorCode::InvalidUtf8;
            }
        }
    };

    let value_str = {
        match std::ffi::CStr::from_ptr(value).to_str() {
            Ok(s) => s,
            Err(_) => {
                error::set_last_error(Error::invalid_utf8());
                return ErrorCode::InvalidUtf8;
            }
        }
    };

    let context = &mut *handle;
    context.set_config(key_str, value_str);
    ErrorCode::Success
}

/// FFI function: Get a configuration value from a context
/// Returns a dynamically allocated string that must be freed with communicator_free_string()
/// Returns NULL if the key doesn't exist or on error
#[no_mangle]
///
/// # Safety
/// This function is unsafe because it deals with raw pointers from C.
/// The caller must ensure all pointer arguments are valid.
pub unsafe extern "C" fn communicator_context_get_config(
    handle: ContextHandle,
    key: *const c_char,
) -> *mut c_char {
    error::clear_last_error();

    if handle.is_null() || key.is_null() {
        error::set_last_error(Error::null_pointer());
        return std::ptr::null_mut();
    }

    let key_str = {
        match std::ffi::CStr::from_ptr(key).to_str() {
            Ok(s) => s,
            Err(_) => {
                error::set_last_error(Error::invalid_utf8());
                return std::ptr::null_mut();
            }
        }
    };

    let context = &*handle;

    match context.get_config(key_str) {
        Some(value) => match CString::new(value.as_str()) {
            Ok(c_string) => c_string.into_raw(),
            Err(_) => {
                error::set_last_error(Error::new(
                    ErrorCode::OutOfMemory,
                    "Failed to allocate string",
                ));
                std::ptr::null_mut()
            }
        },
        None => {
            error::set_last_error(Error::new(ErrorCode::NotFound, "Key not found"));
            std::ptr::null_mut()
        }
    }
}

/// FFI function: Shutdown a context
/// Returns ErrorCode indicating success or failure
#[no_mangle]
///
/// # Safety
/// This function is unsafe because it deals with raw pointers from C.
/// The caller must ensure all pointer arguments are valid.
pub unsafe extern "C" fn communicator_context_shutdown(handle: ContextHandle) -> ErrorCode {
    error::clear_last_error();

    if handle.is_null() {
        error::set_last_error(Error::null_pointer());
        return ErrorCode::NullPointer;
    }

    let context = &mut *handle;

    match context.shutdown() {
        Ok(()) => ErrorCode::Success,
        Err(e) => {
            let code = e.code;
            error::set_last_error(e);
            code
        }
    }
}

/// FFI function: Destroy a context and free its memory
/// After calling this, the handle is invalid and must not be used
#[no_mangle]
///
/// # Safety
/// This function is unsafe because it deals with raw pointers from C.
/// The caller must ensure all pointer arguments are valid.
pub unsafe extern "C" fn communicator_context_destroy(handle: ContextHandle) {
    if !handle.is_null() {
        unsafe {
            let _ = Box::from_raw(handle);
        }
    }
}

// ============================================================================
// Callback Pattern - Function Pointers
// ============================================================================

/// FFI function: Set a log callback on a context
/// The callback will be called for logging events
/// user_data is an opaque pointer passed back to the callback
#[no_mangle]
///
/// # Safety
/// This function is unsafe because it deals with raw pointers from C.
/// The caller must ensure all pointer arguments are valid.
pub unsafe extern "C" fn communicator_context_set_log_callback(
    handle: ContextHandle,
    callback: LogCallback,
    user_data: *mut c_void,
) -> ErrorCode {
    error::clear_last_error();

    if handle.is_null() {
        error::set_last_error(Error::null_pointer());
        return ErrorCode::NullPointer;
    }

    let context = &mut *handle;
    context.set_log_callback(callback, user_data);
    ErrorCode::Success
}

/// FFI function: Clear the log callback on a context
#[no_mangle]
///
/// # Safety
/// This function is unsafe because it deals with raw pointers from C.
/// The caller must ensure all pointer arguments are valid.
pub unsafe extern "C" fn communicator_context_clear_log_callback(handle: ContextHandle) -> ErrorCode {
    error::clear_last_error();

    if handle.is_null() {
        error::set_last_error(Error::null_pointer());
        return ErrorCode::NullPointer;
    }

    let context = &mut *handle;
    context.clear_log_callback();
    ErrorCode::Success
}

// ============================================================================
// Platform FFI - Opaque Handle Pattern
// ============================================================================

/// Opaque handle to a Platform object
pub type PlatformHandle = *mut Box<dyn Platform>;

/// FFI function: Create a new Mattermost platform instance
/// Returns an opaque handle to the platform
/// The handle must be freed with communicator_platform_destroy()
/// Returns NULL on error
#[no_mangle]
///
/// # Safety
/// This function is unsafe because it deals with raw pointers from C.
/// The caller must ensure all pointer arguments are valid.
pub unsafe extern "C" fn communicator_mattermost_create(server_url: *const c_char) -> PlatformHandle {
    error::clear_last_error();

    if server_url.is_null() {
        error::set_last_error(Error::null_pointer());
        return std::ptr::null_mut();
    }

    let url_str = {
        match std::ffi::CStr::from_ptr(server_url).to_str() {
            Ok(s) => s,
            Err(_) => {
                error::set_last_error(Error::invalid_utf8());
                return std::ptr::null_mut();
            }
        }
    };

    match platforms::mattermost::MattermostPlatform::new(url_str) {
        Ok(platform) => {
            let boxed: Box<dyn Platform> = Box::new(platform);
            Box::into_raw(Box::new(boxed))
        }
        Err(e) => {
            error::set_last_error(e);
            std::ptr::null_mut()
        }
    }
}

/// FFI function: Connect to a platform
/// config_json: JSON string with format:
/// {
///   "server": "https://mattermost.example.com",
///   "credentials": {
///     "token": "xxx" OR "login_id": "user@example.com", "password": "xxx"
///   },
///   "team_id": "optional-team-id"
/// }
/// Returns ErrorCode indicating success or failure
#[no_mangle]
///
/// # Safety
/// This function is unsafe because it deals with raw pointers from C.
/// The caller must ensure all pointer arguments are valid.
pub unsafe extern "C" fn communicator_platform_connect(
    handle: PlatformHandle,
    config_json: *const c_char,
) -> ErrorCode {
    error::clear_last_error();

    if handle.is_null() || config_json.is_null() {
        error::set_last_error(Error::null_pointer());
        return ErrorCode::NullPointer;
    }

    let config_str = {
        match std::ffi::CStr::from_ptr(config_json).to_str() {
            Ok(s) => s,
            Err(_) => {
                error::set_last_error(Error::invalid_utf8());
                return ErrorCode::InvalidUtf8;
            }
        }
    };

    // Parse JSON into PlatformConfig
    #[derive(serde::Deserialize)]
    struct ConfigJson {
        server: String,
        credentials: std::collections::HashMap<String, String>,
        team_id: Option<String>,
    }

    let config_data: ConfigJson = match serde_json::from_str(config_str) {
        Ok(c) => c,
        Err(e) => {
            error::set_last_error(Error::new(
                ErrorCode::InvalidArgument,
                format!("Invalid config JSON: {e}"),
            ));
            return ErrorCode::InvalidArgument;
        }
    };

    let mut platform_config = PlatformConfig::new(config_data.server);
    platform_config.credentials = config_data.credentials;
    platform_config.team_id = config_data.team_id;

    let platform = &mut **handle;

    // Run async connect in blocking mode
    match runtime::block_on(platform.connect(platform_config)) {
        Ok(_) => ErrorCode::Success,
        Err(e) => {
            let code = e.code;
            error::set_last_error(e);
            code
        }
    }
}

/// FFI function: Disconnect from a platform
/// Returns ErrorCode indicating success or failure
#[no_mangle]
///
/// # Safety
/// This function is unsafe because it deals with raw pointers from C.
/// The caller must ensure all pointer arguments are valid.
pub unsafe extern "C" fn communicator_platform_disconnect(handle: PlatformHandle) -> ErrorCode {
    error::clear_last_error();

    if handle.is_null() {
        error::set_last_error(Error::null_pointer());
        return ErrorCode::NullPointer;
    }

    let platform = &mut **handle;

    match runtime::block_on(platform.disconnect()) {
        Ok(()) => ErrorCode::Success,
        Err(e) => {
            let code = e.code;
            error::set_last_error(e);
            code
        }
    }
}

/// FFI function: Check if platform is connected
/// Returns 1 if connected, 0 if not, -1 on error
#[no_mangle]
///
/// # Safety
/// This function is unsafe because it deals with raw pointers from C.
/// The caller must ensure all pointer arguments are valid.
pub unsafe extern "C" fn communicator_platform_is_connected(handle: PlatformHandle) -> i32 {
    error::clear_last_error();

    if handle.is_null() {
        error::set_last_error(Error::null_pointer());
        return -1;
    }

    let platform = &**handle;
    if platform.is_connected() { 1 } else { 0 }
}

/// FFI function: Get connection info as JSON
/// Returns a dynamically allocated JSON string that must be freed with communicator_free_string()
/// Returns NULL on error or if not connected
#[no_mangle]
///
/// # Safety
/// This function is unsafe because it deals with raw pointers from C.
/// The caller must ensure all pointer arguments are valid.
pub unsafe extern "C" fn communicator_platform_get_connection_info(
    handle: PlatformHandle,
) -> *mut c_char {
    error::clear_last_error();

    if handle.is_null() {
        error::set_last_error(Error::null_pointer());
        return std::ptr::null_mut();
    }

    let platform = &**handle;

    match platform.connection_info() {
        Some(info) => match serde_json::to_string(info) {
            Ok(json) => match CString::new(json) {
                Ok(c_string) => c_string.into_raw(),
                Err(_) => {
                    error::set_last_error(Error::new(
                        ErrorCode::OutOfMemory,
                        "Failed to allocate string",
                    ));
                    std::ptr::null_mut()
                }
            },
            Err(e) => {
                error::set_last_error(Error::new(
                    ErrorCode::Unknown,
                    format!("Failed to serialize connection info: {e}"),
                ));
                std::ptr::null_mut()
            }
        },
        None => {
            error::set_last_error(Error::new(
                ErrorCode::InvalidState,
                "Not connected",
            ));
            std::ptr::null_mut()
        }
    }
}

/// FFI function: Send a message to a channel
/// Returns a JSON string representing the created Message
/// The caller must free the returned string using communicator_free_string()
/// Returns NULL on error
#[no_mangle]
///
/// # Safety
/// This function is unsafe because it deals with raw pointers from C.
/// The caller must ensure all pointer arguments are valid.
pub unsafe extern "C" fn communicator_platform_send_message(
    handle: PlatformHandle,
    channel_id: *const c_char,
    text: *const c_char,
) -> *mut c_char {
    error::clear_last_error();

    if handle.is_null() || channel_id.is_null() || text.is_null() {
        error::set_last_error(Error::null_pointer());
        return std::ptr::null_mut();
    }

    let channel_id_str = {
        match std::ffi::CStr::from_ptr(channel_id).to_str() {
            Ok(s) => s,
            Err(_) => {
                error::set_last_error(Error::invalid_utf8());
                return std::ptr::null_mut();
            }
        }
    };

    let text_str = {
        match std::ffi::CStr::from_ptr(text).to_str() {
            Ok(s) => s,
            Err(_) => {
                error::set_last_error(Error::invalid_utf8());
                return std::ptr::null_mut();
            }
        }
    };

    let platform = &**handle;

    match runtime::block_on(platform.send_message(channel_id_str, text_str)) {
        Ok(message) => match serde_json::to_string(&message) {
            Ok(json) => match CString::new(json) {
                Ok(c_string) => c_string.into_raw(),
                Err(_) => {
                    error::set_last_error(Error::new(
                        ErrorCode::OutOfMemory,
                        "Failed to allocate string",
                    ));
                    std::ptr::null_mut()
                }
            },
            Err(e) => {
                error::set_last_error(Error::new(
                    ErrorCode::Unknown,
                    format!("Failed to serialize message: {e}"),
                ));
                std::ptr::null_mut()
            }
        },
        Err(e) => {
            error::set_last_error(e);
            std::ptr::null_mut()
        }
    }
}

/// FFI function: Get all channels for the current user
/// Returns a JSON array string of Channel objects
/// The caller must free the returned string using communicator_free_string()
/// Returns NULL on error
#[no_mangle]
///
/// # Safety
/// This function is unsafe because it deals with raw pointers from C.
/// The caller must ensure all pointer arguments are valid.
pub unsafe extern "C" fn communicator_platform_get_channels(handle: PlatformHandle) -> *mut c_char {
    error::clear_last_error();

    if handle.is_null() {
        error::set_last_error(Error::null_pointer());
        return std::ptr::null_mut();
    }

    let platform = &**handle;

    match runtime::block_on(platform.get_channels()) {
        Ok(channels) => match serde_json::to_string(&channels) {
            Ok(json) => match CString::new(json) {
                Ok(c_string) => c_string.into_raw(),
                Err(_) => {
                    error::set_last_error(Error::new(
                        ErrorCode::OutOfMemory,
                        "Failed to allocate string",
                    ));
                    std::ptr::null_mut()
                }
            },
            Err(e) => {
                error::set_last_error(Error::new(
                    ErrorCode::Unknown,
                    format!("Failed to serialize channels: {e}"),
                ));
                std::ptr::null_mut()
            }
        },
        Err(e) => {
            error::set_last_error(e);
            std::ptr::null_mut()
        }
    }
}

/// FFI function: Get a specific channel by ID
/// Returns a JSON string representing the Channel
/// The caller must free the returned string using communicator_free_string()
/// Returns NULL on error
#[no_mangle]
///
/// # Safety
/// This function is unsafe because it deals with raw pointers from C.
/// The caller must ensure all pointer arguments are valid.
pub unsafe extern "C" fn communicator_platform_get_channel(
    handle: PlatformHandle,
    channel_id: *const c_char,
) -> *mut c_char {
    error::clear_last_error();

    if handle.is_null() || channel_id.is_null() {
        error::set_last_error(Error::null_pointer());
        return std::ptr::null_mut();
    }

    let channel_id_str = {
        match std::ffi::CStr::from_ptr(channel_id).to_str() {
            Ok(s) => s,
            Err(_) => {
                error::set_last_error(Error::invalid_utf8());
                return std::ptr::null_mut();
            }
        }
    };

    let platform = &**handle;

    match runtime::block_on(platform.get_channel(channel_id_str)) {
        Ok(channel) => match serde_json::to_string(&channel) {
            Ok(json) => match CString::new(json) {
                Ok(c_string) => c_string.into_raw(),
                Err(_) => {
                    error::set_last_error(Error::new(
                        ErrorCode::OutOfMemory,
                        "Failed to allocate string",
                    ));
                    std::ptr::null_mut()
                }
            },
            Err(e) => {
                error::set_last_error(Error::new(
                    ErrorCode::Unknown,
                    format!("Failed to serialize channel: {e}"),
                ));
                std::ptr::null_mut()
            }
        },
        Err(e) => {
            error::set_last_error(e);
            std::ptr::null_mut()
        }
    }
}

/// FFI function: Get recent messages from a channel
/// Returns a JSON array string of Message objects
/// The caller must free the returned string using communicator_free_string()
/// Returns NULL on error
#[no_mangle]
///
/// # Safety
/// This function is unsafe because it deals with raw pointers from C.
/// The caller must ensure all pointer arguments are valid.
pub unsafe extern "C" fn communicator_platform_get_messages(
    handle: PlatformHandle,
    channel_id: *const c_char,
    limit: u32,
) -> *mut c_char {
    error::clear_last_error();

    if handle.is_null() || channel_id.is_null() {
        error::set_last_error(Error::null_pointer());
        return std::ptr::null_mut();
    }

    let channel_id_str = {
        match std::ffi::CStr::from_ptr(channel_id).to_str() {
            Ok(s) => s,
            Err(_) => {
                error::set_last_error(Error::invalid_utf8());
                return std::ptr::null_mut();
            }
        }
    };

    let platform = &**handle;

    match runtime::block_on(platform.get_messages(channel_id_str, limit as usize)) {
        Ok(messages) => match serde_json::to_string(&messages) {
            Ok(json) => match CString::new(json) {
                Ok(c_string) => c_string.into_raw(),
                Err(_) => {
                    error::set_last_error(Error::new(
                        ErrorCode::OutOfMemory,
                        "Failed to allocate string",
                    ));
                    std::ptr::null_mut()
                }
            },
            Err(e) => {
                error::set_last_error(Error::new(
                    ErrorCode::Unknown,
                    format!("Failed to serialize messages: {e}"),
                ));
                std::ptr::null_mut()
            }
        },
        Err(e) => {
            error::set_last_error(e);
            std::ptr::null_mut()
        }
    }
}

/// FFI function: Get members of a channel
/// Returns a JSON array string of User objects
/// The caller must free the returned string using communicator_free_string()
/// Returns NULL on error
#[no_mangle]
///
/// # Safety
/// This function is unsafe because it deals with raw pointers from C.
/// The caller must ensure all pointer arguments are valid.
pub unsafe extern "C" fn communicator_platform_get_channel_members(
    handle: PlatformHandle,
    channel_id: *const c_char,
) -> *mut c_char {
    error::clear_last_error();

    if handle.is_null() || channel_id.is_null() {
        error::set_last_error(Error::null_pointer());
        return std::ptr::null_mut();
    }

    let channel_id_str = {
        match std::ffi::CStr::from_ptr(channel_id).to_str() {
            Ok(s) => s,
            Err(_) => {
                error::set_last_error(Error::invalid_utf8());
                return std::ptr::null_mut();
            }
        }
    };

    let platform = &**handle;

    match runtime::block_on(platform.get_channel_members(channel_id_str)) {
        Ok(users) => match serde_json::to_string(&users) {
            Ok(json) => match CString::new(json) {
                Ok(c_string) => c_string.into_raw(),
                Err(_) => {
                    error::set_last_error(Error::new(
                        ErrorCode::OutOfMemory,
                        "Failed to allocate string",
                    ));
                    std::ptr::null_mut()
                }
            },
            Err(e) => {
                error::set_last_error(Error::new(
                    ErrorCode::Unknown,
                    format!("Failed to serialize users: {e}"),
                ));
                std::ptr::null_mut()
            }
        },
        Err(e) => {
            error::set_last_error(e);
            std::ptr::null_mut()
        }
    }
}

/// FFI function: Get a specific user by ID
/// Returns a JSON string representing the User
/// The caller must free the returned string using communicator_free_string()
/// Returns NULL on error
#[no_mangle]
///
/// # Safety
/// This function is unsafe because it deals with raw pointers from C.
/// The caller must ensure all pointer arguments are valid.
pub unsafe extern "C" fn communicator_platform_get_user(
    handle: PlatformHandle,
    user_id: *const c_char,
) -> *mut c_char {
    error::clear_last_error();

    if handle.is_null() || user_id.is_null() {
        error::set_last_error(Error::null_pointer());
        return std::ptr::null_mut();
    }

    let user_id_str = {
        match std::ffi::CStr::from_ptr(user_id).to_str() {
            Ok(s) => s,
            Err(_) => {
                error::set_last_error(Error::invalid_utf8());
                return std::ptr::null_mut();
            }
        }
    };

    let platform = &**handle;

    match runtime::block_on(platform.get_user(user_id_str)) {
        Ok(user) => match serde_json::to_string(&user) {
            Ok(json) => match CString::new(json) {
                Ok(c_string) => c_string.into_raw(),
                Err(_) => {
                    error::set_last_error(Error::new(
                        ErrorCode::OutOfMemory,
                        "Failed to allocate string",
                    ));
                    std::ptr::null_mut()
                }
            },
            Err(e) => {
                error::set_last_error(Error::new(
                    ErrorCode::Unknown,
                    format!("Failed to serialize user: {e}"),
                ));
                std::ptr::null_mut()
            }
        },
        Err(e) => {
            error::set_last_error(e);
            std::ptr::null_mut()
        }
    }
}

/// FFI function: Get the current authenticated user
/// Returns a JSON string representing the User
/// The caller must free the returned string using communicator_free_string()
/// Returns NULL on error
#[no_mangle]
///
/// # Safety
/// This function is unsafe because it deals with raw pointers from C.
/// The caller must ensure all pointer arguments are valid.
pub unsafe extern "C" fn communicator_platform_get_current_user(handle: PlatformHandle) -> *mut c_char {
    error::clear_last_error();

    if handle.is_null() {
        error::set_last_error(Error::null_pointer());
        return std::ptr::null_mut();
    }

    let platform = &**handle;

    match runtime::block_on(platform.get_current_user()) {
        Ok(user) => match serde_json::to_string(&user) {
            Ok(json) => match CString::new(json) {
                Ok(c_string) => c_string.into_raw(),
                Err(_) => {
                    error::set_last_error(Error::new(
                        ErrorCode::OutOfMemory,
                        "Failed to allocate string",
                    ));
                    std::ptr::null_mut()
                }
            },
            Err(e) => {
                error::set_last_error(Error::new(
                    ErrorCode::Unknown,
                    format!("Failed to serialize user: {e}"),
                ));
                std::ptr::null_mut()
            }
        },
        Err(e) => {
            error::set_last_error(e);
            std::ptr::null_mut()
        }
    }
}

/// FFI function: Create a direct message channel with another user
/// Returns a JSON string representing the created Channel
/// The caller must free the returned string using communicator_free_string()
/// Returns NULL on error
#[no_mangle]
///
/// # Safety
/// This function is unsafe because it deals with raw pointers from C.
/// The caller must ensure all pointer arguments are valid.
pub unsafe extern "C" fn communicator_platform_create_direct_channel(
    handle: PlatformHandle,
    user_id: *const c_char,
) -> *mut c_char {
    error::clear_last_error();

    if handle.is_null() || user_id.is_null() {
        error::set_last_error(Error::null_pointer());
        return std::ptr::null_mut();
    }

    let user_id_str = {
        match std::ffi::CStr::from_ptr(user_id).to_str() {
            Ok(s) => s,
            Err(_) => {
                error::set_last_error(Error::invalid_utf8());
                return std::ptr::null_mut();
            }
        }
    };

    let platform = &**handle;

    match runtime::block_on(platform.create_direct_channel(user_id_str)) {
        Ok(channel) => match serde_json::to_string(&channel) {
            Ok(json) => match CString::new(json) {
                Ok(c_string) => c_string.into_raw(),
                Err(_) => {
                    error::set_last_error(Error::new(
                        ErrorCode::OutOfMemory,
                        "Failed to allocate string",
                    ));
                    std::ptr::null_mut()
                }
            },
            Err(e) => {
                error::set_last_error(Error::new(
                    ErrorCode::Unknown,
                    format!("Failed to serialize channel: {e}"),
                ));
                std::ptr::null_mut()
            }
        },
        Err(e) => {
            error::set_last_error(e);
            std::ptr::null_mut()
        }
    }
}

/// FFI function: Get all teams the user belongs to
/// Returns a JSON string representing an array of Teams
/// The caller must free the returned string using communicator_free_string()
/// Returns NULL on error
///
/// # Safety
/// The caller must ensure that `handle` is a valid pointer
#[no_mangle]
///
/// # Safety
/// This function is unsafe because it deals with raw pointers from C.
/// The caller must ensure all pointer arguments are valid.
pub unsafe extern "C" fn communicator_platform_get_teams(handle: PlatformHandle) -> *mut c_char {
    error::clear_last_error();

    if handle.is_null() {
        error::set_last_error(Error::null_pointer());
        return std::ptr::null_mut();
    }

    let platform = &**handle;

    match runtime::block_on(platform.get_teams()) {
        Ok(teams) => match serde_json::to_string(&teams) {
            Ok(json) => match CString::new(json) {
                Ok(c_string) => c_string.into_raw(),
                Err(_) => {
                    error::set_last_error(Error::new(
                        ErrorCode::OutOfMemory,
                        "Failed to allocate string",
                    ));
                    std::ptr::null_mut()
                }
            },
            Err(e) => {
                error::set_last_error(Error::new(
                    ErrorCode::Unknown,
                    format!("Failed to serialize teams: {e}"),
                ));
                std::ptr::null_mut()
            }
        },
        Err(e) => {
            error::set_last_error(e);
            std::ptr::null_mut()
        }
    }
}

/// FFI function: Get a specific team by ID
/// Returns a JSON string representing the Team
/// The caller must free the returned string using communicator_free_string()
/// Returns NULL on error
#[no_mangle]
///
/// # Safety
/// This function is unsafe because it deals with raw pointers from C.
/// The caller must ensure all pointer arguments are valid.
pub unsafe extern "C" fn communicator_platform_get_team(
    handle: PlatformHandle,
    team_id: *const c_char,
) -> *mut c_char {
    error::clear_last_error();

    if handle.is_null() || team_id.is_null() {
        error::set_last_error(Error::null_pointer());
        return std::ptr::null_mut();
    }

    let team_id_str = {
        match std::ffi::CStr::from_ptr(team_id).to_str() {
            Ok(s) => s,
            Err(_) => {
                error::set_last_error(Error::invalid_utf8());
                return std::ptr::null_mut();
            }
        }
    };

    let platform = &**handle;

    match runtime::block_on(platform.get_team(team_id_str)) {
        Ok(team) => match serde_json::to_string(&team) {
            Ok(json) => match CString::new(json) {
                Ok(c_string) => c_string.into_raw(),
                Err(_) => {
                    error::set_last_error(Error::new(
                        ErrorCode::OutOfMemory,
                        "Failed to allocate string",
                    ));
                    std::ptr::null_mut()
                }
            },
            Err(e) => {
                error::set_last_error(Error::new(
                    ErrorCode::Unknown,
                    format!("Failed to serialize team: {e}"),
                ));
                std::ptr::null_mut()
            }
        },
        Err(e) => {
            error::set_last_error(e);
            std::ptr::null_mut()
        }
    }
}

/// FFI function: Set the current user's status
/// Returns ErrorCode indicating success or failure
///
/// # Arguments
/// * `handle` - Platform handle
/// * `status` - Status string: "online", "away", "dnd", or "offline"
#[no_mangle]
///
/// # Safety
/// This function is unsafe because it deals with raw pointers from C.
/// The caller must ensure all pointer arguments are valid.
pub unsafe extern "C" fn communicator_platform_set_status(
    handle: PlatformHandle,
    status: *const c_char,
) -> ErrorCode {
    error::clear_last_error();

    if handle.is_null() || status.is_null() {
        error::set_last_error(Error::null_pointer());
        return ErrorCode::NullPointer;
    }

    let status_str = {
        match std::ffi::CStr::from_ptr(status).to_str() {
            Ok(s) => s,
            Err(_) => {
                error::set_last_error(Error::invalid_utf8());
                return ErrorCode::InvalidUtf8;
            }
        }
    };

    // Convert status string to UserStatus
    let user_status = match status_str {
        "online" => crate::types::user::UserStatus::Online,
        "away" => crate::types::user::UserStatus::Away,
        "dnd" => crate::types::user::UserStatus::DoNotDisturb,
        "offline" => crate::types::user::UserStatus::Offline,
        _ => {
            error::set_last_error(Error::new(
                ErrorCode::InvalidArgument,
                "Invalid status. Must be one of: online, away, dnd, offline",
            ));
            return ErrorCode::InvalidArgument;
        }
    };

    let platform = &**handle;

    match runtime::block_on(platform.set_status(user_status, None)) {
        Ok(()) => ErrorCode::Success,
        Err(e) => {
            let code = e.code;
            error::set_last_error(e);
            code
        }
    }
}

/// FFI function: Get a user's status
/// Returns a JSON string representing the status: {"status": "online"}
/// The caller must free the returned string using communicator_free_string()
/// Returns NULL on error
#[no_mangle]
///
/// # Safety
/// This function is unsafe because it deals with raw pointers from C.
/// The caller must ensure all pointer arguments are valid.
pub unsafe extern "C" fn communicator_platform_get_user_status(
    handle: PlatformHandle,
    user_id: *const c_char,
) -> *mut c_char {
    error::clear_last_error();

    if handle.is_null() || user_id.is_null() {
        error::set_last_error(Error::null_pointer());
        return std::ptr::null_mut();
    }

    let user_id_str = {
        match std::ffi::CStr::from_ptr(user_id).to_str() {
            Ok(s) => s,
            Err(_) => {
                error::set_last_error(Error::invalid_utf8());
                return std::ptr::null_mut();
            }
        }
    };

    let platform = &**handle;

    match runtime::block_on(platform.get_user_status(user_id_str)) {
        Ok(status) => {
            // Convert UserStatus to JSON
            let status_str = match status {
                crate::types::user::UserStatus::Online => "online",
                crate::types::user::UserStatus::Away => "away",
                crate::types::user::UserStatus::DoNotDisturb => "dnd",
                crate::types::user::UserStatus::Offline => "offline",
                crate::types::user::UserStatus::Unknown => "unknown",
            };

            let json = serde_json::json!({"status": status_str});

            match serde_json::to_string(&json) {
                Ok(json_str) => match CString::new(json_str) {
                    Ok(c_string) => c_string.into_raw(),
                    Err(_) => {
                        error::set_last_error(Error::new(
                            ErrorCode::OutOfMemory,
                            "Failed to allocate string",
                        ));
                        std::ptr::null_mut()
                    }
                },
                Err(e) => {
                    error::set_last_error(Error::new(
                        ErrorCode::Unknown,
                        format!("Failed to serialize status: {e}"),
                    ));
                    std::ptr::null_mut()
                }
            }
        }
        Err(e) => {
            error::set_last_error(e);
            std::ptr::null_mut()
        }
    }
}

/// FFI function: Send typing indicator to a channel
/// Returns ErrorCode indicating success or failure
///
/// # Arguments
/// * `handle` - Platform handle
/// * `channel_id` - The channel ID to send typing indicator to
/// * `parent_id` - Optional parent post ID for thread typing (pass NULL for regular channel typing)
#[no_mangle]
///
/// # Safety
/// This function is unsafe because it deals with raw pointers from C.
/// The caller must ensure all pointer arguments are valid.
pub unsafe extern "C" fn communicator_platform_send_typing_indicator(
    handle: PlatformHandle,
    channel_id: *const c_char,
    parent_id: *const c_char,
) -> ErrorCode {
    error::clear_last_error();

    if handle.is_null() || channel_id.is_null() {
        error::set_last_error(Error::null_pointer());
        return ErrorCode::NullPointer;
    }

    let channel_id_str = {
        match std::ffi::CStr::from_ptr(channel_id).to_str() {
            Ok(s) => s,
            Err(_) => {
                error::set_last_error(Error::invalid_utf8());
                return ErrorCode::InvalidUtf8;
            }
        }
    };

    // parent_id is optional - NULL is allowed
    let parent_id_str = if parent_id.is_null() {
        None
    } else {
        unsafe {
            match std::ffi::CStr::from_ptr(parent_id).to_str() {
                Ok(s) => {
                    if s.is_empty() {
                        None
                    } else {
                        Some(s)
                    }
                }
                Err(_) => {
                    error::set_last_error(Error::invalid_utf8());
                    return ErrorCode::InvalidUtf8;
                }
            }
        }
    };

    let platform = &**handle;

    match runtime::block_on(platform.send_typing_indicator(channel_id_str, parent_id_str)) {
        Ok(()) => ErrorCode::Success,
        Err(e) => {
            let code = e.code;
            error::set_last_error(e);
            code
        }
    }
}

/// FFI function: Request statuses for all users via WebSocket
/// Returns the sequence number on success, or -1 on error
/// The actual status data will arrive as a Response event with matching seq_reply
#[no_mangle]
///
/// # Safety
/// This function is unsafe because it deals with raw pointers from C.
/// The caller must ensure all pointer arguments are valid.
pub unsafe extern "C" fn communicator_platform_request_all_statuses(
    handle: PlatformHandle
) -> i64 {
    error::clear_last_error();

    if handle.is_null() {
        error::set_last_error(Error::null_pointer());
        return -1;
    }

    let platform = &**handle;

    match runtime::block_on(platform.request_all_statuses()) {
        Ok(seq) => seq,
        Err(e) => {
            error::set_last_error(e);
            -1
        }
    }
}

/// FFI function: Request statuses for specific users via WebSocket
/// Returns the sequence number on success, or -1 on error
/// The actual status data will arrive as a Response event with matching seq_reply
///
/// # Arguments
/// * `handle` - The platform handle
/// * `user_ids_json` - JSON array of user IDs (e.g., ["user1", "user2"])
#[no_mangle]
///
/// # Safety
/// This function is unsafe because it deals with raw pointers from C.
/// The caller must ensure all pointer arguments are valid.
pub unsafe extern "C" fn communicator_platform_request_users_statuses(
    handle: PlatformHandle,
    user_ids_json: *const c_char,
) -> i64 {
    error::clear_last_error();

    if handle.is_null() || user_ids_json.is_null() {
        error::set_last_error(Error::null_pointer());
        return -1;
    }

    let user_ids_json_str = {
        match std::ffi::CStr::from_ptr(user_ids_json).to_str() {
            Ok(s) => s,
            Err(_) => {
                error::set_last_error(Error::invalid_utf8());
                return -1;
            }
        }
    };

    // Parse JSON array of user IDs
    let user_ids: Vec<String> = match serde_json::from_str(user_ids_json_str) {
        Ok(ids) => ids,
        Err(e) => {
            error::set_last_error(Error::new(
                ErrorCode::InvalidArgument,
                format!("Failed to parse user IDs JSON: {}", e),
            ));
            return -1;
        }
    };

    let platform = &**handle;

    match runtime::block_on(platform.request_users_statuses(user_ids)) {
        Ok(seq) => seq,
        Err(e) => {
            error::set_last_error(e);
            -1
        }
    }
}

/// FFI function: Subscribe to real-time events
/// Returns ErrorCode indicating success or failure
#[no_mangle]
///
/// # Safety
/// This function is unsafe because it deals with raw pointers from C.
/// The caller must ensure all pointer arguments are valid.
pub unsafe extern "C" fn communicator_platform_subscribe_events(handle: PlatformHandle) -> ErrorCode {
    error::clear_last_error();

    if handle.is_null() {
        error::set_last_error(Error::null_pointer());
        return ErrorCode::NullPointer;
    }

    let platform = &mut **handle;

    match runtime::block_on(platform.subscribe_events()) {
        Ok(()) => ErrorCode::Success,
        Err(e) => {
            let code = e.code;
            error::set_last_error(e);
            code
        }
    }
}

/// FFI function: Unsubscribe from real-time events
/// Returns ErrorCode indicating success or failure
#[no_mangle]
///
/// # Safety
/// This function is unsafe because it deals with raw pointers from C.
/// The caller must ensure all pointer arguments are valid.
pub unsafe extern "C" fn communicator_platform_unsubscribe_events(handle: PlatformHandle) -> ErrorCode {
    error::clear_last_error();

    if handle.is_null() {
        error::set_last_error(Error::null_pointer());
        return ErrorCode::NullPointer;
    }

    let platform = &mut **handle;

    match runtime::block_on(platform.unsubscribe_events()) {
        Ok(()) => ErrorCode::Success,
        Err(e) => {
            let code = e.code;
            error::set_last_error(e);
            code
        }
    }
}

/// FFI function: Poll for the next event
/// Returns a JSON string representing the PlatformEvent, or NULL if no events are available
/// The caller must free the returned string using communicator_free_string()
/// Returns NULL if no events or on error
#[no_mangle]
///
/// # Safety
/// This function is unsafe because it deals with raw pointers from C.
/// The caller must ensure all pointer arguments are valid.
pub unsafe extern "C" fn communicator_platform_poll_event(handle: PlatformHandle) -> *mut c_char {
    error::clear_last_error();

    if handle.is_null() {
        error::set_last_error(Error::null_pointer());
        return std::ptr::null_mut();
    }

    let platform = &mut **handle;

    match runtime::block_on(platform.poll_event()) {
        Ok(Some(event)) => {
            // Serialize the event to JSON
            // Note: PlatformEvent enum needs custom serialization
            let json = match event {
                PlatformEvent::MessagePosted(msg) => {
                    serde_json::json!({
                        "type": "message_posted",
                        "data": msg
                    })
                }
                PlatformEvent::MessageUpdated(msg) => {
                    serde_json::json!({
                        "type": "message_updated",
                        "data": msg
                    })
                }
                PlatformEvent::MessageDeleted { message_id, channel_id } => {
                    serde_json::json!({
                        "type": "message_deleted",
                        "message_id": message_id,
                        "channel_id": channel_id
                    })
                }
                PlatformEvent::UserStatusChanged { user_id, status } => {
                    serde_json::json!({
                        "type": "user_status_changed",
                        "user_id": user_id,
                        "status": status
                    })
                }
                PlatformEvent::UserTyping { user_id, channel_id } => {
                    serde_json::json!({
                        "type": "user_typing",
                        "user_id": user_id,
                        "channel_id": channel_id
                    })
                }
                PlatformEvent::ChannelCreated(channel) => {
                    serde_json::json!({
                        "type": "channel_created",
                        "data": channel
                    })
                }
                PlatformEvent::ChannelUpdated(channel) => {
                    serde_json::json!({
                        "type": "channel_updated",
                        "data": channel
                    })
                }
                PlatformEvent::ChannelDeleted { channel_id } => {
                    serde_json::json!({
                        "type": "channel_deleted",
                        "channel_id": channel_id
                    })
                }
                PlatformEvent::UserJoinedChannel { user_id, channel_id } => {
                    serde_json::json!({
                        "type": "user_joined_channel",
                        "user_id": user_id,
                        "channel_id": channel_id
                    })
                }
                PlatformEvent::UserLeftChannel { user_id, channel_id } => {
                    serde_json::json!({
                        "type": "user_left_channel",
                        "user_id": user_id,
                        "channel_id": channel_id
                    })
                }
                PlatformEvent::ConnectionStateChanged(state) => {
                    serde_json::json!({
                        "type": "connection_state_changed",
                        "state": state
                    })
                }
                PlatformEvent::ReactionAdded { message_id, user_id, emoji_name, channel_id } => {
                    serde_json::json!({
                        "type": "reaction_added",
                        "message_id": message_id,
                        "user_id": user_id,
                        "emoji_name": emoji_name,
                        "channel_id": channel_id
                    })
                }
                PlatformEvent::ReactionRemoved { message_id, user_id, emoji_name, channel_id } => {
                    serde_json::json!({
                        "type": "reaction_removed",
                        "message_id": message_id,
                        "user_id": user_id,
                        "emoji_name": emoji_name,
                        "channel_id": channel_id
                    })
                }
                PlatformEvent::DirectChannelAdded { channel_id } => {
                    serde_json::json!({
                        "type": "direct_channel_added",
                        "channel_id": channel_id
                    })
                }
                PlatformEvent::GroupChannelAdded { channel_id } => {
                    serde_json::json!({
                        "type": "group_channel_added",
                        "channel_id": channel_id
                    })
                }
                PlatformEvent::PreferenceChanged { category, name, value } => {
                    serde_json::json!({
                        "type": "preference_changed",
                        "category": category,
                        "name": name,
                        "value": value
                    })
                }
                PlatformEvent::EphemeralMessage { message, channel_id } => {
                    serde_json::json!({
                        "type": "ephemeral_message",
                        "message": message,
                        "channel_id": channel_id
                    })
                }
                PlatformEvent::UserAdded { user_id } => {
                    serde_json::json!({
                        "type": "user_added",
                        "user_id": user_id
                    })
                }
                PlatformEvent::UserUpdated { user_id } => {
                    serde_json::json!({
                        "type": "user_updated",
                        "user_id": user_id
                    })
                }
                PlatformEvent::UserRoleUpdated { user_id } => {
                    serde_json::json!({
                        "type": "user_role_updated",
                        "user_id": user_id
                    })
                }
                PlatformEvent::ChannelViewed { user_id, channel_id } => {
                    serde_json::json!({
                        "type": "channel_viewed",
                        "user_id": user_id,
                        "channel_id": channel_id
                    })
                }
                PlatformEvent::ThreadUpdated { thread_id, channel_id } => {
                    serde_json::json!({
                        "type": "thread_updated",
                        "thread_id": thread_id,
                        "channel_id": channel_id
                    })
                }
                PlatformEvent::ThreadReadChanged { thread_id, user_id, channel_id } => {
                    serde_json::json!({
                        "type": "thread_read_changed",
                        "thread_id": thread_id,
                        "user_id": user_id,
                        "channel_id": channel_id
                    })
                }
                PlatformEvent::ThreadFollowChanged { thread_id, user_id, channel_id, following } => {
                    serde_json::json!({
                        "type": "thread_follow_changed",
                        "thread_id": thread_id,
                        "user_id": user_id,
                        "channel_id": channel_id,
                        "following": following
                    })
                }
                PlatformEvent::PostUnread { post_id, channel_id, user_id } => {
                    serde_json::json!({
                        "type": "post_unread",
                        "post_id": post_id,
                        "channel_id": channel_id,
                        "user_id": user_id
                    })
                }
                PlatformEvent::EmojiAdded { emoji_id, emoji_name } => {
                    serde_json::json!({
                        "type": "emoji_added",
                        "emoji_id": emoji_id,
                        "emoji_name": emoji_name
                    })
                }
                PlatformEvent::AddedToTeam { team_id, user_id } => {
                    serde_json::json!({
                        "type": "added_to_team",
                        "team_id": team_id,
                        "user_id": user_id
                    })
                }
                PlatformEvent::LeftTeam { team_id, user_id } => {
                    serde_json::json!({
                        "type": "left_team",
                        "team_id": team_id,
                        "user_id": user_id
                    })
                }
                PlatformEvent::ConfigChanged => {
                    serde_json::json!({
                        "type": "config_changed"
                    })
                }
                PlatformEvent::LicenseChanged => {
                    serde_json::json!({
                        "type": "license_changed"
                    })
                }
                PlatformEvent::ChannelConverted { channel_id } => {
                    serde_json::json!({
                        "type": "channel_converted",
                        "channel_id": channel_id
                    })
                }
                PlatformEvent::ChannelMemberUpdated { channel_id, user_id } => {
                    serde_json::json!({
                        "type": "channel_member_updated",
                        "channel_id": channel_id,
                        "user_id": user_id
                    })
                }
                PlatformEvent::TeamDeleted { team_id } => {
                    serde_json::json!({
                        "type": "team_deleted",
                        "team_id": team_id
                    })
                }
                PlatformEvent::TeamUpdated { team_id } => {
                    serde_json::json!({
                        "type": "team_updated",
                        "team_id": team_id
                    })
                }
                PlatformEvent::MemberRoleUpdated { channel_id, user_id } => {
                    serde_json::json!({
                        "type": "member_role_updated",
                        "channel_id": channel_id,
                        "user_id": user_id
                    })
                }
                PlatformEvent::PluginDisabled { plugin_id } => {
                    serde_json::json!({
                        "type": "plugin_disabled",
                        "plugin_id": plugin_id
                    })
                }
                PlatformEvent::PluginEnabled { plugin_id } => {
                    serde_json::json!({
                        "type": "plugin_enabled",
                        "plugin_id": plugin_id
                    })
                }
                PlatformEvent::PluginStatusesChanged => {
                    serde_json::json!({
                        "type": "plugin_statuses_changed"
                    })
                }
                PlatformEvent::PreferencesDeleted { category, name } => {
                    serde_json::json!({
                        "type": "preferences_deleted",
                        "category": category,
                        "name": name
                    })
                }
                PlatformEvent::Response { status, seq_reply, error } => {
                    serde_json::json!({
                        "type": "response",
                        "status": status,
                        "seq_reply": seq_reply,
                        "error": error
                    })
                }
                PlatformEvent::DialogOpened { dialog_id } => {
                    serde_json::json!({
                        "type": "dialog_opened",
                        "dialog_id": dialog_id
                    })
                }
                PlatformEvent::RoleUpdated { role_id } => {
                    serde_json::json!({
                        "type": "role_updated",
                        "role_id": role_id
                    })
                }
            };

            match serde_json::to_string(&json) {
                Ok(json_str) => match CString::new(json_str) {
                    Ok(c_string) => c_string.into_raw(),
                    Err(_) => {
                        error::set_last_error(Error::new(
                            ErrorCode::OutOfMemory,
                            "Failed to allocate string",
                        ));
                        std::ptr::null_mut()
                    }
                },
                Err(e) => {
                    error::set_last_error(Error::new(
                        ErrorCode::Unknown,
                        format!("Failed to serialize event: {e}"),
                    ));
                    std::ptr::null_mut()
                }
            }
        }
        Ok(None) => {
            // No events available, not an error
            std::ptr::null_mut()
        }
        Err(e) => {
            error::set_last_error(e);
            std::ptr::null_mut()
        }
    }
}

// ============================================================================
// Extended Platform FFI Functions
// ============================================================================

/// FFI function: Send a reply to a message (threaded conversation)
/// Returns a JSON string representing the created Message
/// The caller must free the returned string using communicator_free_string()
/// Returns NULL on error
#[no_mangle]
///
/// # Safety
/// This function is unsafe because it deals with raw pointers from C.
/// The caller must ensure all pointer arguments are valid.
pub unsafe extern "C" fn communicator_platform_send_reply(
    handle: PlatformHandle,
    channel_id: *const c_char,
    text: *const c_char,
    root_id: *const c_char,
) -> *mut c_char {
    error::clear_last_error();

    if handle.is_null() || channel_id.is_null() || text.is_null() || root_id.is_null() {
        error::set_last_error(Error::null_pointer());
        return std::ptr::null_mut();
    }

    let channel_id_str = {
        match std::ffi::CStr::from_ptr(channel_id).to_str() {
            Ok(s) => s,
            Err(_) => {
                error::set_last_error(Error::invalid_utf8());
                return std::ptr::null_mut();
            }
        }
    };

    let text_str = {
        match std::ffi::CStr::from_ptr(text).to_str() {
            Ok(s) => s,
            Err(_) => {
                error::set_last_error(Error::invalid_utf8());
                return std::ptr::null_mut();
            }
        }
    };

    let root_id_str = {
        match std::ffi::CStr::from_ptr(root_id).to_str() {
            Ok(s) => s,
            Err(_) => {
                error::set_last_error(Error::invalid_utf8());
                return std::ptr::null_mut();
            }
        }
    };

    let platform = &**handle;

    match runtime::block_on(platform.send_reply(channel_id_str, text_str, root_id_str)) {
        Ok(message) => match serde_json::to_string(&message) {
            Ok(json) => match CString::new(json) {
                Ok(c_string) => c_string.into_raw(),
                Err(_) => {
                    error::set_last_error(Error::new(
                        ErrorCode::OutOfMemory,
                        "Failed to allocate string",
                    ));
                    std::ptr::null_mut()
                }
            },
            Err(e) => {
                error::set_last_error(Error::new(
                    ErrorCode::Unknown,
                    format!("Failed to serialize message: {e}"),
                ));
                std::ptr::null_mut()
            }
        },
        Err(e) => {
            error::set_last_error(e);
            std::ptr::null_mut()
        }
    }
}

/// FFI function: Update/edit a message
/// Returns a JSON string representing the updated Message
/// The caller must free the returned string using communicator_free_string()
/// Returns NULL on error
#[no_mangle]
///
/// # Safety
/// This function is unsafe because it deals with raw pointers from C.
/// The caller must ensure all pointer arguments are valid.
pub unsafe extern "C" fn communicator_platform_update_message(
    handle: PlatformHandle,
    message_id: *const c_char,
    new_text: *const c_char,
) -> *mut c_char {
    error::clear_last_error();

    if handle.is_null() || message_id.is_null() || new_text.is_null() {
        error::set_last_error(Error::null_pointer());
        return std::ptr::null_mut();
    }

    let message_id_str = {
        match std::ffi::CStr::from_ptr(message_id).to_str() {
            Ok(s) => s,
            Err(_) => {
                error::set_last_error(Error::invalid_utf8());
                return std::ptr::null_mut();
            }
        }
    };

    let text_str = {
        match std::ffi::CStr::from_ptr(new_text).to_str() {
            Ok(s) => s,
            Err(_) => {
                error::set_last_error(Error::invalid_utf8());
                return std::ptr::null_mut();
            }
        }
    };

    let platform = &**handle;

    match runtime::block_on(platform.update_message(message_id_str, text_str)) {
        Ok(message) => match serde_json::to_string(&message) {
            Ok(json) => match CString::new(json) {
                Ok(c_string) => c_string.into_raw(),
                Err(_) => {
                    error::set_last_error(Error::new(
                        ErrorCode::OutOfMemory,
                        "Failed to allocate string",
                    ));
                    std::ptr::null_mut()
                }
            },
            Err(e) => {
                error::set_last_error(Error::new(
                    ErrorCode::Unknown,
                    format!("Failed to serialize message: {e}"),
                ));
                std::ptr::null_mut()
            }
        },
        Err(e) => {
            error::set_last_error(e);
            std::ptr::null_mut()
        }
    }
}

/// FFI function: Delete a message
/// Returns ErrorCode indicating success or failure
#[no_mangle]
///
/// # Safety
/// This function is unsafe because it deals with raw pointers from C.
/// The caller must ensure all pointer arguments are valid.
pub unsafe extern "C" fn communicator_platform_delete_message(
    handle: PlatformHandle,
    message_id: *const c_char,
) -> ErrorCode {
    error::clear_last_error();

    if handle.is_null() || message_id.is_null() {
        error::set_last_error(Error::null_pointer());
        return ErrorCode::NullPointer;
    }

    let message_id_str = {
        match std::ffi::CStr::from_ptr(message_id).to_str() {
            Ok(s) => s,
            Err(_) => {
                error::set_last_error(Error::invalid_utf8());
                return ErrorCode::InvalidUtf8;
            }
        }
    };

    let platform = &**handle;

    match runtime::block_on(platform.delete_message(message_id_str)) {
        Ok(()) => ErrorCode::Success,
        Err(e) => {
            let code = e.code;
            error::set_last_error(e);
            code
        }
    }
}

/// FFI function: Get a specific message by ID
/// Returns a JSON string representing the Message
/// The caller must free the returned string using communicator_free_string()
/// Returns NULL on error
#[no_mangle]
///
/// # Safety
/// This function is unsafe because it deals with raw pointers from C.
/// The caller must ensure all pointer arguments are valid.
pub unsafe extern "C" fn communicator_platform_get_message(
    handle: PlatformHandle,
    message_id: *const c_char,
) -> *mut c_char {
    error::clear_last_error();

    if handle.is_null() || message_id.is_null() {
        error::set_last_error(Error::null_pointer());
        return std::ptr::null_mut();
    }

    let message_id_str = {
        match std::ffi::CStr::from_ptr(message_id).to_str() {
            Ok(s) => s,
            Err(_) => {
                error::set_last_error(Error::invalid_utf8());
                return std::ptr::null_mut();
            }
        }
    };

    let platform = &**handle;

    match runtime::block_on(platform.get_message(message_id_str)) {
        Ok(message) => match serde_json::to_string(&message) {
            Ok(json) => match CString::new(json) {
                Ok(c_string) => c_string.into_raw(),
                Err(_) => {
                    error::set_last_error(Error::new(
                        ErrorCode::OutOfMemory,
                        "Failed to allocate string",
                    ));
                    std::ptr::null_mut()
                }
            },
            Err(e) => {
                error::set_last_error(Error::new(
                    ErrorCode::Unknown,
                    format!("Failed to serialize message: {e}"),
                ));
                std::ptr::null_mut()
            }
        },
        Err(e) => {
            error::set_last_error(e);
            std::ptr::null_mut()
        }
    }
}

/// FFI function: Search for messages
/// Returns a JSON array string of Message objects
/// The caller must free the returned string using communicator_free_string()
/// Returns NULL on error
#[no_mangle]
///
/// # Safety
/// This function is unsafe because it deals with raw pointers from C.
/// The caller must ensure all pointer arguments are valid.
pub unsafe extern "C" fn communicator_platform_search_messages(
    handle: PlatformHandle,
    query: *const c_char,
    limit: u32,
) -> *mut c_char {
    error::clear_last_error();

    if handle.is_null() || query.is_null() {
        error::set_last_error(Error::null_pointer());
        return std::ptr::null_mut();
    }

    let query_str = {
        match std::ffi::CStr::from_ptr(query).to_str() {
            Ok(s) => s,
            Err(_) => {
                error::set_last_error(Error::invalid_utf8());
                return std::ptr::null_mut();
            }
        }
    };

    let platform = &**handle;

    match runtime::block_on(platform.search_messages(query_str, limit as usize)) {
        Ok(messages) => match serde_json::to_string(&messages) {
            Ok(json) => match CString::new(json) {
                Ok(c_string) => c_string.into_raw(),
                Err(_) => {
                    error::set_last_error(Error::new(
                        ErrorCode::OutOfMemory,
                        "Failed to allocate string",
                    ));
                    std::ptr::null_mut()
                }
            },
            Err(e) => {
                error::set_last_error(Error::new(
                    ErrorCode::Unknown,
                    format!("Failed to serialize messages: {e}"),
                ));
                std::ptr::null_mut()
            }
        },
        Err(e) => {
            error::set_last_error(e);
            std::ptr::null_mut()
        }
    }
}

/// FFI function: Get messages before a specific message (pagination)
/// Returns a JSON array string of Message objects
/// The caller must free the returned string using communicator_free_string()
/// Returns NULL on error
#[no_mangle]
///
/// # Safety
/// This function is unsafe because it deals with raw pointers from C.
/// The caller must ensure all pointer arguments are valid.
pub unsafe extern "C" fn communicator_platform_get_messages_before(
    handle: PlatformHandle,
    channel_id: *const c_char,
    before_id: *const c_char,
    limit: u32,
) -> *mut c_char {
    error::clear_last_error();

    if handle.is_null() || channel_id.is_null() || before_id.is_null() {
        error::set_last_error(Error::null_pointer());
        return std::ptr::null_mut();
    }

    let channel_id_str = {
        match std::ffi::CStr::from_ptr(channel_id).to_str() {
            Ok(s) => s,
            Err(_) => {
                error::set_last_error(Error::invalid_utf8());
                return std::ptr::null_mut();
            }
        }
    };

    let before_id_str = {
        match std::ffi::CStr::from_ptr(before_id).to_str() {
            Ok(s) => s,
            Err(_) => {
                error::set_last_error(Error::invalid_utf8());
                return std::ptr::null_mut();
            }
        }
    };

    let platform = &**handle;

    match runtime::block_on(platform.get_messages_before(channel_id_str, before_id_str, limit as usize)) {
        Ok(messages) => match serde_json::to_string(&messages) {
            Ok(json) => match CString::new(json) {
                Ok(c_string) => c_string.into_raw(),
                Err(_) => {
                    error::set_last_error(Error::new(
                        ErrorCode::OutOfMemory,
                        "Failed to allocate string",
                    ));
                    std::ptr::null_mut()
                }
            },
            Err(e) => {
                error::set_last_error(Error::new(
                    ErrorCode::Unknown,
                    format!("Failed to serialize messages: {e}"),
                ));
                std::ptr::null_mut()
            }
        },
        Err(e) => {
            error::set_last_error(e);
            std::ptr::null_mut()
        }
    }
}

/// FFI function: Get messages after a specific message (pagination)
/// Returns a JSON array string of Message objects
/// The caller must free the returned string using communicator_free_string()
/// Returns NULL on error
#[no_mangle]
///
/// # Safety
/// This function is unsafe because it deals with raw pointers from C.
/// The caller must ensure all pointer arguments are valid.
pub unsafe extern "C" fn communicator_platform_get_messages_after(
    handle: PlatformHandle,
    channel_id: *const c_char,
    after_id: *const c_char,
    limit: u32,
) -> *mut c_char {
    error::clear_last_error();

    if handle.is_null() || channel_id.is_null() || after_id.is_null() {
        error::set_last_error(Error::null_pointer());
        return std::ptr::null_mut();
    }

    let channel_id_str = {
        match std::ffi::CStr::from_ptr(channel_id).to_str() {
            Ok(s) => s,
            Err(_) => {
                error::set_last_error(Error::invalid_utf8());
                return std::ptr::null_mut();
            }
        }
    };

    let after_id_str = {
        match std::ffi::CStr::from_ptr(after_id).to_str() {
            Ok(s) => s,
            Err(_) => {
                error::set_last_error(Error::invalid_utf8());
                return std::ptr::null_mut();
            }
        }
    };

    let platform = &**handle;

    match runtime::block_on(platform.get_messages_after(channel_id_str, after_id_str, limit as usize)) {
        Ok(messages) => match serde_json::to_string(&messages) {
            Ok(json) => match CString::new(json) {
                Ok(c_string) => c_string.into_raw(),
                Err(_) => {
                    error::set_last_error(Error::new(
                        ErrorCode::OutOfMemory,
                        "Failed to allocate string",
                    ));
                    std::ptr::null_mut()
                }
            },
            Err(e) => {
                error::set_last_error(Error::new(
                    ErrorCode::Unknown,
                    format!("Failed to serialize messages: {e}"),
                ));
                std::ptr::null_mut()
            }
        },
        Err(e) => {
            error::set_last_error(e);
            std::ptr::null_mut()
        }
    }
}

/// FFI function: Add a reaction to a message
/// Returns error code indicating success or failure
#[no_mangle]
///
/// # Safety
/// This function is unsafe because it deals with raw pointers from C.
/// The caller must ensure all pointer arguments are valid.
pub unsafe extern "C" fn communicator_platform_add_reaction(
    handle: PlatformHandle,
    message_id: *const c_char,
    emoji_name: *const c_char,
) -> ErrorCode {
    error::clear_last_error();

    if handle.is_null() || message_id.is_null() || emoji_name.is_null() {
        error::set_last_error(Error::null_pointer());
        return ErrorCode::NullPointer;
    }

    let message_id_str = {
        match std::ffi::CStr::from_ptr(message_id).to_str() {
            Ok(s) => s,
            Err(_) => {
                error::set_last_error(Error::invalid_utf8());
                return ErrorCode::InvalidUtf8;
            }
        }
    };

    let emoji_name_str = {
        match std::ffi::CStr::from_ptr(emoji_name).to_str() {
            Ok(s) => s,
            Err(_) => {
                error::set_last_error(Error::invalid_utf8());
                return ErrorCode::InvalidUtf8;
            }
        }
    };

    let platform = &**handle;

    match runtime::block_on(platform.add_reaction(message_id_str, emoji_name_str)) {
        Ok(()) => ErrorCode::Success,
        Err(e) => {
            let code = e.code;
            error::set_last_error(e);
            code
        }
    }
}

/// FFI function: Remove a reaction from a message
/// Returns error code indicating success or failure
#[no_mangle]
///
/// # Safety
/// This function is unsafe because it deals with raw pointers from C.
/// The caller must ensure all pointer arguments are valid.
pub unsafe extern "C" fn communicator_platform_remove_reaction(
    handle: PlatformHandle,
    message_id: *const c_char,
    emoji_name: *const c_char,
) -> ErrorCode {
    error::clear_last_error();

    if handle.is_null() || message_id.is_null() || emoji_name.is_null() {
        error::set_last_error(Error::null_pointer());
        return ErrorCode::NullPointer;
    }

    let message_id_str = {
        match std::ffi::CStr::from_ptr(message_id).to_str() {
            Ok(s) => s,
            Err(_) => {
                error::set_last_error(Error::invalid_utf8());
                return ErrorCode::InvalidUtf8;
            }
        }
    };

    let emoji_name_str = {
        match std::ffi::CStr::from_ptr(emoji_name).to_str() {
            Ok(s) => s,
            Err(_) => {
                error::set_last_error(Error::invalid_utf8());
                return ErrorCode::InvalidUtf8;
            }
        }
    };

    let platform = &**handle;

    match runtime::block_on(platform.remove_reaction(message_id_str, emoji_name_str)) {
        Ok(()) => ErrorCode::Success,
        Err(e) => {
            let code = e.code;
            error::set_last_error(e);
            code
        }
    }
}

/// FFI function: Get a list of custom emojis
/// Returns a JSON string representing a Vec<Emoji>
/// The caller must free the returned string using communicator_free_string()
/// Returns NULL on error
#[no_mangle]
///
/// # Safety
/// This function is unsafe because it deals with raw pointers from C.
/// The caller must ensure all pointer arguments are valid.
pub unsafe extern "C" fn communicator_platform_get_emojis(
    handle: PlatformHandle,
    page: u32,
    per_page: u32,
) -> *mut c_char {
    error::clear_last_error();

    if handle.is_null() {
        error::set_last_error(Error::null_pointer());
        return std::ptr::null_mut();
    }

    let platform = &**handle;

    match runtime::block_on(platform.get_emojis(page, per_page)) {
        Ok(emojis) => {
            match serde_json::to_string(&emojis) {
                Ok(json_str) => {
                    match CString::new(json_str) {
                        Ok(c_str) => c_str.into_raw(),
                        Err(_) => {
                            error::set_last_error(Error::invalid_utf8());
                            std::ptr::null_mut()
                        }
                    }
                }
                Err(e) => {
                    error::set_last_error(Error::new(ErrorCode::Unknown, format!("Failed to serialize emojis: {e}")));
                    std::ptr::null_mut()
                }
            }
        }
        Err(e) => {
            error::set_last_error(e);
            std::ptr::null_mut()
        }
    }
}

/// FFI function: Get a channel by name
/// Returns a JSON string representing the Channel
/// The caller must free the returned string using communicator_free_string()
/// Returns NULL on error
#[no_mangle]
///
/// # Safety
/// This function is unsafe because it deals with raw pointers from C.
/// The caller must ensure all pointer arguments are valid.
pub unsafe extern "C" fn communicator_platform_get_channel_by_name(
    handle: PlatformHandle,
    team_id: *const c_char,
    channel_name: *const c_char,
) -> *mut c_char {
    error::clear_last_error();

    if handle.is_null() || team_id.is_null() || channel_name.is_null() {
        error::set_last_error(Error::null_pointer());
        return std::ptr::null_mut();
    }

    let team_id_str = {
        match std::ffi::CStr::from_ptr(team_id).to_str() {
            Ok(s) => s,
            Err(_) => {
                error::set_last_error(Error::invalid_utf8());
                return std::ptr::null_mut();
            }
        }
    };

    let channel_name_str = {
        match std::ffi::CStr::from_ptr(channel_name).to_str() {
            Ok(s) => s,
            Err(_) => {
                error::set_last_error(Error::invalid_utf8());
                return std::ptr::null_mut();
            }
        }
    };

    let platform = &**handle;

    match runtime::block_on(platform.get_channel_by_name(team_id_str, channel_name_str)) {
        Ok(channel) => match serde_json::to_string(&channel) {
            Ok(json) => match CString::new(json) {
                Ok(c_string) => c_string.into_raw(),
                Err(_) => {
                    error::set_last_error(Error::new(
                        ErrorCode::OutOfMemory,
                        "Failed to allocate string",
                    ));
                    std::ptr::null_mut()
                }
            },
            Err(e) => {
                error::set_last_error(Error::new(
                    ErrorCode::Unknown,
                    format!("Failed to serialize channel: {e}"),
                ));
                std::ptr::null_mut()
            }
        },
        Err(e) => {
            error::set_last_error(e);
            std::ptr::null_mut()
        }
    }
}

/// FFI function: Create a group direct message channel
/// user_ids_json: JSON array of user IDs, e.g. ["user1", "user2", "user3"]
/// Returns a JSON string representing the created Channel
/// The caller must free the returned string using communicator_free_string()
/// Returns NULL on error
#[no_mangle]
///
/// # Safety
/// This function is unsafe because it deals with raw pointers from C.
/// The caller must ensure all pointer arguments are valid.
pub unsafe extern "C" fn communicator_platform_create_group_channel(
    handle: PlatformHandle,
    user_ids_json: *const c_char,
) -> *mut c_char {
    error::clear_last_error();

    if handle.is_null() || user_ids_json.is_null() {
        error::set_last_error(Error::null_pointer());
        return std::ptr::null_mut();
    }

    let user_ids_str = {
        match std::ffi::CStr::from_ptr(user_ids_json).to_str() {
            Ok(s) => s,
            Err(_) => {
                error::set_last_error(Error::invalid_utf8());
                return std::ptr::null_mut();
            }
        }
    };

    // Parse JSON array of user IDs
    let user_ids: Vec<String> = match serde_json::from_str(user_ids_str) {
        Ok(ids) => ids,
        Err(e) => {
            error::set_last_error(Error::new(
                ErrorCode::InvalidArgument,
                format!("Invalid user IDs JSON: {e}"),
            ));
            return std::ptr::null_mut();
        }
    };

    let platform = &**handle;

    match runtime::block_on(platform.create_group_channel(user_ids)) {
        Ok(channel) => match serde_json::to_string(&channel) {
            Ok(json) => match CString::new(json) {
                Ok(c_string) => c_string.into_raw(),
                Err(_) => {
                    error::set_last_error(Error::new(
                        ErrorCode::OutOfMemory,
                        "Failed to allocate string",
                    ));
                    std::ptr::null_mut()
                }
            },
            Err(e) => {
                error::set_last_error(Error::new(
                    ErrorCode::Unknown,
                    format!("Failed to serialize channel: {e}"),
                ));
                std::ptr::null_mut()
            }
        },
        Err(e) => {
            error::set_last_error(e);
            std::ptr::null_mut()
        }
    }
}

/// FFI function: Add a user to a channel
/// Returns ErrorCode indicating success or failure
#[no_mangle]
///
/// # Safety
/// This function is unsafe because it deals with raw pointers from C.
/// The caller must ensure all pointer arguments are valid.
pub unsafe extern "C" fn communicator_platform_add_channel_member(
    handle: PlatformHandle,
    channel_id: *const c_char,
    user_id: *const c_char,
) -> ErrorCode {
    error::clear_last_error();

    if handle.is_null() || channel_id.is_null() || user_id.is_null() {
        error::set_last_error(Error::null_pointer());
        return ErrorCode::NullPointer;
    }

    let channel_id_str = {
        match std::ffi::CStr::from_ptr(channel_id).to_str() {
            Ok(s) => s,
            Err(_) => {
                error::set_last_error(Error::invalid_utf8());
                return ErrorCode::InvalidUtf8;
            }
        }
    };

    let user_id_str = {
        match std::ffi::CStr::from_ptr(user_id).to_str() {
            Ok(s) => s,
            Err(_) => {
                error::set_last_error(Error::invalid_utf8());
                return ErrorCode::InvalidUtf8;
            }
        }
    };

    let platform = &**handle;

    match runtime::block_on(platform.add_channel_member(channel_id_str, user_id_str)) {
        Ok(()) => ErrorCode::Success,
        Err(e) => {
            let code = e.code;
            error::set_last_error(e);
            code
        }
    }
}

/// FFI function: Remove a user from a channel
/// Returns ErrorCode indicating success or failure
#[no_mangle]
///
/// # Safety
/// This function is unsafe because it deals with raw pointers from C.
/// The caller must ensure all pointer arguments are valid.
pub unsafe extern "C" fn communicator_platform_remove_channel_member(
    handle: PlatformHandle,
    channel_id: *const c_char,
    user_id: *const c_char,
) -> ErrorCode {
    error::clear_last_error();

    if handle.is_null() || channel_id.is_null() || user_id.is_null() {
        error::set_last_error(Error::null_pointer());
        return ErrorCode::NullPointer;
    }

    let channel_id_str = {
        match std::ffi::CStr::from_ptr(channel_id).to_str() {
            Ok(s) => s,
            Err(_) => {
                error::set_last_error(Error::invalid_utf8());
                return ErrorCode::InvalidUtf8;
            }
        }
    };

    let user_id_str = {
        match std::ffi::CStr::from_ptr(user_id).to_str() {
            Ok(s) => s,
            Err(_) => {
                error::set_last_error(Error::invalid_utf8());
                return ErrorCode::InvalidUtf8;
            }
        }
    };

    let platform = &**handle;

    match runtime::block_on(platform.remove_channel_member(channel_id_str, user_id_str)) {
        Ok(()) => ErrorCode::Success,
        Err(e) => {
            let code = e.code;
            error::set_last_error(e);
            code
        }
    }
}

/// FFI function: Get a user by username
/// Returns a JSON string representing the User
/// The caller must free the returned string using communicator_free_string()
/// Returns NULL on error
#[no_mangle]
///
/// # Safety
/// This function is unsafe because it deals with raw pointers from C.
/// The caller must ensure all pointer arguments are valid.
pub unsafe extern "C" fn communicator_platform_get_user_by_username(
    handle: PlatformHandle,
    username: *const c_char,
) -> *mut c_char {
    error::clear_last_error();

    if handle.is_null() || username.is_null() {
        error::set_last_error(Error::null_pointer());
        return std::ptr::null_mut();
    }

    let username_str = {
        match std::ffi::CStr::from_ptr(username).to_str() {
            Ok(s) => s,
            Err(_) => {
                error::set_last_error(Error::invalid_utf8());
                return std::ptr::null_mut();
            }
        }
    };

    let platform = &**handle;

    match runtime::block_on(platform.get_user_by_username(username_str)) {
        Ok(user) => match serde_json::to_string(&user) {
            Ok(json) => match CString::new(json) {
                Ok(c_string) => c_string.into_raw(),
                Err(_) => {
                    error::set_last_error(Error::new(
                        ErrorCode::OutOfMemory,
                        "Failed to allocate string",
                    ));
                    std::ptr::null_mut()
                }
            },
            Err(e) => {
                error::set_last_error(Error::new(
                    ErrorCode::Unknown,
                    format!("Failed to serialize user: {e}"),
                ));
                std::ptr::null_mut()
            }
        },
        Err(e) => {
            error::set_last_error(e);
            std::ptr::null_mut()
        }
    }
}

/// FFI function: Get a user by email
/// Returns a JSON string representing the User
/// The caller must free the returned string using communicator_free_string()
/// Returns NULL on error
#[no_mangle]
///
/// # Safety
/// This function is unsafe because it deals with raw pointers from C.
/// The caller must ensure all pointer arguments are valid.
pub unsafe extern "C" fn communicator_platform_get_user_by_email(
    handle: PlatformHandle,
    email: *const c_char,
) -> *mut c_char {
    error::clear_last_error();

    if handle.is_null() || email.is_null() {
        error::set_last_error(Error::null_pointer());
        return std::ptr::null_mut();
    }

    let email_str = {
        match std::ffi::CStr::from_ptr(email).to_str() {
            Ok(s) => s,
            Err(_) => {
                error::set_last_error(Error::invalid_utf8());
                return std::ptr::null_mut();
            }
        }
    };

    let platform = &**handle;

    match runtime::block_on(platform.get_user_by_email(email_str)) {
        Ok(user) => match serde_json::to_string(&user) {
            Ok(json) => match CString::new(json) {
                Ok(c_string) => c_string.into_raw(),
                Err(_) => {
                    error::set_last_error(Error::new(
                        ErrorCode::OutOfMemory,
                        "Failed to allocate string",
                    ));
                    std::ptr::null_mut()
                }
            },
            Err(e) => {
                error::set_last_error(Error::new(
                    ErrorCode::Unknown,
                    format!("Failed to serialize user: {e}"),
                ));
                std::ptr::null_mut()
            }
        },
        Err(e) => {
            error::set_last_error(e);
            std::ptr::null_mut()
        }
    }
}

/// FFI function: Get multiple users by their IDs (batch operation)
/// user_ids_json: JSON array of user IDs, e.g. ["user1", "user2", "user3"]
/// Returns a JSON array string of User objects
/// The caller must free the returned string using communicator_free_string()
/// Returns NULL on error
#[no_mangle]
///
/// # Safety
/// This function is unsafe because it deals with raw pointers from C.
/// The caller must ensure all pointer arguments are valid.
pub unsafe extern "C" fn communicator_platform_get_users_by_ids(
    handle: PlatformHandle,
    user_ids_json: *const c_char,
) -> *mut c_char {
    error::clear_last_error();

    if handle.is_null() || user_ids_json.is_null() {
        error::set_last_error(Error::null_pointer());
        return std::ptr::null_mut();
    }

    let user_ids_str = {
        match std::ffi::CStr::from_ptr(user_ids_json).to_str() {
            Ok(s) => s,
            Err(_) => {
                error::set_last_error(Error::invalid_utf8());
                return std::ptr::null_mut();
            }
        }
    };

    // Parse JSON array of user IDs
    let user_ids: Vec<String> = match serde_json::from_str(user_ids_str) {
        Ok(ids) => ids,
        Err(e) => {
            error::set_last_error(Error::new(
                ErrorCode::InvalidArgument,
                format!("Invalid user IDs JSON: {e}"),
            ));
            return std::ptr::null_mut();
        }
    };

    let platform = &**handle;

    match runtime::block_on(platform.get_users_by_ids(user_ids)) {
        Ok(users) => match serde_json::to_string(&users) {
            Ok(json) => match CString::new(json) {
                Ok(c_string) => c_string.into_raw(),
                Err(_) => {
                    error::set_last_error(Error::new(
                        ErrorCode::OutOfMemory,
                        "Failed to allocate string",
                    ));
                    std::ptr::null_mut()
                }
            },
            Err(e) => {
                error::set_last_error(Error::new(
                    ErrorCode::Unknown,
                    format!("Failed to serialize users: {e}"),
                ));
                std::ptr::null_mut()
            }
        },
        Err(e) => {
            error::set_last_error(e);
            std::ptr::null_mut()
        }
    }
}

/// FFI function: Set a custom status message
/// custom_status_json: JSON object with format:
/// {
///   "emoji": "optional-emoji",
///   "text": "status text",
///   "expires_at": 1234567890  // Optional Unix timestamp
/// }
/// Returns ErrorCode indicating success or failure
#[no_mangle]
///
/// # Safety
/// This function is unsafe because it deals with raw pointers from C.
/// The caller must ensure all pointer arguments are valid.
pub unsafe extern "C" fn communicator_platform_set_custom_status(
    handle: PlatformHandle,
    custom_status_json: *const c_char,
) -> ErrorCode {
    error::clear_last_error();

    if handle.is_null() || custom_status_json.is_null() {
        error::set_last_error(Error::null_pointer());
        return ErrorCode::NullPointer;
    }

    let status_str = {
        match std::ffi::CStr::from_ptr(custom_status_json).to_str() {
            Ok(s) => s,
            Err(_) => {
                error::set_last_error(Error::invalid_utf8());
                return ErrorCode::InvalidUtf8;
            }
        }
    };

    // Parse custom status JSON
    #[derive(serde::Deserialize)]
    struct CustomStatusJson {
        emoji: Option<String>,
        text: String,
        expires_at: Option<i64>,
    }

    let status_data: CustomStatusJson = match serde_json::from_str(status_str) {
        Ok(s) => s,
        Err(e) => {
            error::set_last_error(Error::new(
                ErrorCode::InvalidArgument,
                format!("Invalid custom status JSON: {e}"),
            ));
            return ErrorCode::InvalidArgument;
        }
    };

    let platform = &**handle;

    match runtime::block_on(platform.set_custom_status(
        status_data.emoji.as_deref(),
        &status_data.text,
        status_data.expires_at,
    )) {
        Ok(()) => ErrorCode::Success,
        Err(e) => {
            let code = e.code;
            error::set_last_error(e);
            code
        }
    }
}

/// FFI function: Remove/clear the current user's custom status
/// Returns ErrorCode indicating success or failure
#[no_mangle]
///
/// # Safety
/// This function is unsafe because it deals with raw pointers from C.
/// The caller must ensure all pointer arguments are valid.
pub unsafe extern "C" fn communicator_platform_remove_custom_status(handle: PlatformHandle) -> ErrorCode {
    error::clear_last_error();

    if handle.is_null() {
        error::set_last_error(Error::null_pointer());
        return ErrorCode::NullPointer;
    }

    let platform = &**handle;

    match runtime::block_on(platform.remove_custom_status()) {
        Ok(()) => ErrorCode::Success,
        Err(e) => {
            let code = e.code;
            error::set_last_error(e);
            code
        }
    }
}

/// FFI function: Get status for multiple users (batch operation)
/// user_ids_json: JSON array of user IDs, e.g. ["user1", "user2", "user3"]
/// Returns a JSON object mapping user IDs to status strings: {"user1": "online", "user2": "away", ...}
/// The caller must free the returned string using communicator_free_string()
/// Returns NULL on error
#[no_mangle]
///
/// # Safety
/// This function is unsafe because it deals with raw pointers from C.
/// The caller must ensure all pointer arguments are valid.
pub unsafe extern "C" fn communicator_platform_get_users_status(
    handle: PlatformHandle,
    user_ids_json: *const c_char,
) -> *mut c_char {
    error::clear_last_error();

    if handle.is_null() || user_ids_json.is_null() {
        error::set_last_error(Error::null_pointer());
        return std::ptr::null_mut();
    }

    let user_ids_str = {
        match std::ffi::CStr::from_ptr(user_ids_json).to_str() {
            Ok(s) => s,
            Err(_) => {
                error::set_last_error(Error::invalid_utf8());
                return std::ptr::null_mut();
            }
        }
    };

    // Parse JSON array of user IDs
    let user_ids: Vec<String> = match serde_json::from_str(user_ids_str) {
        Ok(ids) => ids,
        Err(e) => {
            error::set_last_error(Error::new(
                ErrorCode::InvalidArgument,
                format!("Invalid user IDs JSON: {e}"),
            ));
            return std::ptr::null_mut();
        }
    };

    let platform = &**handle;

    match runtime::block_on(platform.get_users_status(user_ids)) {
        Ok(status_map) => {
            // Convert UserStatus enum to strings
            let status_strings: std::collections::HashMap<String, String> = status_map
                .into_iter()
                .map(|(id, status)| {
                    let status_str = match status {
                        crate::types::user::UserStatus::Online => "online",
                        crate::types::user::UserStatus::Away => "away",
                        crate::types::user::UserStatus::DoNotDisturb => "dnd",
                        crate::types::user::UserStatus::Offline => "offline",
                        crate::types::user::UserStatus::Unknown => "unknown",
                    };
                    (id, status_str.to_string())
                })
                .collect();

            match serde_json::to_string(&status_strings) {
                Ok(json) => match CString::new(json) {
                    Ok(c_string) => c_string.into_raw(),
                    Err(_) => {
                        error::set_last_error(Error::new(
                            ErrorCode::OutOfMemory,
                            "Failed to allocate string",
                        ));
                        std::ptr::null_mut()
                    }
                },
                Err(e) => {
                    error::set_last_error(Error::new(
                        ErrorCode::Unknown,
                        format!("Failed to serialize status map: {e}"),
                    ));
                    std::ptr::null_mut()
                }
            }
        }
        Err(e) => {
            error::set_last_error(e);
            std::ptr::null_mut()
        }
    }
}

/// FFI function: Get a team by name
/// Returns a JSON string representing the Team
/// The caller must free the returned string using communicator_free_string()
/// Returns NULL on error
///
/// # Safety
/// The caller must ensure that `handle` and `team_name` are valid pointers
#[no_mangle]
///
/// # Safety
/// This function is unsafe because it deals with raw pointers from C.
/// The caller must ensure all pointer arguments are valid.
pub unsafe extern "C" fn communicator_platform_get_team_by_name(
    handle: PlatformHandle,
    team_name: *const c_char,
) -> *mut c_char {
    error::clear_last_error();

    if handle.is_null() || team_name.is_null() {
        error::set_last_error(Error::null_pointer());
        return std::ptr::null_mut();
    }

    let team_name_str = match std::ffi::CStr::from_ptr(team_name).to_str() {
        Ok(s) => s,
        Err(_) => {
            error::set_last_error(Error::invalid_utf8());
            return std::ptr::null_mut();
        }
    };

    let platform = &**handle;

    match runtime::block_on(platform.get_team_by_name(team_name_str)) {
        Ok(team) => match serde_json::to_string(&team) {
            Ok(json) => match CString::new(json) {
                Ok(c_string) => c_string.into_raw(),
                Err(_) => {
                    error::set_last_error(Error::new(
                        ErrorCode::OutOfMemory,
                        "Failed to allocate string",
                    ));
                    std::ptr::null_mut()
                }
            },
            Err(e) => {
                error::set_last_error(Error::new(
                    ErrorCode::Unknown,
                    format!("Failed to serialize team: {e}"),
                ));
                std::ptr::null_mut()
            }
        },
        Err(e) => {
            error::set_last_error(e);
            std::ptr::null_mut()
        }
    }
}

/// FFI function: Set the active team/workspace ID
/// team_id: The team ID to set as active (pass NULL to unset)
/// Returns ErrorCode indicating success or failure
///
/// # Safety
/// The caller must ensure that `handle` is a valid pointer.
/// If `team_id` is not NULL, it must be a valid C string pointer.
#[no_mangle]
///
/// # Safety
/// This function is unsafe because it deals with raw pointers from C.
/// The caller must ensure all pointer arguments are valid.
pub unsafe extern "C" fn communicator_platform_set_team_id(
    handle: PlatformHandle,
    team_id: *const c_char,
) -> ErrorCode {
    error::clear_last_error();

    if handle.is_null() {
        error::set_last_error(Error::null_pointer());
        return ErrorCode::NullPointer;
    }

    // team_id can be NULL (to unset the team ID)
    let team_id_opt = if team_id.is_null() {
        None
    } else {
        let team_id_str = match std::ffi::CStr::from_ptr(team_id).to_str() {
            Ok(s) => s,
            Err(_) => {
                error::set_last_error(Error::invalid_utf8());
                return ErrorCode::InvalidUtf8;
            }
        };
        Some(team_id_str.to_string())
    };

    let platform = &**handle;

    match runtime::block_on(platform.set_team_id(team_id_opt)) {
        Ok(()) => ErrorCode::Success,
        Err(e) => {
            let code = e.code;
            error::set_last_error(e);
            code
        }
    }
}

// ============================================================================
// File Operations FFI Functions
// ============================================================================

/// FFI function: Upload a file to a channel
/// Returns a dynamically allocated string containing the file ID
/// The caller must free the returned string using communicator_free_string()
/// Returns NULL on error
///
/// # Arguments
/// * `handle` - The platform handle
/// * `channel_id` - The channel ID where the file will be uploaded
/// * `file_path` - Path to the file to upload
#[no_mangle]
///
/// # Safety
/// This function is unsafe because it deals with raw pointers from C.
/// The caller must ensure all pointer arguments are valid.
pub unsafe extern "C" fn communicator_platform_upload_file(
    handle: PlatformHandle,
    channel_id: *const c_char,
    file_path: *const c_char,
) -> *mut c_char {
    error::clear_last_error();

    if handle.is_null() || channel_id.is_null() || file_path.is_null() {
        error::set_last_error(Error::null_pointer());
        return std::ptr::null_mut();
    }

    let channel_id_str = {
        match std::ffi::CStr::from_ptr(channel_id).to_str() {
            Ok(s) => s,
            Err(_) => {
                error::set_last_error(Error::invalid_utf8());
                return std::ptr::null_mut();
            }
        }
    };

    let file_path_str = {
        match std::ffi::CStr::from_ptr(file_path).to_str() {
            Ok(s) => s,
            Err(_) => {
                error::set_last_error(Error::invalid_utf8());
                return std::ptr::null_mut();
            }
        }
    };

    let platform = &**handle;
    let path = std::path::Path::new(file_path_str);

    match runtime::block_on(platform.upload_file(channel_id_str, path)) {
        Ok(file_id) => match CString::new(file_id) {
            Ok(c_string) => c_string.into_raw(),
            Err(_) => {
                error::set_last_error(Error::new(
                    ErrorCode::Unknown,
                    "Failed to convert file ID to C string",
                ));
                std::ptr::null_mut()
            }
        },
        Err(e) => {
            error::set_last_error(e);
            std::ptr::null_mut()
        }
    }
}

/// FFI function: Download a file by its ID
/// The file data is returned through the out_data and out_size parameters
/// The caller must free the returned data using communicator_free_file_data()
/// Returns ErrorCode indicating success or failure
///
/// # Arguments
/// * `handle` - The platform handle
/// * `file_id` - The ID of the file to download
/// * `out_data` - Output parameter for the file data (caller must free with communicator_free_file_data)
/// * `out_size` - Output parameter for the size of the file data in bytes
#[no_mangle]
///
/// # Safety
/// This function is unsafe because it deals with raw pointers from C.
/// The caller must ensure all pointer arguments are valid.
pub unsafe extern "C" fn communicator_platform_download_file(
    handle: PlatformHandle,
    file_id: *const c_char,
    out_data: *mut *mut u8,
    out_size: *mut usize,
) -> ErrorCode {
    error::clear_last_error();

    if handle.is_null() || file_id.is_null() || out_data.is_null() || out_size.is_null() {
        error::set_last_error(Error::null_pointer());
        return ErrorCode::NullPointer;
    }

    let file_id_str = {
        match std::ffi::CStr::from_ptr(file_id).to_str() {
            Ok(s) => s,
            Err(_) => {
                error::set_last_error(Error::invalid_utf8());
                return ErrorCode::InvalidUtf8;
            }
        }
    };

    let platform = &**handle;

    match runtime::block_on(platform.download_file(file_id_str)) {
        Ok(data) => {
            let size = data.len();
            let boxed_data = data.into_boxed_slice();
            let raw_ptr = Box::into_raw(boxed_data) as *mut u8;

            *out_data = raw_ptr;
            *out_size = size;
            ErrorCode::Success
        }
        Err(e) => {
            let code = e.code;
            error::set_last_error(e);
            code
        }
    }
}

/// FFI function: Get file metadata without downloading the file
/// Returns a JSON string representing the Attachment metadata
/// The caller must free the returned string using communicator_free_string()
/// Returns NULL on error
///
/// # Arguments
/// * `handle` - The platform handle
/// * `file_id` - The ID of the file
#[no_mangle]
///
/// # Safety
/// This function is unsafe because it deals with raw pointers from C.
/// The caller must ensure all pointer arguments are valid.
pub unsafe extern "C" fn communicator_platform_get_file_metadata(
    handle: PlatformHandle,
    file_id: *const c_char,
) -> *mut c_char {
    error::clear_last_error();

    if handle.is_null() || file_id.is_null() {
        error::set_last_error(Error::null_pointer());
        return std::ptr::null_mut();
    }

    let file_id_str = {
        match std::ffi::CStr::from_ptr(file_id).to_str() {
            Ok(s) => s,
            Err(_) => {
                error::set_last_error(Error::invalid_utf8());
                return std::ptr::null_mut();
            }
        }
    };

    let platform = &**handle;

    match runtime::block_on(platform.get_file_metadata(file_id_str)) {
        Ok(attachment) => match serde_json::to_string(&attachment) {
            Ok(json) => match CString::new(json) {
                Ok(c_string) => c_string.into_raw(),
                Err(_) => {
                    error::set_last_error(Error::new(
                        ErrorCode::Unknown,
                        "Failed to convert metadata to C string",
                    ));
                    std::ptr::null_mut()
                }
            },
            Err(e) => {
                error::set_last_error(Error::new(
                    ErrorCode::Unknown,
                    format!("Failed to serialize metadata: {e}"),
                ));
                std::ptr::null_mut()
            }
        },
        Err(e) => {
            error::set_last_error(e);
            std::ptr::null_mut()
        }
    }
}

/// FFI function: Get file thumbnail
/// The thumbnail data is returned through the out_data and out_size parameters
/// The caller must free the returned data using communicator_free_file_data()
/// Returns ErrorCode indicating success or failure
///
/// # Arguments
/// * `handle` - The platform handle
/// * `file_id` - The ID of the file
/// * `out_data` - Output parameter for the thumbnail data (caller must free with communicator_free_file_data)
/// * `out_size` - Output parameter for the size of the thumbnail data in bytes
#[no_mangle]
///
/// # Safety
/// This function is unsafe because it deals with raw pointers from C.
/// The caller must ensure all pointer arguments are valid.
pub unsafe extern "C" fn communicator_platform_get_file_thumbnail(
    handle: PlatformHandle,
    file_id: *const c_char,
    out_data: *mut *mut u8,
    out_size: *mut usize,
) -> ErrorCode {
    error::clear_last_error();

    if handle.is_null() || file_id.is_null() || out_data.is_null() || out_size.is_null() {
        error::set_last_error(Error::null_pointer());
        return ErrorCode::NullPointer;
    }

    let file_id_str = {
        match std::ffi::CStr::from_ptr(file_id).to_str() {
            Ok(s) => s,
            Err(_) => {
                error::set_last_error(Error::invalid_utf8());
                return ErrorCode::InvalidUtf8;
            }
        }
    };

    let platform = &**handle;

    match runtime::block_on(platform.get_file_thumbnail(file_id_str)) {
        Ok(data) => {
            let size = data.len();
            let boxed_data = data.into_boxed_slice();
            let raw_ptr = Box::into_raw(boxed_data) as *mut u8;

            *out_data = raw_ptr;
            *out_size = size;
            ErrorCode::Success
        }
        Err(e) => {
            let code = e.code;
            error::set_last_error(e);
            code
        }
    }
}

/// FFI function: Free file data allocated by download_file or get_file_thumbnail
///
/// # Arguments
/// * `data` - Pointer to file data returned by communicator_platform_download_file or communicator_platform_get_file_thumbnail
/// * `size` - Size of the data in bytes (as returned in out_size)
///
/// # Safety
/// This function is unsafe because it deals with raw pointers from C.
/// The caller must ensure the data pointer was allocated by this library and has not been freed already.
#[no_mangle]
///
/// # Safety
/// This function is unsafe because it deals with raw pointers from C.
/// The caller must ensure all pointer arguments are valid.
pub unsafe extern "C" fn communicator_free_file_data(data: *mut u8, size: usize) {
    if !data.is_null() && size > 0 {
        let _ = Box::from_raw(std::slice::from_raw_parts_mut(data, size));
    }
}

// ============================================================================
// Thread Operations
// ============================================================================

/// FFI function: Get a thread (root post and all replies)
/// Returns a JSON string containing an array of messages
///
/// # Safety
/// This function is unsafe because it deals with raw pointers from C.
/// The caller must ensure all pointer arguments are valid.
/// The returned string must be freed using communicator_free_string.
#[no_mangle]
pub unsafe extern "C" fn communicator_platform_get_thread(
    handle: PlatformHandle,
    post_id: *const c_char,
) -> *mut c_char {
    error::clear_last_error();

    if handle.is_null() || post_id.is_null() {
        error::set_last_error(Error::null_pointer());
        return std::ptr::null_mut();
    }

    let post_id_str = {
        match std::ffi::CStr::from_ptr(post_id).to_str() {
            Ok(s) => s,
            Err(_) => {
                error::set_last_error(Error::invalid_utf8());
                return std::ptr::null_mut();
            }
        }
    };

    let platform = &**handle;

    match runtime::block_on(platform.get_thread(post_id_str)) {
        Ok(messages) => match serde_json::to_string(&messages) {
            Ok(json) => match CString::new(json) {
                Ok(c_string) => c_string.into_raw(),
                Err(_) => {
                    error::set_last_error(Error::new(
                        ErrorCode::Unknown,
                        "Failed to create C string from thread JSON",
                    ));
                    std::ptr::null_mut()
                }
            },
            Err(e) => {
                error::set_last_error(Error::new(
                    ErrorCode::Unknown,
                    format!("Failed to serialize thread: {e}"),
                ));
                std::ptr::null_mut()
            }
        },
        Err(e) => {
            error::set_last_error(e);
            std::ptr::null_mut()
        }
    }
}

/// FFI function: Start following a thread
/// Returns error code indicating success or failure
///
/// # Safety
/// This function is unsafe because it deals with raw pointers from C.
/// The caller must ensure all pointer arguments are valid.
#[no_mangle]
pub unsafe extern "C" fn communicator_platform_follow_thread(
    handle: PlatformHandle,
    thread_id: *const c_char,
) -> ErrorCode {
    error::clear_last_error();

    if handle.is_null() || thread_id.is_null() {
        error::set_last_error(Error::null_pointer());
        return ErrorCode::NullPointer;
    }

    let thread_id_str = {
        match std::ffi::CStr::from_ptr(thread_id).to_str() {
            Ok(s) => s,
            Err(_) => {
                error::set_last_error(Error::invalid_utf8());
                return ErrorCode::InvalidUtf8;
            }
        }
    };

    let platform = &**handle;

    match runtime::block_on(platform.follow_thread(thread_id_str)) {
        Ok(()) => ErrorCode::Success,
        Err(e) => {
            let code = e.code;
            error::set_last_error(e);
            code
        }
    }
}

/// FFI function: Stop following a thread
/// Returns error code indicating success or failure
///
/// # Safety
/// This function is unsafe because it deals with raw pointers from C.
/// The caller must ensure all pointer arguments are valid.
#[no_mangle]
pub unsafe extern "C" fn communicator_platform_unfollow_thread(
    handle: PlatformHandle,
    thread_id: *const c_char,
) -> ErrorCode {
    error::clear_last_error();

    if handle.is_null() || thread_id.is_null() {
        error::set_last_error(Error::null_pointer());
        return ErrorCode::NullPointer;
    }

    let thread_id_str = {
        match std::ffi::CStr::from_ptr(thread_id).to_str() {
            Ok(s) => s,
            Err(_) => {
                error::set_last_error(Error::invalid_utf8());
                return ErrorCode::InvalidUtf8;
            }
        }
    };

    let platform = &**handle;

    match runtime::block_on(platform.unfollow_thread(thread_id_str)) {
        Ok(()) => ErrorCode::Success,
        Err(e) => {
            let code = e.code;
            error::set_last_error(e);
            code
        }
    }
}

/// FFI function: Mark a thread as read
/// Returns error code indicating success or failure
///
/// # Safety
/// This function is unsafe because it deals with raw pointers from C.
/// The caller must ensure all pointer arguments are valid.
#[no_mangle]
pub unsafe extern "C" fn communicator_platform_mark_thread_read(
    handle: PlatformHandle,
    thread_id: *const c_char,
) -> ErrorCode {
    error::clear_last_error();

    if handle.is_null() || thread_id.is_null() {
        error::set_last_error(Error::null_pointer());
        return ErrorCode::NullPointer;
    }

    let thread_id_str = {
        match std::ffi::CStr::from_ptr(thread_id).to_str() {
            Ok(s) => s,
            Err(_) => {
                error::set_last_error(Error::invalid_utf8());
                return ErrorCode::InvalidUtf8;
            }
        }
    };

    let platform = &**handle;

    match runtime::block_on(platform.mark_thread_read(thread_id_str)) {
        Ok(()) => ErrorCode::Success,
        Err(e) => {
            let code = e.code;
            error::set_last_error(e);
            code
        }
    }
}

/// FFI function: Mark a thread as unread from a specific post
/// Returns error code indicating success or failure
///
/// # Safety
/// This function is unsafe because it deals with raw pointers from C.
/// The caller must ensure all pointer arguments are valid.
#[no_mangle]
pub unsafe extern "C" fn communicator_platform_mark_thread_unread(
    handle: PlatformHandle,
    thread_id: *const c_char,
    post_id: *const c_char,
) -> ErrorCode {
    error::clear_last_error();

    if handle.is_null() || thread_id.is_null() || post_id.is_null() {
        error::set_last_error(Error::null_pointer());
        return ErrorCode::NullPointer;
    }

    let thread_id_str = {
        match std::ffi::CStr::from_ptr(thread_id).to_str() {
            Ok(s) => s,
            Err(_) => {
                error::set_last_error(Error::invalid_utf8());
                return ErrorCode::InvalidUtf8;
            }
        }
    };

    let post_id_str = {
        match std::ffi::CStr::from_ptr(post_id).to_str() {
            Ok(s) => s,
            Err(_) => {
                error::set_last_error(Error::invalid_utf8());
                return ErrorCode::InvalidUtf8;
            }
        }
    };

    let platform = &**handle;

    match runtime::block_on(platform.mark_thread_unread(thread_id_str, post_id_str)) {
        Ok(()) => ErrorCode::Success,
        Err(e) => {
            let code = e.code;
            error::set_last_error(e);
            code
        }
    }
}

// ============================================================================
// Platform Cleanup
// ============================================================================

/// FFI function: Destroy a platform and free its memory
/// After calling this, the handle is invalid and must not be used
///
/// # Safety
/// The caller must ensure that `handle` is a valid pointer that was created by
/// this library and has not been freed already.
#[no_mangle]
///
/// # Safety
/// This function is unsafe because it deals with raw pointers from C.
/// The caller must ensure all pointer arguments are valid.
pub unsafe extern "C" fn communicator_platform_destroy(handle: PlatformHandle) {
    if !handle.is_null() {
        let _ = Box::from_raw(handle);
    }
}

