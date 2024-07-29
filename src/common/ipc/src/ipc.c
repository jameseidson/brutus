#include "../include/ipc.h"
#include "log.h"

#include <assert.h>
#include <errno.h>
#include <fcntl.h>
#include <stdint.h>
#include <stdio.h>
#include <sys/eventfd.h>
#include <sys/mman.h>
#include <sys/stat.h>
#include <unistd.h>

typedef struct Ipc {
  /**
   * Eventfd file descriptor based semaphore. Counts number of IPC messages
   * available to receive.
   */
  int consumable_fd;

  /**
   * Eventfd file descriptor based semaphore. Counts free slots available for
   * new IPC messages.
   */
  int producible_fd;
  size_t increment;
  Queue queue;
} Ipc;

Ipc *ipc_open(const char *handle, size_t message_size) {
  int shared_mem_fd = shm_open(handle, O_RDWR | O_CREAT | O_EXCL, 0600);

  if (errno == EEXIST) {
    LOG_INFO("Attaching to pre-existing IPC channel (%s)", handle);
    shared_mem_fd = shm_open(handle, O_RDWR, 0600);
    Ipc *ipc = mmap(NULL, sizeof(Ipc), PROT_READ | PROT_WRITE, MAP_SHARED,
                    shared_mem_fd, 0);
    LOG_DEBUG("HERE ARE THE FIELDS OF THE IPC WE FOUND:");
    LOG_DEBUG("q.l %lu, q.r: %lu", ipc->queue.l, ipc->queue.r);
    return ipc;
  }

  LOG_INFO("Opening new IPC channel (%s)", handle);
  assert(ftruncate(shared_mem_fd, sizeof(Ipc)) != -1);
  Ipc *ipc = mmap(NULL, sizeof(Ipc), PROT_READ | PROT_WRITE, MAP_SHARED,
                  shared_mem_fd, 0);

  // BUFFER_SIZE / (BUFFER_SIZE / message_size)
  // ipc->increment = message_size;
  ipc->producible_fd = eventfd(0, EFD_SEMAPHORE);
  assert(ipc->producible_fd != -1);
  eventfd_write(ipc->producible_fd, UINT64_MAX - 1);

  LOG_DEBUG("Created sendable_fd");

  ipc->consumable_fd = eventfd(0, EFD_SEMAPHORE);
  assert(ipc->consumable_fd != -1);

  LOG_DEBUG("Created receivable_fd");

  q_init(&ipc->queue, message_size);

  return ipc;
}

void ipc_send(Ipc *ipc, const void *message) {
  LOG_DEBUG("START SEND >>>> ");
  ipc_await_producible(ipc);
  ipc_send_unsafe(ipc, message);
  ipc_signal_consumable(ipc);
  LOG_DEBUG("END SEND >>>>");
}

void ipc_recv(Ipc *ipc, void *message) {
  LOG_DEBUG("START RECV <<<<");
  ipc_await_consumable(ipc);
  ipc_recv_unsafe(ipc, message);
  ipc_signal_producible(ipc);
  LOG_DEBUG("END RECV <<<<");
}

void ipc_send_unsafe(Ipc *ipc, const void *message) {
  q_push(&ipc->queue, message);
}

void ipc_await_producible(Ipc *ipc) {
  uint64_t value;
  eventfd_read(ipc->producible_fd, &value);
}

void ipc_await_consumable(Ipc *ipc) {
  uint64_t value;
  eventfd_read(ipc->consumable_fd, &value);
}

void ipc_signal_producible(Ipc *ipc) { eventfd_write(ipc->producible_fd, 1); }

void ipc_signal_consumable(Ipc *ipc) { eventfd_write(ipc->consumable_fd, 1); }

void ipc_recv_unsafe(Ipc *ipc, void *message) { q_pop(&ipc->queue, message); }

int ipc_get_consumable_fd(Ipc *ipc) { return ipc->consumable_fd; }

int ipc_get_producible_fd(Ipc *ipc) { return ipc->producible_fd; }

void ipc_free(Ipc *ipc) {
  LOG_ERR("NOT IMPLEMENTED!");
  assert(0);
}
