use std::{env::var, path::Path};

fn main() {
    let proto_dir = Path::new(&var("BAZEL_PROTO_EVENT_SRC").unwrap())
        .parent()
        .unwrap()
        .to_owned();

    capnpc::CompilerCommand::new()
        .capnp_executable(var("BAZEL_CAPNP_COMPILER_PATH").unwrap())
        .import_path(var("BAZEL_PROTO_GO_INCLUDE_PATH").unwrap())
        .src_prefix(&proto_dir)
        .file(var("BAZEL_PROTO_EVENT_SRC").unwrap())
        .file(var("BAZEL_PROTO_COMMAND_SRC").unwrap())
        .run()
        .unwrap()
}
