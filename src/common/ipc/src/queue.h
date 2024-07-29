#include <pthread.h>
#include <stdint.h>

#define BUFFER_SIZE (64 * 1024)

typedef struct {
  pthread_mutex_t mutex;
  size_t len;
  size_t item_size;
  size_t l;
  size_t r;
  volatile uint8_t buffer[BUFFER_SIZE];
} Queue;

void q_init(Queue *q, size_t item_size);

void q_push(Queue *q, const uint8_t *item);

void q_pop(Queue *q, uint8_t *item);
