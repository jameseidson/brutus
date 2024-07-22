#!/usr/bin/env bash
exec bazel build --//:use-system-capnp=true --@rules_rust//:error_format=json --aspects=@rules_rust//rust:defs.bzl%rust_clippy_aspect --output_groups=clippy_checks //server "${@}"
