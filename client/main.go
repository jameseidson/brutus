package main

import (
	"log/slog"
)

// #include "common/ffi/ffi.h"
import "C"

func main() {
	slog.Info("hello from brutus client")

	server_pid := C.spawn_server_if_not_running()

  println("the server's pid is ", server_pid)
	// slog.Debug("pid from server: ", a)
	for {
	}
}
