use std::{
    env::var,
    path::{Path, PathBuf},
};

fn main() {
    let proto_dir = Path::new(&var("BAZEL_PROTO_ROOT_SRC").unwrap())
        .parent()
        .unwrap()
        .to_owned();

    let proto_srcs = var("BAZEL_PROTO_SRCS").unwrap();

    let mut cmd = capnpc::CompilerCommand::new();

    cmd.capnp_executable(var("BAZEL_CAPNP_COMPILER_PATH").unwrap())
        .import_path(var("BAZEL_PROTO_GO_INCLUDE_PATH").unwrap())
        .src_prefix(&proto_dir);

    proto_srcs.split_whitespace().for_each(|src| {
        cmd.file(proto_dir.join(PathBuf::from(src).strip_prefix("src/common/proto").unwrap()));
    });

    cmd.run().unwrap();
}
