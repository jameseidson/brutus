use std::{env::var, path::Path};

fn main() {
    let path_to_proto_label = var("BAZEL_PROTO_SRC_PATH").unwrap();
    let path_to_proto_file = Path::new(&path_to_proto_label);

    capnpc::CompilerCommand::new()
        .file(path_to_proto_file)
        .import_path(var("BAZEL_PROTO_GO_INCLUDE_PATH").unwrap())
        .src_prefix(path_to_proto_file.parent().unwrap())
        .run()
        .unwrap()
}
