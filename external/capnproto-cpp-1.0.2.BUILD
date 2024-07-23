load("@rules_foreign_cc//foreign_cc:defs.bzl", "configure_make")

filegroup(
    name = "compiler",
    srcs = [":capnproto"],
    output_group = "capnp",
    visibility = ["//visibility:public"],
)

configure_make(
    name = "capnproto",
    configure_options = ["--disable-shared"],
    lib_source = "//:all",
    out_binaries = ["capnp"],
    targets = ["install"],
    visibility = ["//visibility:private"],
)

filegroup(
    name = "all",
    srcs = glob(["**"]),
    visibility = ["//visibility:private"],
)
