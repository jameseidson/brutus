#!/usr/bin/env bash
exec bazel run --//:use-system-capnp=true @io_bazel_rules_go//go/tools/gopackagesdriver "${@}"
