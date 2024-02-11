package main

import (
	"log/slog"
)

// #include "common/ffi/ffi.h"
import "C"

func main() {
	slog.Info("hello from brutus client")

	C.run_server()
}
