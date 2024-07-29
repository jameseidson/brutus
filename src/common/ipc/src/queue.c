#include "queue.h"
#include "log.h"

#include <assert.h>
#include <pthread.h>
#include <stdint.h>
#include <string.h>
#include <sys/types.h>
#include <time.h>

#define NEXT(i, q) (((i) + (q)->item_size) % BUFFER_SIZE)

void q_init(Queue *q, size_t item_size) {
  pthread_mutex_init(&q->mutex, NULL);
  q->len = 0;
  q->item_size = item_size;
  q->l = 0;
  q->r = 0;
}

void q_push(Queue *q, const uint8_t *item) {
  pthread_mutex_lock(&q->mutex);
  LOG_DEBUG("Pushed item %u to buffer", item[0]);
  memcpy((void *)&q->buffer[q->r], item, q->item_size);
  q->r = NEXT(q->r, q);
  __atomic_fetch_add(&q->len, 1, __ATOMIC_SEQ_CST);
  LOG_DEBUG("incremented r (l: %lu, r: %lu)", q->l, q->r);
  assert(q->l < q->r);
  pthread_mutex_unlock(&q->mutex);
}

void q_pop(Queue *q, uint8_t *item) {
  LOG_DEBUG("Popping item from buffer");
  pthread_mutex_lock(&q->mutex);
  memcpy(item, (void *)&q->buffer[q->l], q->item_size);
  LOG_DEBUG("The item is: %u", item[0]);
  q->l = NEXT(q->l, q);
  __atomic_fetch_sub(&q->len, 1, __ATOMIC_SEQ_CST);
  LOG_DEBUG("incremented l (l: %lu, r: %lu)", q->l, q->r);
  assert(q->l <= q->r);
  pthread_mutex_unlock(&q->mutex);
}
