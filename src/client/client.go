package client

import (
	"log/slog"
	"unsafe"

	"capnproto.org/go/capnp/v3"
	"github.com/jameseidson/brutus/tree/main/src/common/proto"
)

// #include <stdlib.h>
// #include "src/common/ipc/include/ipc.h"
import "C"

//export RunClient
func RunClient(server_pid uint32) {
	_, seg, err := capnp.NewMessage(capnp.SingleSegment(nil))
	if err != nil {
		panic(err)
	}

	cmd, err := proto.NewRootCommand(seg)
	if err != nil {
		panic(err)
	}

	_ = cmd.SetMessage_("A convincing message.")

	slog.Info("hello from brutus client")

	slog.Info("the server's pid is", "pid", server_pid)

	data := []uint8{0, 0, 0, 0}
	ipc := C.ipc_open(C.CString("testhandle"), 4)
	slog.Debug("back in GO after opening IPC")

	C.ipc_recv(ipc, unsafe.Pointer(&data[0]))

	slog.Debug("the data is: ", data)

	for {
	}
}
