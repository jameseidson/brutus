using Go = import "/go.capnp";

@0xc085da025837ad57;

$Go.package("proto");
$Go.import("bazel-out/k8-fastbuild/bin/common/proto");

struct Command {
  message @0 :Text;
}
