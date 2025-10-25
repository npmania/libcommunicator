/**
 * Comprehensive FFI Patterns Demo
 *
 * This example demonstrates all the FFI patterns implemented in libcommunicator:
 * 1. Library Initialization/Cleanup
 * 2. Version Information
 * 3. Error Handling
 * 4. Opaque Handles (Context Management)
 * 5. Callbacks (Function Pointers)
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "../include/communicator.h"

// User data structure for callback demonstration
typedef struct {
    int log_count;
    const char* name;
} UserData;

// Log callback function
void log_callback(CommunicatorLogLevel level, const char* message, void* user_data) {
    UserData* data = (UserData*)user_data;
    data->log_count++;

    const char* level_str;
    switch (level) {
        case COMMUNICATOR_LOG_DEBUG:   level_str = "DEBUG"; break;
        case COMMUNICATOR_LOG_INFO:    level_str = "INFO"; break;
        case COMMUNICATOR_LOG_WARNING: level_str = "WARN"; break;
        case COMMUNICATOR_LOG_ERROR:   level_str = "ERROR"; break;
        default:                       level_str = "UNKNOWN"; break;
    }

    printf("[CALLBACK #%d] [%s] [%s] %s\n", data->log_count, data->name, level_str, message);
}

// Helper function to print error details
void print_error(const char* operation) {
    CommunicatorErrorCode code = communicator_last_error_code();
    if (code != COMMUNICATOR_SUCCESS) {
        char* msg = communicator_last_error_message();
        printf("  ERROR during %s: [%d] %s - %s\n",
               operation,
               code,
               communicator_error_code_string(code),
               msg ? msg : "(no message)");
        if (msg) {
            communicator_free_string(msg);
        }
    }
}

int main(void) {
    printf("========================================\n");
    printf("FFI Patterns Demonstration\n");
    printf("========================================\n\n");

    // ========================================================================
    // Pattern 1: Library Initialization
    // ========================================================================
    printf("1. Library Initialization Pattern\n");
    printf("----------------------------------\n");

    CommunicatorErrorCode init_result = communicator_init();
    if (init_result != COMMUNICATOR_SUCCESS) {
        printf("Failed to initialize library!\n");
        return 1;
    }
    printf("  ✓ Library initialized successfully\n\n");

    // ========================================================================
    // Pattern 2: Version Information
    // ========================================================================
    printf("2. Version Information\n");
    printf("----------------------------------\n");
    printf("  Version string: %s\n", communicator_version());
    printf("  Version numbers: %u.%u.%u\n",
           communicator_version_major(),
           communicator_version_minor(),
           communicator_version_patch());
    printf("\n");

    // ========================================================================
    // Pattern 3: Opaque Handles (Context Management)
    // ========================================================================
    printf("3. Opaque Handle Pattern\n");
    printf("----------------------------------\n");

    // Create a context
    CommunicatorContext ctx = communicator_context_create("demo-context");
    if (ctx == NULL) {
        printf("  Failed to create context!\n");
        print_error("context creation");
        goto cleanup;
    }
    printf("  ✓ Context created\n");

    // Check initialization state
    int is_init = communicator_context_is_initialized(ctx);
    printf("  ✓ Context initialized: %s\n", is_init ? "yes" : "no");

    // Set configuration
    CommunicatorErrorCode err = communicator_context_set_config(ctx, "server", "mattermost.example.com");
    if (err == COMMUNICATOR_SUCCESS) {
        printf("  ✓ Configuration set: server=mattermost.example.com\n");
    }

    err = communicator_context_set_config(ctx, "port", "443");
    if (err == COMMUNICATOR_SUCCESS) {
        printf("  ✓ Configuration set: port=443\n");
    }

    // Get configuration
    char* server = communicator_context_get_config(ctx, "server");
    if (server != NULL) {
        printf("  ✓ Configuration retrieved: server=%s\n", server);
        communicator_free_string(server);
    }

    char* port = communicator_context_get_config(ctx, "port");
    if (port != NULL) {
        printf("  ✓ Configuration retrieved: port=%s\n", port);
        communicator_free_string(port);
    }

    // Try to get non-existent key
    char* missing = communicator_context_get_config(ctx, "nonexistent");
    if (missing == NULL) {
        printf("  ✓ Non-existent key correctly returns NULL\n");
        print_error("getting non-existent config");
        communicator_clear_error();
    }
    printf("\n");

    // ========================================================================
    // Pattern 5: Callbacks (Function Pointers)
    // ========================================================================
    printf("5. Callback Pattern\n");
    printf("----------------------------------\n");

    // Set up user data for callback
    UserData user_data = { .log_count = 0, .name = "MyApp" };

    // Register callback
    err = communicator_context_set_log_callback(ctx, log_callback, &user_data);
    if (err == COMMUNICATOR_SUCCESS) {
        printf("  ✓ Log callback registered\n");
    }

    // Initialize context (this will trigger log callbacks)
    printf("  Initializing context (will trigger callbacks):\n");
    err = communicator_context_initialize(ctx);
    if (err == COMMUNICATOR_SUCCESS) {
        printf("  ✓ Context initialized\n");
    }

    // Verify initialization
    is_init = communicator_context_is_initialized(ctx);
    printf("  ✓ Context initialized: %s\n", is_init ? "yes" : "no");

    // Shutdown context (will trigger more callbacks)
    printf("  Shutting down context (will trigger callbacks):\n");
    err = communicator_context_shutdown(ctx);
    if (err == COMMUNICATOR_SUCCESS) {
        printf("  ✓ Context shutdown\n");
    }

    printf("  ✓ Total callbacks received: %d\n", user_data.log_count);
    printf("\n");

    // ========================================================================
    // Cleanup
    // ========================================================================
    printf("6. Cleanup\n");
    printf("----------------------------------\n");

    // Clear callback
    communicator_context_clear_log_callback(ctx);
    printf("  ✓ Callback cleared\n");

    // Destroy context
    communicator_context_destroy(ctx);
    printf("  ✓ Context destroyed\n");

cleanup:
    // Cleanup library
    communicator_cleanup();
    printf("  ✓ Library cleaned up\n\n");

    printf("========================================\n");
    printf("All FFI patterns demonstrated successfully!\n");
    printf("========================================\n");

    return 0;
}
