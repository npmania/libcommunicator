/**
 * Mattermost Platform Demo
 *
 * This example demonstrates how to use libcommunicator to connect to a
 * Mattermost server and perform basic operations.
 *
 * Usage:
 *   ./mattermost_demo <server_url> <login_id> <password> <team_id>
 *
 * Example:
 *   ./mattermost_demo https://mattermost.example.com user@example.com mypassword abc123
 *
 * Or with token authentication:
 *   ./mattermost_demo <server_url> <token> "" <team_id>
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "../include/communicator.h"

#define BUFFER_SIZE 4096

/**
 * Helper function to format JSON config for platform connection
 */
char* format_config_json(const char* server_url, const char* auth1, const char* auth2, const char* team_id) {
    char* buffer = malloc(BUFFER_SIZE);
    if (!buffer) return NULL;

    // Check if using token or login_id/password
    if (auth2 == NULL || strlen(auth2) == 0) {
        // Token authentication
        snprintf(buffer, BUFFER_SIZE,
            "{"
            "\"server\":\"%s\","
            "\"credentials\":{\"token\":\"%s\"},"
            "\"team_id\":\"%s\""
            "}",
            server_url, auth1, team_id);
    } else {
        // Username/password authentication
        snprintf(buffer, BUFFER_SIZE,
            "{"
            "\"server\":\"%s\","
            "\"credentials\":{\"login_id\":\"%s\",\"password\":\"%s\"},"
            "\"team_id\":\"%s\""
            "}",
            server_url, auth1, auth2, team_id);
    }

    return buffer;
}

/**
 * Helper function to print error details
 */
void print_error(const char* operation) {
    CommunicatorErrorCode code = communicator_last_error_code();
    char* message = communicator_last_error_message();

    printf("ERROR during %s:\n", operation);
    printf("  Code: %d (%s)\n", code, communicator_error_code_string(code));
    if (message) {
        printf("  Message: %s\n", message);
        communicator_free_string(message);
    }
}

/**
 * Helper function to print JSON response
 */
void print_json(const char* label, const char* json) {
    printf("\n%s:\n", label);
    if (json) {
        // Pretty print (basic - just add newlines for readability)
        printf("%s\n", json);
    } else {
        printf("  (null)\n");
    }
}

int main(int argc, char* argv[]) {
    // Check arguments
    if (argc < 5) {
        printf("Usage: %s <server_url> <login_id_or_token> <password_or_empty> <team_id>\n", argv[0]);
        printf("\nExamples:\n");
        printf("  Token auth:    %s https://mattermost.example.com mytoken \"\" abc123\n", argv[0]);
        printf("  Password auth: %s https://mattermost.example.com user@example.com mypass abc123\n", argv[0]);
        return 1;
    }

    const char* server_url = argv[1];
    const char* auth1 = argv[2];
    const char* auth2 = argv[3];
    const char* team_id = argv[4];

    printf("=== Mattermost Platform Demo ===\n");
    printf("Server: %s\n", server_url);
    printf("Team ID: %s\n\n", team_id);

    // ========================================================================
    // 1. Initialize the library
    // ========================================================================
    printf("1. Initializing library...\n");
    CommunicatorErrorCode err = communicator_init();
    if (err != COMMUNICATOR_SUCCESS) {
        print_error("library initialization");
        return 1;
    }
    printf("   Library version: %s\n", communicator_version());
    printf("   ✓ Initialized\n\n");

    // ========================================================================
    // 2. Create Mattermost platform instance
    // ========================================================================
    printf("2. Creating Mattermost platform...\n");
    CommunicatorPlatform platform = communicator_mattermost_create(server_url);
    if (!platform) {
        print_error("platform creation");
        communicator_cleanup();
        return 1;
    }
    printf("   ✓ Platform created\n\n");

    // ========================================================================
    // 3. Connect and authenticate
    // ========================================================================
    printf("3. Connecting to Mattermost...\n");
    char* config_json = format_config_json(server_url, auth1, auth2, team_id);
    if (!config_json) {
        printf("ERROR: Failed to allocate config buffer\n");
        communicator_platform_destroy(platform);
        communicator_cleanup();
        return 1;
    }

    printf("   Config: %s\n", config_json);
    err = communicator_platform_connect(platform, config_json);
    free(config_json);

    if (err != COMMUNICATOR_SUCCESS) {
        print_error("platform connection");
        communicator_platform_destroy(platform);
        communicator_cleanup();
        return 1;
    }
    printf("   ✓ Connected\n\n");

    // ========================================================================
    // 4. Check connection status
    // ========================================================================
    printf("4. Checking connection status...\n");
    int is_connected = communicator_platform_is_connected(platform);
    if (is_connected < 0) {
        print_error("connection status check");
    } else {
        printf("   Connected: %s\n", is_connected ? "yes" : "no");
    }

    char* conn_info = communicator_platform_get_connection_info(platform);
    if (conn_info) {
        print_json("   Connection Info", conn_info);
        communicator_free_string(conn_info);
    }
    printf("\n");

    // ========================================================================
    // 5. Get current user
    // ========================================================================
    printf("5. Getting current user info...\n");
    char* user_json = communicator_platform_get_current_user(platform);
    if (user_json) {
        print_json("   Current User", user_json);
        communicator_free_string(user_json);
        printf("   ✓ Retrieved user info\n\n");
    } else {
        print_error("get current user");
        printf("\n");
    }

    // ========================================================================
    // 6. Get channels
    // ========================================================================
    printf("6. Getting channels...\n");
    char* channels_json = communicator_platform_get_channels(platform);
    if (channels_json) {
        print_json("   Channels", channels_json);
        communicator_free_string(channels_json);
        printf("   ✓ Retrieved channels\n\n");
    } else {
        print_error("get channels");
        printf("\n");
    }

    // ========================================================================
    // 7. Send a message (optional - requires channel ID)
    // ========================================================================
    // Uncomment and modify to test sending a message:
    /*
    printf("7. Sending a test message...\n");
    const char* test_channel_id = "your-channel-id-here";
    const char* test_message = "Hello from libcommunicator!";

    char* message_json = communicator_platform_send_message(platform, test_channel_id, test_message);
    if (message_json) {
        print_json("   Sent Message", message_json);
        communicator_free_string(message_json);
        printf("   ✓ Message sent\n\n");
    } else {
        print_error("send message");
        printf("\n");
    }
    */

    // ========================================================================
    // 8. Disconnect
    // ========================================================================
    printf("8. Disconnecting...\n");
    err = communicator_platform_disconnect(platform);
    if (err != COMMUNICATOR_SUCCESS) {
        print_error("disconnect");
    } else {
        printf("   ✓ Disconnected\n\n");
    }

    // ========================================================================
    // 9. Cleanup
    // ========================================================================
    printf("9. Cleaning up...\n");
    communicator_platform_destroy(platform);
    communicator_cleanup();
    printf("   ✓ Cleanup complete\n\n");

    printf("=== Demo Complete ===\n");
    return 0;
}
