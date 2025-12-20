vim.lsp.start({
	name = "test_language_server",
	cmd = { "target/debug/lc3_lsp" },
	-- filetypes = { "lua" },
	root_dir = vim.fs.dirname(vim.fs.find({ "Cargo.toml" }, { upward = true })[1]),
})

vim.api.nvim_create_autocmd('LspAttach', {
  callback = function(args)
    print(vim.inspect(args))
  end,
})
