package client

import (
	"C"
	"log/slog"
	"os"
	"path/filepath"
	"strconv"
	"sync"

	"capnproto.org/go/capnp/v3"
	"github.com/jameseidson/brutus/common/proto"
)

var RuntimeDir = sync.OnceValue[string](func() string {
	return filepath.Join("/var/run/user", strconv.Itoa(os.Geteuid()), "brutus")
})

var CmdPipe = sync.OnceValue[*os.File](func() *os.File {
	slog.Debug("opening fifo in client")
	path := filepath.Join(RuntimeDir(), "cmd.pipe")
	pipe, err := os.OpenFile(path, os.O_WRONLY, os.ModeNamedPipe)
	if err != nil {
		panic(err)
	}
	return pipe
})

var Log = sync.OnceValue[*slog.Logger](func() (logger *slog.Logger) {
	logger = slog.New(slog.NewTextHandler(os.Stderr, &slog.HandlerOptions{Level: slog.LevelDebug}))
	return
})

func testCommand() (cmd proto.Command) {
	_, seg, err := capnp.NewMessage(capnp.SingleSegment(nil))
	if err != nil {
		panic(err)
	}

	cmd, err = proto.NewRootCommand(seg)
	if err != nil {
		panic(err)
	}

	cmd.SetMsg("Hello from client :)")

	return
}

//export RunClient
func RunClient(server_pid uint32) {
	Log().Info("hello from brutus client")

	Log().Info("the server's pid is", "pid", server_pid)

	encoder := capnp.NewEncoder(CmdPipe())
	err := encoder.Encode(testCommand().Message())
	if err != nil {
		panic(err)
	}
}
