package main

import (
	"log/slog"

	"capnproto.org/go/capnp/v3"
	"github.com/jameseidson/brutus/tree/main/common/proto"
)

// #include "common/ffi/ffi.h"
import "C"

func main() {
	_, seg, err := capnp.NewMessage(capnp.SingleSegment(nil))
	if err != nil {
		panic(err)
	}

	cmd, err := proto.NewRootCommand(seg)
	if err != nil {
		panic(err)
	}

	_ = cmd.SetMessage_("A convincing message.")
	slog.Info("the message is", cmd.Message())

	slog.Info("hello from brutus client")

	server_pid := C.spawn_server_if_not_running()

	slog.Info("the server's pid is", "pid", server_pid)

	for {
	}
}
