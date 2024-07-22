package client

import (
	"C"
	"capnproto.org/go/capnp/v3"
	"github.com/jameseidson/brutus/tree/main/common/proto"
	"log/slog"
)

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
	slog.Info("the message is", cmd.Message())

	slog.Info("hello from brutus client")

	slog.Info("the server's pid is", "pid", server_pid)

	for {
	}
}
