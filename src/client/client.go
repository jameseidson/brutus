package client

import (
	"C"
	"log/slog"
	"os"
	"path/filepath"
	"strconv"
	"sync"
	"syscall"

	"capnproto.org/go/capnp/v3"
	"github.com/jameseidson/brutus/common/proto"
)

var Pid = sync.OnceValue[int](syscall.Getpid)

var RuntimeDir = sync.OnceValue[string](func() string {
	return filepath.Join("/var/run/user", strconv.Itoa(os.Geteuid()), "brutus")
})

var Log = sync.OnceValue[*slog.Logger](func() (logger *slog.Logger) {
	logger = slog.New(slog.NewTextHandler(os.Stderr, &slog.HandlerOptions{Level: slog.LevelDebug}))
	return
})

func testCommand() proto.Command {
	_, seg, err := capnp.NewMessage(capnp.SingleSegment(nil))
	if err != nil {
		panic(err)
	}

	cmd, err := proto.NewRootCommand(seg)
	if err != nil {
		panic(err)
	}

	cmd.SetPid(uint32(Pid()))
	cmd.SetConnect()

	return cmd
}

//export RunClient
func RunClient(serverPid uint32) {
	slog.Debug("opening fifo in client")
	path := filepath.Join(RuntimeDir(), strconv.FormatUint(uint64(serverPid), 10)+".cmd")
	cmdPipe, err := os.OpenFile(path, os.O_WRONLY, os.ModeNamedPipe)
	if err != nil {
		panic(err)
	}

	Log().Info("hello from brutus client")

	Log().Info("the server's pid is", "pid", serverPid)
	Log().Info("my pid is", "pid", Pid())

	encoder := capnp.NewEncoder(cmdPipe)
	err = encoder.Encode(testCommand().Message())
	if err != nil {
		panic(err)
	}
}
