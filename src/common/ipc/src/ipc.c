#include <assert.h>
#include <bits/pthreadtypes.h>
#include <errno.h>
#include <memory.h>
#include <pthread.h>
#include <semaphore.h>
#include <stdint.h>
#include <stdlib.h>
#include <sys/fcntl.h>
#include <sys/mman.h>
#include <time.h>
#include <unistd.h>

#include "log.h"

#define BUFFER_SIZE (64 * 1024)

// Panic if there's a nonzero error code.
#define EXPECT(expr) assert((expr) == 0)

typedef struct {
  uint8_t buf[BUFFER_SIZE];
  size_t head;
  size_t tail;
  size_t size;
} queue_t;

#define NEXT(i) (((i) + size) % BUFFER_SIZE)

void q_push(queue_t *q, const uint8_t *item, const size_t size) {
  size_t slot = q->head;
  q->head = NEXT(q->head);
  q->size += size;
  assert(q->tail <= q->head);

  memcpy((void *)&q->buf[slot], item, size);
}

void q_pop(queue_t *q, uint8_t *item, const size_t size) {
  size_t slot = q->tail;
  q->tail = NEXT(q->tail);
  q->size -= size;
  assert(q->tail <= q->head);

  memcpy(item, (void *)&q->buf[slot], size);
}

typedef struct ipc {
  queue_t q;
  pthread_mutex_t lock;
  pthread_cond_t sendable;
  pthread_cond_t recvable;
} ipc_t;

ipc_t *ipc_open(const char *handle) {
  int shm = shm_open(handle, O_RDWR | O_CREAT | O_EXCL, 0600);

  if (errno == EEXIST) {
    LOG_INFO("Attaching to pre-existing IPC channel (%s)", handle);

    shm = shm_open(handle, O_RDWR, 0600);
    return mmap(NULL, sizeof(ipc_t), PROT_READ | PROT_WRITE, MAP_SHARED, shm,
                0);
  }

  LOG_INFO("Opening new IPC channel (%s)", handle);

  assert(ftruncate(shm, sizeof(ipc_t)) != -1);
  ipc_t *ipc =
      mmap(NULL, sizeof(ipc_t), PROT_READ | PROT_WRITE, MAP_SHARED, shm, 0);

  EXPECT(pthread_mutex_init(&ipc->lock, NULL));
  EXPECT(pthread_cond_init(&ipc->sendable, NULL));
  EXPECT(pthread_cond_init(&ipc->recvable, NULL));
  ipc->q.size = 0;
  ipc->q.head = 0;
  ipc->q.tail = 0;

  return ipc;
}

void ipc_send(ipc_t *ipc, const uint8_t *msg, const size_t size) {
  EXPECT(pthread_mutex_lock(&ipc->lock));
  while (BUFFER_SIZE - ipc->q.size < size) {
    EXPECT(pthread_cond_wait(&ipc->sendable, &ipc->lock));
  }

  q_push(&ipc->q, msg, size);

  EXPECT(pthread_cond_signal(&ipc->recvable));
  EXPECT(pthread_mutex_unlock(&ipc->lock));
}

void ipc_recv(ipc_t *ipc, uint8_t *msg, const size_t size) {
  EXPECT(pthread_mutex_lock(&ipc->lock));
  while (ipc->q.size < size) {
    EXPECT(pthread_cond_wait(&ipc->recvable, &ipc->lock));
  }

  q_pop(&ipc->q, msg, size);

  EXPECT(pthread_cond_signal(&ipc->sendable));
  EXPECT(pthread_mutex_unlock(&ipc->lock));
}

void ipc_destroy(ipc_t *ipc) {
  pthread_cond_destroy(&ipc->sendable);
  pthread_cond_destroy(&ipc->recvable);
}
