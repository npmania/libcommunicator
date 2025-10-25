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
pub use types::{Attachment, Channel, ChannelType, ConnectionInfo, ConnectionState, Message, Team, TeamType, User};

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
pub extern "C" fn communicator_free_string(s: *mut c_char) {
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
pub extern "C" fn communicator_init() -> ErrorCode {
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
pub extern "C" fn communicator_cleanup() {
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
pub extern "C" fn communicator_version() -> *const c_char {
    concat!(env!("CARGO_PKG_VERSION"), " (libcommunicator)\0").as_ptr() as *const c_char
}

/// FFI function: Get the major version number
#[no_mangle]
pub extern "C" fn communicator_version_major() -> u32 {
    VERSION_MAJOR
}

/// FFI function: Get the minor version number
#[no_mangle]
pub extern "C" fn communicator_version_minor() -> u32 {
    VERSION_MINOR
}

/// FFI function: Get the patch version number
#[no_mangle]
pub extern "C" fn communicator_version_patch() -> u32 {
    VERSION_PATCH
}

// ============================================================================
// Error Handling FFI
// ============================================================================

/// FFI function: Get the error code of the last error
/// Returns ErrorCode::Success (0) if no error has occurred
#[no_mangle]
pub extern "C" fn communicator_last_error_code() -> ErrorCode {
    error::get_last_error()
        .map(|e| e.code)
        .unwrap_or(ErrorCode::Success)
}

/// FFI function: Get the error message of the last error
/// Returns a dynamically allocated string that must be freed with communicator_free_string()
/// Returns NULL if no error has occurred
#[no_mangle]
pub extern "C" fn communicator_last_error_message() -> *mut c_char {
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
pub extern "C" fn communicator_error_code_string(code: ErrorCode) -> *const c_char {
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
    };
    s.as_ptr() as *const c_char
}

/// FFI function: Clear the last error
#[no_mangle]
pub extern "C" fn communicator_clear_error() {
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
pub extern "C" fn communicator_context_create(id: *const c_char) -> ContextHandle {
    error::clear_last_error();

    if id.is_null() {
        error::set_last_error(Error::null_pointer());
        return std::ptr::null_mut();
    }

    let id_str = unsafe {
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
pub extern "C" fn communicator_context_initialize(handle: ContextHandle) -> ErrorCode {
    error::clear_last_error();

    if handle.is_null() {
        error::set_last_error(Error::null_pointer());
        return ErrorCode::NullPointer;
    }

    let context = unsafe { &mut *handle };

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
pub extern "C" fn communicator_context_is_initialized(handle: ContextHandle) -> i32 {
    error::clear_last_error();

    if handle.is_null() {
        error::set_last_error(Error::null_pointer());
        return -1;
    }

    let context = unsafe { &*handle };
    if context.is_initialized() { 1 } else { 0 }
}

/// FFI function: Set a configuration value on a context
/// Returns ErrorCode indicating success or failure
#[no_mangle]
pub extern "C" fn communicator_context_set_config(
    handle: ContextHandle,
    key: *const c_char,
    value: *const c_char,
) -> ErrorCode {
    error::clear_last_error();

    if handle.is_null() || key.is_null() || value.is_null() {
        error::set_last_error(Error::null_pointer());
        return ErrorCode::NullPointer;
    }

    let key_str = unsafe {
        match std::ffi::CStr::from_ptr(key).to_str() {
            Ok(s) => s,
            Err(_) => {
                error::set_last_error(Error::invalid_utf8());
                return ErrorCode::InvalidUtf8;
            }
        }
    };

    let value_str = unsafe {
        match std::ffi::CStr::from_ptr(value).to_str() {
            Ok(s) => s,
            Err(_) => {
                error::set_last_error(Error::invalid_utf8());
                return ErrorCode::InvalidUtf8;
            }
        }
    };

    let context = unsafe { &mut *handle };
    context.set_config(key_str, value_str);
    ErrorCode::Success
}

/// FFI function: Get a configuration value from a context
/// Returns a dynamically allocated string that must be freed with communicator_free_string()
/// Returns NULL if the key doesn't exist or on error
#[no_mangle]
pub extern "C" fn communicator_context_get_config(
    handle: ContextHandle,
    key: *const c_char,
) -> *mut c_char {
    error::clear_last_error();

    if handle.is_null() || key.is_null() {
        error::set_last_error(Error::null_pointer());
        return std::ptr::null_mut();
    }

    let key_str = unsafe {
        match std::ffi::CStr::from_ptr(key).to_str() {
            Ok(s) => s,
            Err(_) => {
                error::set_last_error(Error::invalid_utf8());
                return std::ptr::null_mut();
            }
        }
    };

    let context = unsafe { &*handle };

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
pub extern "C" fn communicator_context_shutdown(handle: ContextHandle) -> ErrorCode {
    error::clear_last_error();

    if handle.is_null() {
        error::set_last_error(Error::null_pointer());
        return ErrorCode::NullPointer;
    }

    let context = unsafe { &mut *handle };

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
pub extern "C" fn communicator_context_destroy(handle: ContextHandle) {
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
pub extern "C" fn communicator_context_set_log_callback(
    handle: ContextHandle,
    callback: LogCallback,
    user_data: *mut c_void,
) -> ErrorCode {
    error::clear_last_error();

    if handle.is_null() {
        error::set_last_error(Error::null_pointer());
        return ErrorCode::NullPointer;
    }

    let context = unsafe { &mut *handle };
    context.set_log_callback(callback, user_data);
    ErrorCode::Success
}

/// FFI function: Clear the log callback on a context
#[no_mangle]
pub extern "C" fn communicator_context_clear_log_callback(handle: ContextHandle) -> ErrorCode {
    error::clear_last_error();

    if handle.is_null() {
        error::set_last_error(Error::null_pointer());
        return ErrorCode::NullPointer;
    }

    let context = unsafe { &mut *handle };
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
pub extern "C" fn communicator_mattermost_create(server_url: *const c_char) -> PlatformHandle {
    error::clear_last_error();

    if server_url.is_null() {
        error::set_last_error(Error::null_pointer());
        return std::ptr::null_mut();
    }

    let url_str = unsafe {
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
            let code = e.code;
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
pub extern "C" fn communicator_platform_connect(
    handle: PlatformHandle,
    config_json: *const c_char,
) -> ErrorCode {
    error::clear_last_error();

    if handle.is_null() || config_json.is_null() {
        error::set_last_error(Error::null_pointer());
        return ErrorCode::NullPointer;
    }

    let config_str = unsafe {
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
                &format!("Invalid config JSON: {}", e),
            ));
            return ErrorCode::InvalidArgument;
        }
    };

    let mut platform_config = PlatformConfig::new(config_data.server);
    platform_config.credentials = config_data.credentials;
    platform_config.team_id = config_data.team_id;

    let platform = unsafe { &mut **handle };

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
pub extern "C" fn communicator_platform_disconnect(handle: PlatformHandle) -> ErrorCode {
    error::clear_last_error();

    if handle.is_null() {
        error::set_last_error(Error::null_pointer());
        return ErrorCode::NullPointer;
    }

    let platform = unsafe { &mut **handle };

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
pub extern "C" fn communicator_platform_is_connected(handle: PlatformHandle) -> i32 {
    error::clear_last_error();

    if handle.is_null() {
        error::set_last_error(Error::null_pointer());
        return -1;
    }

    let platform = unsafe { &**handle };
    if platform.is_connected() { 1 } else { 0 }
}

/// FFI function: Get connection info as JSON
/// Returns a dynamically allocated JSON string that must be freed with communicator_free_string()
/// Returns NULL on error or if not connected
#[no_mangle]
pub extern "C" fn communicator_platform_get_connection_info(
    handle: PlatformHandle,
) -> *mut c_char {
    error::clear_last_error();

    if handle.is_null() {
        error::set_last_error(Error::null_pointer());
        return std::ptr::null_mut();
    }

    let platform = unsafe { &**handle };

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
                    &format!("Failed to serialize connection info: {}", e),
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
pub extern "C" fn communicator_platform_send_message(
    handle: PlatformHandle,
    channel_id: *const c_char,
    text: *const c_char,
) -> *mut c_char {
    error::clear_last_error();

    if handle.is_null() || channel_id.is_null() || text.is_null() {
        error::set_last_error(Error::null_pointer());
        return std::ptr::null_mut();
    }

    let channel_id_str = unsafe {
        match std::ffi::CStr::from_ptr(channel_id).to_str() {
            Ok(s) => s,
            Err(_) => {
                error::set_last_error(Error::invalid_utf8());
                return std::ptr::null_mut();
            }
        }
    };

    let text_str = unsafe {
        match std::ffi::CStr::from_ptr(text).to_str() {
            Ok(s) => s,
            Err(_) => {
                error::set_last_error(Error::invalid_utf8());
                return std::ptr::null_mut();
            }
        }
    };

    let platform = unsafe { &**handle };

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
                    &format!("Failed to serialize message: {}", e),
                ));
                std::ptr::null_mut()
            }
        },
        Err(e) => {
            let code = e.code;
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
pub extern "C" fn communicator_platform_get_channels(handle: PlatformHandle) -> *mut c_char {
    error::clear_last_error();

    if handle.is_null() {
        error::set_last_error(Error::null_pointer());
        return std::ptr::null_mut();
    }

    let platform = unsafe { &**handle };

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
                    &format!("Failed to serialize channels: {}", e),
                ));
                std::ptr::null_mut()
            }
        },
        Err(e) => {
            let code = e.code;
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
pub extern "C" fn communicator_platform_get_channel(
    handle: PlatformHandle,
    channel_id: *const c_char,
) -> *mut c_char {
    error::clear_last_error();

    if handle.is_null() || channel_id.is_null() {
        error::set_last_error(Error::null_pointer());
        return std::ptr::null_mut();
    }

    let channel_id_str = unsafe {
        match std::ffi::CStr::from_ptr(channel_id).to_str() {
            Ok(s) => s,
            Err(_) => {
                error::set_last_error(Error::invalid_utf8());
                return std::ptr::null_mut();
            }
        }
    };

    let platform = unsafe { &**handle };

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
                    &format!("Failed to serialize channel: {}", e),
                ));
                std::ptr::null_mut()
            }
        },
        Err(e) => {
            let code = e.code;
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
pub extern "C" fn communicator_platform_get_messages(
    handle: PlatformHandle,
    channel_id: *const c_char,
    limit: u32,
) -> *mut c_char {
    error::clear_last_error();

    if handle.is_null() || channel_id.is_null() {
        error::set_last_error(Error::null_pointer());
        return std::ptr::null_mut();
    }

    let channel_id_str = unsafe {
        match std::ffi::CStr::from_ptr(channel_id).to_str() {
            Ok(s) => s,
            Err(_) => {
                error::set_last_error(Error::invalid_utf8());
                return std::ptr::null_mut();
            }
        }
    };

    let platform = unsafe { &**handle };

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
                    &format!("Failed to serialize messages: {}", e),
                ));
                std::ptr::null_mut()
            }
        },
        Err(e) => {
            let code = e.code;
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
pub extern "C" fn communicator_platform_get_channel_members(
    handle: PlatformHandle,
    channel_id: *const c_char,
) -> *mut c_char {
    error::clear_last_error();

    if handle.is_null() || channel_id.is_null() {
        error::set_last_error(Error::null_pointer());
        return std::ptr::null_mut();
    }

    let channel_id_str = unsafe {
        match std::ffi::CStr::from_ptr(channel_id).to_str() {
            Ok(s) => s,
            Err(_) => {
                error::set_last_error(Error::invalid_utf8());
                return std::ptr::null_mut();
            }
        }
    };

    let platform = unsafe { &**handle };

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
                    &format!("Failed to serialize users: {}", e),
                ));
                std::ptr::null_mut()
            }
        },
        Err(e) => {
            let code = e.code;
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
pub extern "C" fn communicator_platform_get_user(
    handle: PlatformHandle,
    user_id: *const c_char,
) -> *mut c_char {
    error::clear_last_error();

    if handle.is_null() || user_id.is_null() {
        error::set_last_error(Error::null_pointer());
        return std::ptr::null_mut();
    }

    let user_id_str = unsafe {
        match std::ffi::CStr::from_ptr(user_id).to_str() {
            Ok(s) => s,
            Err(_) => {
                error::set_last_error(Error::invalid_utf8());
                return std::ptr::null_mut();
            }
        }
    };

    let platform = unsafe { &**handle };

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
                    &format!("Failed to serialize user: {}", e),
                ));
                std::ptr::null_mut()
            }
        },
        Err(e) => {
            let code = e.code;
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
pub extern "C" fn communicator_platform_get_current_user(handle: PlatformHandle) -> *mut c_char {
    error::clear_last_error();

    if handle.is_null() {
        error::set_last_error(Error::null_pointer());
        return std::ptr::null_mut();
    }

    let platform = unsafe { &**handle };

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
                    &format!("Failed to serialize user: {}", e),
                ));
                std::ptr::null_mut()
            }
        },
        Err(e) => {
            let code = e.code;
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
pub extern "C" fn communicator_platform_create_direct_channel(
    handle: PlatformHandle,
    user_id: *const c_char,
) -> *mut c_char {
    error::clear_last_error();

    if handle.is_null() || user_id.is_null() {
        error::set_last_error(Error::null_pointer());
        return std::ptr::null_mut();
    }

    let user_id_str = unsafe {
        match std::ffi::CStr::from_ptr(user_id).to_str() {
            Ok(s) => s,
            Err(_) => {
                error::set_last_error(Error::invalid_utf8());
                return std::ptr::null_mut();
            }
        }
    };

    let platform = unsafe { &**handle };

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
                    &format!("Failed to serialize channel: {}", e),
                ));
                std::ptr::null_mut()
            }
        },
        Err(e) => {
            let code = e.code;
            error::set_last_error(e);
            std::ptr::null_mut()
        }
    }
}

/// FFI function: Get all teams the user belongs to
/// Returns a JSON string representing an array of Teams
/// The caller must free the returned string using communicator_free_string()
/// Returns NULL on error
#[no_mangle]
pub extern "C" fn communicator_platform_get_teams(handle: PlatformHandle) -> *mut c_char {
    error::clear_last_error();

    if handle.is_null() {
        error::set_last_error(Error::null_pointer());
        return std::ptr::null_mut();
    }

    let platform = unsafe { &**handle };

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
                    &format!("Failed to serialize teams: {}", e),
                ));
                std::ptr::null_mut()
            }
        },
        Err(e) => {
            let code = e.code;
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
pub extern "C" fn communicator_platform_get_team(
    handle: PlatformHandle,
    team_id: *const c_char,
) -> *mut c_char {
    error::clear_last_error();

    if handle.is_null() || team_id.is_null() {
        error::set_last_error(Error::null_pointer());
        return std::ptr::null_mut();
    }

    let team_id_str = unsafe {
        match std::ffi::CStr::from_ptr(team_id).to_str() {
            Ok(s) => s,
            Err(_) => {
                error::set_last_error(Error::invalid_utf8());
                return std::ptr::null_mut();
            }
        }
    };

    let platform = unsafe { &**handle };

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
                    &format!("Failed to serialize team: {}", e),
                ));
                std::ptr::null_mut()
            }
        },
        Err(e) => {
            let code = e.code;
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
pub extern "C" fn communicator_platform_set_status(
    handle: PlatformHandle,
    status: *const c_char,
) -> ErrorCode {
    error::clear_last_error();

    if handle.is_null() || status.is_null() {
        error::set_last_error(Error::null_pointer());
        return ErrorCode::NullPointer;
    }

    let status_str = unsafe {
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

    let platform = unsafe { &**handle };

    match runtime::block_on(platform.set_status(user_status)) {
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
pub extern "C" fn communicator_platform_get_user_status(
    handle: PlatformHandle,
    user_id: *const c_char,
) -> *mut c_char {
    error::clear_last_error();

    if handle.is_null() || user_id.is_null() {
        error::set_last_error(Error::null_pointer());
        return std::ptr::null_mut();
    }

    let user_id_str = unsafe {
        match std::ffi::CStr::from_ptr(user_id).to_str() {
            Ok(s) => s,
            Err(_) => {
                error::set_last_error(Error::invalid_utf8());
                return std::ptr::null_mut();
            }
        }
    };

    let platform = unsafe { &**handle };

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
                        &format!("Failed to serialize status: {}", e),
                    ));
                    std::ptr::null_mut()
                }
            }
        }
        Err(e) => {
            let _code = e.code;
            error::set_last_error(e);
            std::ptr::null_mut()
        }
    }
}

/// FFI function: Subscribe to real-time events
/// Returns ErrorCode indicating success or failure
#[no_mangle]
pub extern "C" fn communicator_platform_subscribe_events(handle: PlatformHandle) -> ErrorCode {
    error::clear_last_error();

    if handle.is_null() {
        error::set_last_error(Error::null_pointer());
        return ErrorCode::NullPointer;
    }

    let platform = unsafe { &mut **handle };

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
pub extern "C" fn communicator_platform_unsubscribe_events(handle: PlatformHandle) -> ErrorCode {
    error::clear_last_error();

    if handle.is_null() {
        error::set_last_error(Error::null_pointer());
        return ErrorCode::NullPointer;
    }

    let platform = unsafe { &mut **handle };

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
pub extern "C" fn communicator_platform_poll_event(handle: PlatformHandle) -> *mut c_char {
    error::clear_last_error();

    if handle.is_null() {
        error::set_last_error(Error::null_pointer());
        return std::ptr::null_mut();
    }

    let platform = unsafe { &mut **handle };

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
                        &format!("Failed to serialize event: {}", e),
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
            let code = e.code;
            error::set_last_error(e);
            std::ptr::null_mut()
        }
    }
}

/// FFI function: Destroy a platform and free its memory
/// After calling this, the handle is invalid and must not be used
#[no_mangle]
pub extern "C" fn communicator_platform_destroy(handle: PlatformHandle) {
    if !handle.is_null() {
        unsafe {
            let _ = Box::from_raw(handle);
        }
    }
}

