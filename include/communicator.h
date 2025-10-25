#ifndef COMMUNICATOR_H
#define COMMUNICATOR_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

// ============================================================================
// Error Handling
// ============================================================================

/**
 * Error codes returned by library functions
 */
typedef enum {
    COMMUNICATOR_SUCCESS = 0,
    COMMUNICATOR_ERROR_UNKNOWN = 1,
    COMMUNICATOR_ERROR_INVALID_ARGUMENT = 2,
    COMMUNICATOR_ERROR_NULL_POINTER = 3,
    COMMUNICATOR_ERROR_OUT_OF_MEMORY = 4,
    COMMUNICATOR_ERROR_INVALID_UTF8 = 5,
    COMMUNICATOR_ERROR_NETWORK = 6,
    COMMUNICATOR_ERROR_AUTH_FAILED = 7,
    COMMUNICATOR_ERROR_NOT_FOUND = 8,
    COMMUNICATOR_ERROR_PERMISSION_DENIED = 9,
    COMMUNICATOR_ERROR_TIMEOUT = 10,
    COMMUNICATOR_ERROR_INVALID_STATE = 11,
} CommunicatorErrorCode;

/**
 * Get the error code of the last error
 *
 * @return The error code, or COMMUNICATOR_SUCCESS if no error occurred
 */
CommunicatorErrorCode communicator_last_error_code(void);

/**
 * Get the error message of the last error
 *
 * @return A dynamically allocated string that must be freed with communicator_free_string()
 *         Returns NULL if no error has occurred
 */
char* communicator_last_error_message(void);

/**
 * Get a human-readable description of an error code
 *
 * @param code The error code
 * @return A static string describing the error (do NOT free this pointer)
 */
const char* communicator_error_code_string(CommunicatorErrorCode code);

/**
 * Clear the last error
 */
void communicator_clear_error(void);

// ============================================================================
// Library Initialization
// ============================================================================

/**
 * Initialize the library
 * This should be called once before using any other library functions
 *
 * @return Error code indicating success or failure
 */
CommunicatorErrorCode communicator_init(void);

/**
 * Cleanup the library
 * This should be called once when done using the library
 * Frees any global resources allocated by the library
 */
void communicator_cleanup(void);

// ============================================================================
// Version Information
// ============================================================================

/**
 * Get the library version string
 *
 * @return A static string containing the version (e.g., "0.1.0 (libcommunicator)")
 *         Do NOT free this pointer
 */
const char* communicator_version(void);

/**
 * Get the major version number
 *
 * @return The major version number
 */
uint32_t communicator_version_major(void);

/**
 * Get the minor version number
 *
 * @return The minor version number
 */
uint32_t communicator_version_minor(void);

/**
 * Get the patch version number
 *
 * @return The patch version number
 */
uint32_t communicator_version_patch(void);

// ============================================================================
// Context Management (Opaque Handle Pattern)
// ============================================================================

/**
 * Opaque handle to a Context object
 */
typedef void* CommunicatorContext;

/**
 * Create a new context
 *
 * @param id A unique identifier for this context
 * @return An opaque handle to the context, or NULL on error
 *         Must be freed with communicator_context_destroy()
 */
CommunicatorContext communicator_context_create(const char* id);

/**
 * Initialize a context
 *
 * @param handle The context handle
 * @return Error code indicating success or failure
 */
CommunicatorErrorCode communicator_context_initialize(CommunicatorContext handle);

/**
 * Check if a context is initialized
 *
 * @param handle The context handle
 * @return 1 if initialized, 0 if not, -1 on error
 */
int communicator_context_is_initialized(CommunicatorContext handle);

/**
 * Set a configuration value on a context
 *
 * @param handle The context handle
 * @param key The configuration key
 * @param value The configuration value
 * @return Error code indicating success or failure
 */
CommunicatorErrorCode communicator_context_set_config(
    CommunicatorContext handle,
    const char* key,
    const char* value
);

/**
 * Get a configuration value from a context
 *
 * @param handle The context handle
 * @param key The configuration key
 * @return A dynamically allocated string that must be freed with communicator_free_string()
 *         Returns NULL if the key doesn't exist or on error
 */
char* communicator_context_get_config(CommunicatorContext handle, const char* key);

/**
 * Shutdown a context
 *
 * @param handle The context handle
 * @return Error code indicating success or failure
 */
CommunicatorErrorCode communicator_context_shutdown(CommunicatorContext handle);

/**
 * Destroy a context and free its memory
 * After calling this, the handle is invalid and must not be used
 *
 * @param handle The context handle
 */
void communicator_context_destroy(CommunicatorContext handle);

// ============================================================================
// Callbacks (Function Pointer Pattern)
// ============================================================================

/**
 * Log levels for callbacks
 */
typedef enum {
    COMMUNICATOR_LOG_DEBUG = 0,
    COMMUNICATOR_LOG_INFO = 1,
    COMMUNICATOR_LOG_WARNING = 2,
    COMMUNICATOR_LOG_ERROR = 3,
} CommunicatorLogLevel;

/**
 * Log callback function type
 *
 * @param level The log level
 * @param message The log message (do NOT free this pointer)
 * @param user_data Opaque user data passed to the callback
 */
typedef void (*CommunicatorLogCallback)(
    CommunicatorLogLevel level,
    const char* message,
    void* user_data
);

/**
 * Set a log callback on a context
 *
 * @param handle The context handle
 * @param callback The callback function
 * @param user_data Opaque pointer passed back to the callback
 * @return Error code indicating success or failure
 */
CommunicatorErrorCode communicator_context_set_log_callback(
    CommunicatorContext handle,
    CommunicatorLogCallback callback,
    void* user_data
);

/**
 * Clear the log callback on a context
 *
 * @param handle The context handle
 * @return Error code indicating success or failure
 */
CommunicatorErrorCode communicator_context_clear_log_callback(CommunicatorContext handle);

// ============================================================================
// Utility Functions
// ============================================================================

/**
 * Get a greeting message from libcommunicator (example function)
 *
 * @param name The name to greet
 * @return A dynamically allocated string that must be freed with communicator_free_string()
 *         Returns NULL on error
 */
char* communicator_greet(const char* name);

/**
 * Free a string allocated by libcommunicator
 *
 * @param s The string to free
 */
void communicator_free_string(char* s);

#ifdef __cplusplus
}
#endif

#endif /* COMMUNICATOR_H */
