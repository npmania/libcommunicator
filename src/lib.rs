use std::ffi::CString;
use std::os::raw::c_char;

// Platform-specific implementations
pub mod platforms;

/// Internal Rust function
pub fn greet(name: &str) -> String {
    format!("Hello from libcommunicator, {}!", name)
}

/// FFI function: Get a greeting message
/// The caller must free the returned string using communicator_free_string
#[no_mangle]
pub extern "C" fn communicator_greet(name: *const c_char) -> *mut c_char {
    if name.is_null() {
        return std::ptr::null_mut();
    }

    let name_str = unsafe {
        match std::ffi::CStr::from_ptr(name).to_str() {
            Ok(s) => s,
            Err(_) => return std::ptr::null_mut(),
        }
    };

    let greeting = greet(name_str);

    match CString::new(greeting) {
        Ok(c_string) => c_string.into_raw(),
        Err(_) => std::ptr::null_mut(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_greet() {
        let result = greet("World");
        assert_eq!(result, "Hello from libcommunicator, World!");
    }
}
