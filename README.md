# Brutus

Brutus is a terminal multiplexer.

## Development

### Editor Setup

Getting LSP servers to work well with Bazel and generated code requires some setup.

1. Make sure you're using [rust-analyzer](https://rust-analyzer.github.io/) for rust and [gopls](https://pkg.go.dev/golang.org/x/tools/gopls) for go.

2. Run `gen-project-for-rust-analyzer` to generate a `rust-project.json` in the repository root.

   ```
   bazel run //:gen-project-for-rust-analyzer
   ```

   This file supplies Bazel build information to rust-analyzer and will be automatically discovered. Documentation can be found [here](https://rust-analyzer.github.io/manual.html#non-cargo-based-projects).

3. Apply the following language server configuration as appropriate for your editor.

   ```
   "rust-analyzer.check.overrideCommand": ["scripts/rustanalyzer-bazel-bridge.sh"],
   ```

   ```
   "gopls.env": {
     "GOPACKAGESDRIVER_BAZEL_QUERY": "kind(go_binary, //...)",
     "GOPACKAGESDRIVER": "<PATH TO REPOSITORY>/scripts/gopls-bazel-bridge.sh",
   }
   ```

   - **VS Code** is already configured via `.vscode/settings.json`.
   - **Neovim** is already configured via `.nvim.lua`. This requires [`vim.opt.exrc = true`](https://neovim.io/doc/user/options.html#'exrc') in your global Neovim configuration.
