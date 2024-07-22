local lspconfig = require("lspconfig")

lspconfig.rust_analyzer.setup({
	settings = {
		["rust-analyzer"] = {
			check = {
				overrideCommand = { "scripts/rustanalyzer-bazel-bridge.sh" },
			},
		},
	},
})

lspconfig.gopls.setup({
	settings = {
		gopls = {
			env = {
				["GOPACKAGESDRIVER"] = vim.loop.cwd() .. "/scripts/gopls-bazel-bridge.sh",
			},
		},
	},
})
