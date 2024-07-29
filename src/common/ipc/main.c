#include "include/ipc.h"
#include "src/log.h"

#include <fcntl.h>
#include <pthread.h>
#include <stdint.h>
#include <sys/mman.h>
#include <unistd.h>

void *send_1(void *arg) {
  LOG_INFO("spawned send_thread_1");
  sleep(1);
  Ipc *ipc = ipc_open("testhandle", 1);

  // for (uint8_t i = 0; i < 10; i++) {
  //   ipc_send(ipc, &i);
  // }
  uint8_t data = 1;
  ipc_send(ipc, &data);

  return NULL;
}

void *send_2(void *arg) {
  LOG_INFO("spawned send_thread");
  Ipc *ipc = ipc_open("testhandle", 1);

  LOG_DEBUG("send_thread_2 is sending the data");
  uint8_t data = 2;
  ipc_send(ipc, &data);

  return NULL;
}

int main(int argc, char **argv) {
  if (argc == 2) {
    LOG_INFO("UNLINKING....");
    shm_unlink("testhandle");
    return 0;
  }
  Ipc *ipc = ipc_open("testhandle", 1);

  pthread_t send_thread_1;
  pthread_create(&send_thread_1, NULL, send_1, NULL);

  // pthread_t send_thread_2;
  // pthread_create(&send_thread_2, NULL, send_2, NULL);

  // pthread_t send_thread_3;
  // pthread_create(&send_thread_3, NULL, send_3, NULL);

  // for (uint8_t i = 0; i < 10; i++) {
  //   ipc_send(ipc, &i);
  // }

  uint8_t data = 0;
  LOG_DEBUG("blocking on recv...");
  ipc_recv(ipc, &data);

  // while (1) {
  //   LOG_DEBUG("blocking on recv...");
  //   uint8_t data = 0;
  //   ipc_recv(ipc, &data);
  //   LOG_DEBUG("RECEIVED DATA: %u", data);
  // }
  pthread_join(send_thread_1, NULL);
  // pthread_join(send_thread_2, NULL);
  // pthread_join(send_thread_3, NULL);
  shm_unlink("testhandle");
}
