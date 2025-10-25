//! Async runtime management for FFI
//!
//! This module provides a global Tokio runtime that allows FFI functions
//! to execute async Rust code synchronously from the C perspective.

use std::future::Future;
use std::sync::Mutex;
use tokio::runtime::Runtime;

lazy_static::lazy_static! {
    /// Global Tokio runtime for async operations
    static ref RUNTIME: Mutex<Option<Runtime>> = Mutex::new(None);
}

/// Initialize the async runtime
///
/// This should be called during library initialization.
/// It's safe to call multiple times - subsequent calls are no-ops.
pub fn init_runtime() -> crate::error::Result<()> {
    let mut runtime_guard = RUNTIME.lock().map_err(|_| {
        crate::error::Error::new(
            crate::error::ErrorCode::Unknown,
            "Failed to acquire runtime lock",
        )
    })?;

    if runtime_guard.is_none() {
        let runtime = Runtime::new().map_err(|e| {
            crate::error::Error::new(
                crate::error::ErrorCode::Unknown,
                format!("Failed to create Tokio runtime: {e}"),
            )
        })?;
        *runtime_guard = Some(runtime);
    }

    Ok(())
}

/// Shutdown the async runtime
///
/// This should be called during library cleanup.
/// After calling this, no async operations can be performed until
/// init_runtime is called again.
pub fn shutdown_runtime() {
    if let Ok(mut runtime_guard) = RUNTIME.lock() {
        if let Some(runtime) = runtime_guard.take() {
            runtime.shutdown_timeout(std::time::Duration::from_secs(5));
        }
    }
}

/// Execute an async future synchronously
///
/// This blocks the current thread until the future completes.
/// The runtime must be initialized before calling this function.
///
/// # Panics
/// Panics if the runtime is not initialized
pub fn block_on<F>(future: F) -> F::Output
where
    F: Future + Send,
    F::Output: Send,
{
    let runtime_guard = RUNTIME.lock().expect("Failed to acquire runtime lock");
    let runtime = runtime_guard.as_ref().expect("Runtime not initialized");
    runtime.handle().block_on(future)
}

/// Get a handle to the runtime for spawning background tasks
///
/// Returns None if the runtime is not initialized
pub fn runtime_handle() -> Option<tokio::runtime::Handle> {
    RUNTIME
        .lock()
        .ok()?
        .as_ref()
        .map(|rt| rt.handle().clone())
}

/// Spawn a background task on the runtime
///
/// # Returns
/// A handle to the spawned task, or None if the runtime is not initialized
pub fn spawn<F>(future: F) -> Option<tokio::task::JoinHandle<F::Output>>
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    let handle = runtime_handle()?;
    Some(handle.spawn(future))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_lifecycle() {
        // Initialize runtime
        init_runtime().expect("Failed to initialize runtime");

        // Execute async code
        let result = block_on(async {
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            42
        });
        assert_eq!(result, 42);

        // Shutdown runtime
        shutdown_runtime();
    }

    #[test]
    fn test_runtime_spawn() {
        init_runtime().expect("Failed to initialize runtime");

        let handle = spawn(async {
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            "done"
        });

        assert!(handle.is_some());

        let result = block_on(async { handle.unwrap().await.unwrap() });
        assert_eq!(result, "done");

        shutdown_runtime();
    }

    #[test]
    fn test_multiple_init() {
        // Multiple initializations should be safe
        init_runtime().expect("Failed to initialize runtime");
        init_runtime().expect("Second init should be a no-op");

        shutdown_runtime();
    }
}
