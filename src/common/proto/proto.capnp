@0xc085da025837ad57;

using Go = import "/go.capnp";
using ServerEvent = import "event/server.capnp";
using ClientEvent = import "event/client.capnp";

$Go.package("proto");
$Go.import("github.com/jameseidson/brutus/common/proto");

struct Command {
  msg @0 :Text;
}
