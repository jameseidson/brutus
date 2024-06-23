#ifndef FFI_H
#define FFI_H

#include <stdint.h>

/**
 * Spawns a 'brutusd' server process if it's not already running.
 *
 * Returns the PID of the server process.
 */
extern uint32_t spawn_server_if_not_running();

#endif
