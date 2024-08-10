@0xc085da025837ad57;

using Go = import "/go.capnp";
using Cmd = import "command.capnp";
using Evt = import "event.capnp";

$Go.package("proto");
$Go.import("github.com/jameseidson/brutus/common/proto");

struct Command {
  pid @0 : UInt32;

  union {
    connect @1: Void;
    empty @2: Void;
  }
}

struct Event {
  union {
    connected @0: Void;
    empty @1: Void;
  }
}
