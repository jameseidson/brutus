using Go = import "/go.capnp";

@0xc085da025837ad57;

$Go.package("proto");
$Go.import("github.com/jameseidson/brutus/tree/main/src/common/proto");

struct Command {
  msg @0 :Text;
}