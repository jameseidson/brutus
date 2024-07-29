#ifndef IPC_H
#define IPC_H

#include "../src/queue.h"

#include <stdint.h>
#include <stdio.h>

typedef struct Ipc Ipc;

Ipc *ipc_open(const char *handle, size_t message_size);

void ipc_send(Ipc *ipc, const void *message);

void ipc_recv(Ipc *ipc, void *message);

void ipc_send_unsafe(Ipc *ipc, const void *message);

void ipc_recv_unsafe(Ipc *ipc, void *message);

int ipc_get_fd_receivable(Ipc *ipc);

int ipc_get_fd_sendable(Ipc *ipc);

void ipc_await_producible(Ipc *ipc);

void ipc_await_consumable(Ipc *ipc);

void ipc_signal_producible(Ipc *ipc);

void ipc_signal_consumable(Ipc *ipc);

void ipc_free(Ipc *ipc);

#endif
