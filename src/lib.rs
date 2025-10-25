use std::ffi::CString;
use std::os::raw::{c_char, c_void};

// Core modules
pub mod context;
pub mod error;
pub mod platforms;

// Re-exports for convenience
pub use context::{Context, LogCallback, LogLevel};
pub use error::{Error, ErrorCode, Result};

// Library version information
pub const VERSION_MAJOR: u32 = 0;
pub const VERSION_MINOR: u32 = 1;
pub const VERSION_PATCH: u32 = 0;
pub const VERSION_STRING: &str = concat!(
    env!("CARGO_PKG_VERSION"),
    " (libcommunicator)"
);

/// Internal Rust function
pub fn greet(name: &str) -> String {
    format!("Hello from libcommunicator, {}!", name)
}

/// FFI function: Get a greeting message
/// The caller must free the returned string using communicator_free_string
/// Returns NULL on error; use communicator_last_error_* to get error details
#[no_mangle]
pub extern "C" fn communicator_greet(name: *const c_char) -> *mut c_char {
    error::clear_last_error();

    if name.is_null() {
        error::set_last_error(Error::null_pointer());
        return std::ptr::null_mut();
    }

    let name_str = unsafe {
        match std::ffi::CStr::from_ptr(name).to_str() {
            Ok(s) => s,
            Err(_) => {
                error::set_last_error(Error::invalid_utf8());
                return std::ptr::null_mut();
            }
        }
    };

    let greeting = greet(name_str);

    match CString::new(greeting) {
        Ok(c_string) => c_string.into_raw(),
        Err(_) => {
            error::set_last_error(Error::new(
                ErrorCode::OutOfMemory,
                "Failed to allocate string",
            ));
            std::ptr::null_mut()
        }
    }
}

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
    // In a real implementation, this might:
    // - Initialize logging systems
    // - Set up thread pools
    // - Register signal handlers
    // - Load configuration files
    // For now, it's a no-op that always succeeds
    ErrorCode::Success
}

/// FFI function: Cleanup the library
/// This should be called once when done using the library
/// Frees any global resources allocated by the library
#[no_mangle]
pub extern "C" fn communicator_cleanup() {
    error::clear_last_error();
    // In a real implementation, this might:
    // - Flush and close log files
    // - Shutdown thread pools
    // - Free global caches
    // - Disconnect any remaining connections
    // For now, it's a no-op
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_greet() {
        let result = greet("World");
        assert_eq!(result, "Hello from libcommunicator, World!");
    }
}
