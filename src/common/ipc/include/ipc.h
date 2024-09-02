#ifndef IPC_H
#define IPC_H

#include <stdint.h>
#include <stdlib.h>

typedef struct ipc ipc_t;

ipc_t *ipc_open(const char *handle);

void ipc_send(ipc_t *ipc, const uint8_t *msg, const size_t size);

void ipc_recv(ipc_t *ipc, uint8_t *msg, const size_t size);

void ipc_destroy(ipc_t *ipc);

#endif
