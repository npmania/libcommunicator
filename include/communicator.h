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
    COMMUNICATOR_ERROR_UNSUPPORTED = 12,
    COMMUNICATOR_ERROR_RATE_LIMITED = 13,
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
 * Free a string allocated by libcommunicator
 *
 * @param s The string to free
 */
void communicator_free_string(char* s);

// ============================================================================
// Platform API - Mattermost Integration
// ============================================================================

/**
 * Opaque handle to a Platform object
 */
typedef void* CommunicatorPlatform;

/**
 * Create a new Mattermost platform instance
 *
 * @param server_url The Mattermost server URL (e.g., "https://mattermost.example.com")
 * @return An opaque handle to the platform, or NULL on error
 *         Must be freed with communicator_platform_destroy()
 */
CommunicatorPlatform communicator_mattermost_create(const char* server_url);

/**
 * Connect to a platform and authenticate
 *
 * @param platform The platform handle
 * @param config_json JSON configuration string with format:
 *                    {
 *                      "server": "https://mattermost.example.com",
 *                      "credentials": {
 *                        "token": "xxx" OR "login_id": "user@example.com", "password": "xxx"
 *                      },
 *                      "team_id": "optional-team-id"
 *                    }
 * @return Error code indicating success or failure
 */
CommunicatorErrorCode communicator_platform_connect(
    CommunicatorPlatform platform,
    const char* config_json
);

/**
 * Disconnect from a platform
 *
 * @param platform The platform handle
 * @return Error code indicating success or failure
 */
CommunicatorErrorCode communicator_platform_disconnect(CommunicatorPlatform platform);

/**
 * Check if platform is connected
 *
 * @param platform The platform handle
 * @return 1 if connected, 0 if not, -1 on error
 */
int communicator_platform_is_connected(CommunicatorPlatform platform);

/**
 * Get connection info as JSON
 *
 * @param platform The platform handle
 * @return A dynamically allocated JSON string that must be freed with communicator_free_string()
 *         Returns NULL on error or if not connected
 */
char* communicator_platform_get_connection_info(CommunicatorPlatform platform);

/**
 * Send a message to a channel
 *
 * @param platform The platform handle
 * @param channel_id The channel ID to send the message to
 * @param text The message text
 * @return A JSON string representing the created Message
 *         Must be freed with communicator_free_string()
 *         Returns NULL on error
 */
char* communicator_platform_send_message(
    CommunicatorPlatform platform,
    const char* channel_id,
    const char* text
);

/**
 * Get all channels for the current user
 *
 * @param platform The platform handle
 * @return A JSON array string of Channel objects
 *         Must be freed with communicator_free_string()
 *         Returns NULL on error
 */
char* communicator_platform_get_channels(CommunicatorPlatform platform);

/**
 * Get a specific channel by ID
 *
 * @param platform The platform handle
 * @param channel_id The channel ID
 * @return A JSON string representing the Channel
 *         Must be freed with communicator_free_string()
 *         Returns NULL on error
 */
char* communicator_platform_get_channel(
    CommunicatorPlatform platform,
    const char* channel_id
);

/**
 * Get recent messages from a channel
 *
 * @param platform The platform handle
 * @param channel_id The channel ID
 * @param limit Maximum number of messages to retrieve
 * @return A JSON array string of Message objects
 *         Must be freed with communicator_free_string()
 *         Returns NULL on error
 */
char* communicator_platform_get_messages(
    CommunicatorPlatform platform,
    const char* channel_id,
    uint32_t limit
);

/**
 * Get members of a channel
 *
 * @param platform The platform handle
 * @param channel_id The channel ID
 * @return A JSON array string of User objects
 *         Must be freed with communicator_free_string()
 *         Returns NULL on error
 */
char* communicator_platform_get_channel_members(
    CommunicatorPlatform platform,
    const char* channel_id
);

/**
 * Get a specific user by ID
 *
 * @param platform The platform handle
 * @param user_id The user ID
 * @return A JSON string representing the User
 *         Must be freed with communicator_free_string()
 *         Returns NULL on error
 */
char* communicator_platform_get_user(
    CommunicatorPlatform platform,
    const char* user_id
);

/**
 * Get the current authenticated user
 *
 * @param platform The platform handle
 * @return A JSON string representing the User
 *         Must be freed with communicator_free_string()
 *         Returns NULL on error
 */
char* communicator_platform_get_current_user(CommunicatorPlatform platform);

/**
 * Create a direct message channel with another user
 *
 * @param platform The platform handle
 * @param user_id The user ID to create a DM channel with
 * @return A JSON string representing the created Channel
 *         Must be freed with communicator_free_string()
 *         Returns NULL on error
 */
char* communicator_platform_create_direct_channel(
    CommunicatorPlatform platform,
    const char* user_id
);

/**
 * Request statuses for all users via WebSocket (async operation)
 *
 * This is a non-blocking operation that returns immediately with a sequence number.
 * The actual status data will arrive later as a Response event with matching seq_reply.
 * Requires an active WebSocket connection (call subscribe_events first).
 *
 * @param platform The platform handle
 * @return The sequence number on success, or -1 on error
 */
int64_t communicator_platform_request_all_statuses(CommunicatorPlatform platform);

/**
 * Request statuses for specific users via WebSocket (async operation)
 *
 * This is a non-blocking operation that returns immediately with a sequence number.
 * The actual status data will arrive later as a Response event with matching seq_reply.
 * Requires an active WebSocket connection (call subscribe_events first).
 *
 * @param platform The platform handle
 * @param user_ids_json JSON array of user IDs (e.g., "[\"user1\", \"user2\"]")
 * @return The sequence number on success, or -1 on error
 */
int64_t communicator_platform_request_users_statuses(
    CommunicatorPlatform platform,
    const char* user_ids_json
);

/**
 * Subscribe to real-time events
 *
 * @param platform The platform handle
 * @return Error code indicating success or failure
 */
CommunicatorErrorCode communicator_platform_subscribe_events(CommunicatorPlatform platform);

/**
 * Unsubscribe from real-time events
 *
 * @param platform The platform handle
 * @return Error code indicating success or failure
 */
CommunicatorErrorCode communicator_platform_unsubscribe_events(CommunicatorPlatform platform);

/**
 * Poll for the next event
 *
 * @param platform The platform handle
 * @return A JSON string representing the PlatformEvent, or NULL if no events are available
 *         Event format: { "type": "event_type", "data": {...} }
 *         Must be freed with communicator_free_string()
 *         Returns NULL if no events or on error
 */
char* communicator_platform_poll_event(CommunicatorPlatform platform);

// ============================================================================
// Extended Message Operations
// ============================================================================

/**
 * Send a reply to a message (threaded conversation)
 *
 * @param platform The platform handle
 * @param channel_id The channel ID
 * @param text The reply text
 * @param root_id The ID of the root message to reply to
 * @return A JSON string representing the created Message
 *         Must be freed with communicator_free_string()
 *         Returns NULL on error
 */
char* communicator_platform_send_reply(
    CommunicatorPlatform platform,
    const char* channel_id,
    const char* text,
    const char* root_id
);

/**
 * Update/edit a message
 *
 * @param platform The platform handle
 * @param message_id The ID of the message to update
 * @param new_text The new message text
 * @return A JSON string representing the updated Message
 *         Must be freed with communicator_free_string()
 *         Returns NULL on error
 */
char* communicator_platform_update_message(
    CommunicatorPlatform platform,
    const char* message_id,
    const char* new_text
);

/**
 * Delete a message
 *
 * @param platform The platform handle
 * @param message_id The ID of the message to delete
 * @return Error code indicating success or failure
 */
CommunicatorErrorCode communicator_platform_delete_message(
    CommunicatorPlatform platform,
    const char* message_id
);

/**
 * Get a specific message by ID
 *
 * @param platform The platform handle
 * @param message_id The message ID
 * @return A JSON string representing the Message
 *         Must be freed with communicator_free_string()
 *         Returns NULL on error
 */
char* communicator_platform_get_message(
    CommunicatorPlatform platform,
    const char* message_id
);

/**
 * Search for messages
 *
 * @param platform The platform handle
 * @param query The search query
 * @param limit Maximum number of messages to retrieve
 * @return A JSON array string of Message objects
 *         Must be freed with communicator_free_string()
 *         Returns NULL on error
 */
char* communicator_platform_search_messages(
    CommunicatorPlatform platform,
    const char* query,
    uint32_t limit
);

/**
 * Get messages before a specific message (pagination)
 *
 * @param platform The platform handle
 * @param channel_id The channel ID
 * @param before_id The message ID to get messages before
 * @param limit Maximum number of messages to retrieve
 * @return A JSON array string of Message objects
 *         Must be freed with communicator_free_string()
 *         Returns NULL on error
 */
char* communicator_platform_get_messages_before(
    CommunicatorPlatform platform,
    const char* channel_id,
    const char* before_id,
    uint32_t limit
);

/**
 * Get messages after a specific message (pagination)
 *
 * @param platform The platform handle
 * @param channel_id The channel ID
 * @param after_id The message ID to get messages after
 * @param limit Maximum number of messages to retrieve
 * @return A JSON array string of Message objects
 *         Must be freed with communicator_free_string()
 *         Returns NULL on error
 */
char* communicator_platform_get_messages_after(
    CommunicatorPlatform platform,
    const char* channel_id,
    const char* after_id,
    uint32_t limit
);

// ============================================================================
// Reaction Operations
// ============================================================================

/**
 * Add a reaction to a message
 *
 * @param platform The platform handle
 * @param message_id The message ID to react to
 * @param emoji_name The emoji name (e.g., "thumbsup", "smile", "heart")
 * @return Error code indicating success or failure
 */
CommunicatorErrorCode communicator_platform_add_reaction(
    CommunicatorPlatform platform,
    const char* message_id,
    const char* emoji_name
);

/**
 * Remove a reaction from a message
 *
 * @param platform The platform handle
 * @param message_id The message ID
 * @param emoji_name The emoji name to remove
 * @return Error code indicating success or failure
 */
CommunicatorErrorCode communicator_platform_remove_reaction(
    CommunicatorPlatform platform,
    const char* message_id,
    const char* emoji_name
);

/**
 * Pin a message/post to its channel
 *
 * @param platform The platform handle
 * @param message_id The ID of the message to pin
 * @return COMMUNICATOR_SUCCESS on success, error code on failure
 */
CommunicatorErrorCode communicator_platform_pin_post(
    CommunicatorPlatform platform,
    const char* message_id
);

/**
 * Unpin a message/post from its channel
 *
 * @param platform The platform handle
 * @param message_id The ID of the message to unpin
 * @return COMMUNICATOR_SUCCESS on success, error code on failure
 */
CommunicatorErrorCode communicator_platform_unpin_post(
    CommunicatorPlatform platform,
    const char* message_id
);

/**
 * Get all pinned messages/posts for a channel
 *
 * @param platform The platform handle
 * @param channel_id The ID of the channel
 * @return A JSON string containing an array of pinned messages
 *         Must be freed with communicator_free_string()
 *         Returns NULL on error
 */
char* communicator_platform_get_pinned_posts(
    CommunicatorPlatform platform,
    const char* channel_id
);

/**
 * Get a list of custom emojis
 *
 * @param platform The platform handle
 * @param page The page number to retrieve (0-indexed)
 * @param per_page Number of emojis per page
 * @return A JSON string representing a Vec<Emoji>
 *         Must be freed with communicator_free_string()
 *         Returns NULL on error
 */
char* communicator_platform_get_emojis(
    CommunicatorPlatform platform,
    uint32_t page,
    uint32_t per_page
);

// ============================================================================
// Extended Channel Operations
// ============================================================================

/**
 * Get a channel by name
 *
 * @param platform The platform handle
 * @param team_id The team ID
 * @param channel_name The channel name
 * @return A JSON string representing the Channel
 *         Must be freed with communicator_free_string()
 *         Returns NULL on error
 */
char* communicator_platform_get_channel_by_name(
    CommunicatorPlatform platform,
    const char* team_id,
    const char* channel_name
);

/**
 * Create a group direct message channel
 *
 * @param platform The platform handle
 * @param user_ids_json JSON array of user IDs, e.g. ["user1", "user2", "user3"]
 * @return A JSON string representing the created Channel
 *         Must be freed with communicator_free_string()
 *         Returns NULL on error
 */
char* communicator_platform_create_group_channel(
    CommunicatorPlatform platform,
    const char* user_ids_json
);

/**
 * Add a user to a channel
 *
 * @param platform The platform handle
 * @param channel_id The channel ID
 * @param user_id The user ID to add
 * @return Error code indicating success or failure
 */
CommunicatorErrorCode communicator_platform_add_channel_member(
    CommunicatorPlatform platform,
    const char* channel_id,
    const char* user_id
);

/**
 * Remove a user from a channel
 *
 * @param platform The platform handle
 * @param channel_id The channel ID
 * @param user_id The user ID to remove
 * @return Error code indicating success or failure
 */
CommunicatorErrorCode communicator_platform_remove_channel_member(
    CommunicatorPlatform platform,
    const char* channel_id,
    const char* user_id
);

// ============================================================================
// Extended User Operations
// ============================================================================

/**
 * Get a user by username
 *
 * @param platform The platform handle
 * @param username The username
 * @return A JSON string representing the User
 *         Must be freed with communicator_free_string()
 *         Returns NULL on error
 */
char* communicator_platform_get_user_by_username(
    CommunicatorPlatform platform,
    const char* username
);

/**
 * Get a user by email
 *
 * @param platform The platform handle
 * @param email The email address
 * @return A JSON string representing the User
 *         Must be freed with communicator_free_string()
 *         Returns NULL on error
 */
char* communicator_platform_get_user_by_email(
    CommunicatorPlatform platform,
    const char* email
);

/**
 * Get multiple users by their IDs (batch operation)
 *
 * @param platform The platform handle
 * @param user_ids_json JSON array of user IDs, e.g. ["user1", "user2", "user3"]
 * @return A JSON array string of User objects
 *         Must be freed with communicator_free_string()
 *         Returns NULL on error
 */
char* communicator_platform_get_users_by_ids(
    CommunicatorPlatform platform,
    const char* user_ids_json
);

// ============================================================================
// Team Management
// ============================================================================

/**
 * Get all teams the user belongs to
 *
 * @param platform The platform handle
 * @return A JSON array string of Team objects
 *         Must be freed with communicator_free_string()
 *         Returns NULL on error
 */
char* communicator_platform_get_teams(CommunicatorPlatform platform);

/**
 * Get a specific team by ID
 *
 * @param platform The platform handle
 * @param team_id The team ID
 * @return A JSON string representing the Team
 *         Must be freed with communicator_free_string()
 *         Returns NULL on error
 */
char* communicator_platform_get_team(
    CommunicatorPlatform platform,
    const char* team_id
);

/**
 * Get a team by name
 *
 * @param platform The platform handle
 * @param team_name The team name
 * @return A JSON string representing the Team
 *         Must be freed with communicator_free_string()
 *         Returns NULL on error
 */
char* communicator_platform_get_team_by_name(
    CommunicatorPlatform platform,
    const char* team_name
);

/**
 * Set the active team/workspace ID
 *
 * @param platform The platform handle
 * @param team_id The team ID to set as active (pass NULL to unset)
 * @return Error code indicating success or failure
 */
CommunicatorErrorCode communicator_platform_set_team_id(
    CommunicatorPlatform platform,
    const char* team_id
);

// ============================================================================
// User Status Management
// ============================================================================

/**
 * Set the current user's status
 *
 * @param platform The platform handle
 * @param status Status string: "online", "away", "dnd", or "offline"
 * @return Error code indicating success or failure
 */
CommunicatorErrorCode communicator_platform_set_status(
    CommunicatorPlatform platform,
    const char* status
);

/**
 * Get a user's status
 *
 * @param platform The platform handle
 * @param user_id The user ID
 * @return A JSON string representing the status: {"status": "online"}
 *         Must be freed with communicator_free_string()
 *         Returns NULL on error
 */
char* communicator_platform_get_user_status(
    CommunicatorPlatform platform,
    const char* user_id
);

/**
 * Get status for multiple users (batch operation)
 *
 * @param platform The platform handle
 * @param user_ids_json JSON array of user IDs, e.g. ["user1", "user2", "user3"]
 * @return A JSON object mapping user IDs to status strings: {"user1": "online", "user2": "away", ...}
 *         Must be freed with communicator_free_string()
 *         Returns NULL on error
 */
char* communicator_platform_get_users_status(
    CommunicatorPlatform platform,
    const char* user_ids_json
);

// ============================================================================
// Custom Status Management
// ============================================================================

/**
 * Set a custom status message
 *
 * @param platform The platform handle
 * @param custom_status_json JSON object with format:
 *                          {
 *                            "emoji": "optional-emoji",
 *                            "text": "status text",
 *                            "expires_at": 1234567890  // Optional Unix timestamp
 *                          }
 * @return Error code indicating success or failure
 */
CommunicatorErrorCode communicator_platform_set_custom_status(
    CommunicatorPlatform platform,
    const char* custom_status_json
);

/**
 * Remove/clear the current user's custom status
 *
 * @param platform The platform handle
 * @return Error code indicating success or failure
 */
CommunicatorErrorCode communicator_platform_remove_custom_status(CommunicatorPlatform platform);

// ============================================================================
// Typing Indicators
// ============================================================================

/**
 * Send a typing indicator to a channel
 *
 * @param platform The platform handle
 * @param channel_id The channel ID to send the typing indicator to
 * @param parent_id The parent message ID for threaded replies (pass NULL for main channel)
 * @return Error code indicating success or failure
 */
CommunicatorErrorCode communicator_platform_send_typing_indicator(
    CommunicatorPlatform platform,
    const char* channel_id,
    const char* parent_id
);

// ============================================================================
// File Operations
// ============================================================================

/**
 * Upload a file to a channel
 *
 * @param platform The platform handle
 * @param channel_id The channel ID where the file will be uploaded
 * @param file_path Path to the file to upload
 * @return A dynamically allocated string containing the file ID (caller must free with communicator_free_string())
 *         Returns NULL on error
 */
char* communicator_platform_upload_file(
    CommunicatorPlatform platform,
    const char* channel_id,
    const char* file_path
);

/**
 * Download a file by its ID
 *
 * @param platform The platform handle
 * @param file_id The ID of the file to download
 * @param out_data Output parameter for the file data (caller must free with communicator_free_file_data())
 * @param out_size Output parameter for the size of the file data in bytes
 * @return Error code indicating success or failure
 */
CommunicatorErrorCode communicator_platform_download_file(
    CommunicatorPlatform platform,
    const char* file_id,
    uint8_t** out_data,
    size_t* out_size
);

/**
 * Get file metadata without downloading the file
 *
 * @param platform The platform handle
 * @param file_id The ID of the file
 * @return A dynamically allocated JSON string representing the Attachment metadata
 *         (caller must free with communicator_free_string())
 *         Returns NULL on error
 */
char* communicator_platform_get_file_metadata(
    CommunicatorPlatform platform,
    const char* file_id
);

/**
 * Get file thumbnail
 *
 * @param platform The platform handle
 * @param file_id The ID of the file
 * @param out_data Output parameter for the thumbnail data (caller must free with communicator_free_file_data())
 * @param out_size Output parameter for the size of the thumbnail data in bytes
 * @return Error code indicating success or failure
 */
CommunicatorErrorCode communicator_platform_get_file_thumbnail(
    CommunicatorPlatform platform,
    const char* file_id,
    uint8_t** out_data,
    size_t* out_size
);

/**
 * Free file data allocated by download_file or get_file_thumbnail
 *
 * @param data Pointer to file data
 * @param size Size of the data in bytes
 */
void communicator_free_file_data(uint8_t* data, size_t size);

// ============================================================================
// Platform Cleanup
// ============================================================================

/**
 * Destroy a platform and free its memory
 * After calling this, the handle is invalid and must not be used
 *
 * @param platform The platform handle
 */
void communicator_platform_destroy(CommunicatorPlatform platform);

#ifdef __cplusplus
}
#endif

#endif /* COMMUNICATOR_H */
