local lspconfig = require("lspconfig")

lspconfig.rust_analyzer.setup({
	settings = {
		["rust-analyzer"] = {
			check = {
				overrideCommand = { vim.loop.cwd() .. "scripts/rustanalyzer-bazel-bridge.sh" },
			},
		},
	},
})

lspconfig.gopls.setup({
	settings = {
		gopls = {
			env = {
				["GOPACKAGESDRIVER_BAZEL_QUERY"] = "kind(go_binary, //...)",
				["GOPACKAGESDRIVER"] = vim.loop.cwd() .. "/scripts/gopls-bazel-bridge.sh",
			},
		},
	},
})
