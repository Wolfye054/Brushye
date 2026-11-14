# Script examples

Brushye scripts are plain text in the command syntax (`docs/command-format.md`): one command or
query per line, `#` comments, queries start with `?` and print JSON. The command reference is
`docs/commands.md` (or `brushye commands --markdown`).

Run one with the CLI:

```sh
brushye run examples/scripts/strokes-and-undo.bcmd --png out.png --journal out.journal
```

or send its text to the MCP server's `run` tool (`docs/mcp.md`). A test runs every
`*.bcmd` file here (`crates/brushye-mcp/tests/server.rs`), so they stay valid.

| Script | Shows |
|---|---|
| `layers-and-blending.bcmd` | layers, fills, raw pixels, blending modes and opacity, queries |
| `strokes-and-undo.bcmd` | brush strokes from readable pen samples, pressure, undo and redo |
| `subtools.bcmd` | tool state as commands: adding and tuning a sub tool, colours |
