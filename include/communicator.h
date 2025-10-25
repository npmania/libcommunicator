#ifndef COMMUNICATOR_H
#define COMMUNICATOR_H

#ifdef __cplusplus
extern "C" {
#endif

/**
 * Get a greeting message from libcommunicator
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
