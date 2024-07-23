#ifndef FFI_H
#define FFI_H

#include <stdint.h>

/**
 * Spawns a 'brutusd' server process if it's not already running.
 *
 * Return: The PID of the running server process.
 */
extern uint32_t spawn_server_if_not_running();

/**
 * Runs the 'brutus' client.
 *
 * server_pid: The PID of the running server process.
 */
extern void RunClient(uint32_t server_pid);

#endif
