load("@rules_foreign_cc//foreign_cc:defs.bzl", "configure_make")

filegroup(
    name = "compiler",
    srcs = [":capnproto"],
    output_group = "capnp",
    visibility = ["//visibility:public"],
)

configure_make(
    name = "capnproto",
    lib_source = "//:all",
    out_binaries = ["capnp"],
    configure_options = ["--disable-shared"],
    targets = ["install"],
    visibility = ["//visibility:private"],
)

filegroup(
    name = "all",
    srcs = glob(["**"]),
    visibility = ["//visibility:private"],
)
