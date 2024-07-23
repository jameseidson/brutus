#include "common/ffi/ffi.h"

#include <stdint.h>

int main() {
  const uint32_t server_pid = spawn_server_if_not_running();
  RunClient(server_pid);
}
