package main

import (
	"log/slog"
)

// #include "common/ffi/ffi.h"
import "C"

func main() {
	slog.Info("hello from brutus client")

	a := C.spawn_server_daemon()

	println("yo me gusta: ", a)
	// slog.Debug("pid from server: ", a)
	for {
	}
}
