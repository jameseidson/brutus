# Brutus

Brutus is a terminal multiplexer.

## Building

Brutus uses the [Bazel](https://bazel.build/) build system.

```
bazel build //src:brutus
```

Will produce a binary at `bazel-bin/src/brutus`.

If you have a system installation of [Cap'n Proto](https://capnproto.org/install.html), you can tell bazel to use it by adding the flag `--//:use-system-capnp=true`. Otherwise, the Cap'n Proto compiler will be built from source (this can take some time).

## Development

### Editor Setup

Getting language servers to work well with Bazel and generated code requires some setup.

1. Make sure you're using [rust-analyzer](https://rust-analyzer.github.io/) for rust and [gopls](https://pkg.go.dev/golang.org/x/tools/gopls) for go.

2. Run `gen-project-for-rust-analyzer` to generate a `rust-project.json` in the repository root.

   ```
   bazel run //:gen-project-for-rust-analyzer
   ```

   This file supplies Bazel build information to rust-analyzer and will be automatically discovered. Documentation can be found [here](https://rust-analyzer.github.io/manual.html#non-cargo-based-projects).

3. Apply the following language server configurations as appropriate for your editor. The scripts used in these configurations assume you have the [`capnp`](https://capnproto.org/capnp-tool.html) binary on your `$PATH`.

   ```
   "rust-analyzer.check.overrideCommand": ["scripts/rustanalyzer-bazel-bridge.sh"],
   ```

   ```
   "gopls.env": {
     "GOPACKAGESDRIVER": "<ABSOLUTE PATH TO REPOSITORY>/scripts/gopls-bazel-bridge.sh",
   }
   ```

   - **VS Code** is already configured via `.vscode/settings.json`.
   - **Neovim** is already configured via `.nvim.lua`. This requires [`vim.opt.exrc = true`](https://neovim.io/doc/user/options.html#'exrc') in your global Neovim configuration.

### Workflows

Update Bazel dependencies from `src/server/Cargo.toml`:

```
CARGO_BAZEL_REPIN=1 bazel sync --only=crate_index
```
