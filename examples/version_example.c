#include <stdio.h>
#include <stdlib.h>
#include "../include/communicator.h"

int main(void) {
    // Display version information
    printf("libcommunicator Version Information:\n");
    printf("=====================================\n");
    printf("Version string: %s\n", communicator_version());
    printf("Version numbers: %u.%u.%u\n",
           communicator_version_major(),
           communicator_version_minor(),
           communicator_version_patch());
    printf("\n");

    // Test the greeting function
    printf("Testing greeting function:\n");
    printf("=====================================\n");
    char* greeting = communicator_greet("FFI User");
    if (greeting != NULL) {
        printf("%s\n", greeting);
        communicator_free_string(greeting);
    } else {
        fprintf(stderr, "Error: Failed to get greeting\n");
        return 1;
    }

    return 0;
}
