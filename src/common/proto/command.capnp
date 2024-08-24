@0x959bbfcf439b0ba9;

using Go = import "/go.capnp";

$Go.package("proto");
$Go.import("github.com/jameseidson/brutus/common/proto");

struct Command {
  pid @0 :UInt32;

  union {
    connect @1 :Void;
    empty @2 :Void;
  }
}
