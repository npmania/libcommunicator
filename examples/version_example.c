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

    return 0;
}
