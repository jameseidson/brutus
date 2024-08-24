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

var Log = sync.OnceValue[*slog.Logger](func() (logger *slog.Logger) {
	logger = slog.New(slog.NewTextHandler(os.Stderr, &slog.HandlerOptions{Level: slog.LevelDebug}))
	return
})

var RuntimeDir = sync.OnceValue[string](func() string {
	return filepath.Join("/var/run/user", strconv.Itoa(os.Geteuid()), "brutus")
})

func connectCmd() proto.Command {
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

func handleConnectedEvt(evtPipe *os.File) {
	msg, err := capnp.NewDecoder(evtPipe).Decode()
	if err != nil {
		panic(err)
	}

	evt, err := proto.ReadRootEvent(msg)
	if err != nil {
		panic(err)
	}
	Log().Debug("received event from server", "evt", evt.Which().String())
}

//export RunClient
func RunClient(serverPid uint32) {
	Log().Info("hello from brutus client")

	slog.Debug("opening fifo in client")
	cmdPipe, err := os.OpenFile(filepath.Join(RuntimeDir(), strconv.FormatUint(uint64(serverPid), 10)+".cmd"), os.O_WRONLY, os.ModeNamedPipe)
	defer cmdPipe.Close()
	if err != nil {
		panic(err)
	}

	Log().Info("the server's pid is", "pid", serverPid)

	// Create event pipe
	path := filepath.Join(RuntimeDir(), strconv.FormatUint(uint64(Pid()), 10)+".evt")
	syscall.Mkfifo(path, syscall.S_IRUSR|syscall.S_IWUSR)

	// Send connect command
	encoder := capnp.NewEncoder(cmdPipe)
	err = encoder.Encode(connectCmd().Message())
	if err != nil {
		panic(err)
	}

	// Open event pipe for writing
	evtPipe, err := os.OpenFile(path, os.O_RDONLY, os.ModeNamedPipe)
	defer evtPipe.Close()
	if err != nil {
		panic(err)
	}

	handleConnectedEvt(evtPipe)
}
