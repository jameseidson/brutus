@0xfa5b5b07b0aff456;

using Go = import "/go.capnp";

$Go.package("proto");
$Go.import("github.com/jameseidson/brutus/common/proto");

struct Event {
  union {
    connected @0: Void;
    empty @1: Void;
  }
}
