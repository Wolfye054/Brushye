# Brushye MCP server

`brushye-mcp` (crate `crates/brushye-mcp`, M9) is a [Model Context Protocol](https://modelcontextprotocol.io)
server. It lets an AI client (Claude Code, Claude Desktop, any MCP client) paint and inspect
documents through Brushye's command layer. It is one more client of that layer, like the CLI and
the app: every change goes through the same commands (D-010, D-011), so everything an AI client
does is in the journal and replays exactly.

The server speaks MCP over stdio, and logs go to stderr. It runs in one of two modes:

- **Headless** (the default). It holds **one session**: a document, its undo history and
  journal, and the tool state (tools, sub tools, colours). It starts with the built-in tool set,
  not the app's saved one.
- **Attached** (`--attach`, D-022). It drives the **running Brushye window**: the open
  document, its History and Journal, and the user's tools. See below.

## Running it

Release builds include `brushye-mcp` (`brushye-mcp.exe` on Windows) next to `brushye-app`
in the Windows `.zip` and in the Linux and macOS `.tar.gz`. The Linux AppImage starts only the
app, so on Linux use the binary from the `.tar.gz`. From source:

```sh
cargo build --release -p brushye-mcp      # target/release/brushye-mcp
```

Claude Code:

```sh
claude mcp add brushye -- /path/to/brushye-mcp
```

Other clients take the usual stdio entry:

```json
{ "mcpServers": { "brushye": { "command": "/path/to/brushye-mcp" } } }
```

The file tools read and write any path the user running the server can access, as the
`brushye` CLI does. Only connect clients you trust with those files.

## Attach mode

1. In Brushye, turn on **File > Allow AI clients (MCP)**. It is off at every start. While it
   is on, the canvas status strip shows *AI clients allowed*, or *AI client connected* with a
   **Disconnect** button.
2. Start the server with `--attach`, for example
   `claude mcp add brushye-app -- /path/to/brushye-mcp --attach`. With more than one window
   allowing AI clients, pass the process id: `--attach 12345`.

In attach mode:
- Tool calls run in the app between frames, as if the user had done them. Each change is one
  History step, and undo, the Journal window and autosave all see it. A call waits while the
  user is drawing a stroke or has a dialog open.
- `doc_open` and `replay` replace the open document. If it has unsaved changes, the app asks
  the user first. Cancel returns a tool error.
- `doc_save` to the document's own file counts as saving it.
- The server connects on the first tool call and reconnects if the app restarts or AI access is
  toggled. With AI access off, tools return an error that says how to turn it on.

The endpoint is local only. On Linux and macOS it is a Unix socket in a directory private to
the user (`$XDG_RUNTIME_DIR/brushye`, or the per-user temp dir). On Windows it is a named pipe
that rejects remote clients. `BRUSHYE_RUNTIME_DIR` overrides the directory.

## Tools

| Tool | Changes state | What it does |
|---|---|---|
| `help` | no | The command text syntax (`docs/command-format.md`) and the full command reference (`docs/commands.md`). |
| `run` | yes | Runs command and query lines in the text syntax, in order. Stops at the first failing line and reports its number; earlier lines stay applied. Returns the query results as a JSON array. |
| `command` | yes | Runs one command in its JSON form `{"cmd", "v", "args"}` (`docs/command-format.md` §5). |
| `query` | no | Answers one query (`?layer.list`, `?pixel x=… y=…`, …) and returns its JSON. The `?` is optional. |
| `render` | no | Returns a PNG image: the flattened document (as exported) or one layer's own pixels (`layer`), optionally a region (`x`, `y`, `w`, `h`), downscaled to fit `max_size` (default 1024). |
| `stroke` | yes | Paints one stroke with the current tool's selected sub tool and the active colour, as drawing in the app does. The eraser tool and the transparent colour erase. It records a complete `brush.stroke` command. `samples` is `[[x, y, pressure, t_µs], …]`. |
| `journal` | no | The session journal as replayable text. |
| `doc_open` | yes | Opens a `.brsh` file. History and journal start fresh, recorded on top of the opened document (`base=` in the journal header). An embedded journal is reported, not replayed. |
| `doc_save` | no | Saves the document as `.brsh`, with `embed_journal` to store the journal too. |
| `export_png` | no | Writes the flattened document, or one layer, as a PNG file. |
| `replay` | yes | Replaces the session by replaying a journal (`text` or `path`). A journal recorded on an opened file needs that file as `base`. |

Errors come back as tool results with `isError: true` and a message. A bad command never
leaves a half-applied change, because commands are atomic.

Tool state (the selected tool, sub tools, colours) carries over when `doc_open` or `replay`
replaces the session, as it does in the app. Tool commands are not journaled (D-018).

## Example

```text
run      doc.new 800 600
         layer.new L2 name="Ink"
         layer.select L2
         tool.select pen
         subtool.set size=12 size_pressure=20
         color.set main color=#1a1a1a
stroke   samples=[[100,300,0.2,0],[200,250,0.8,16000],[300,300,1,32000],[400,350,0.3,48000]]
render   max_size=512
journal
```

More scripts are in `examples/scripts/`. The `run` tool and `brushye run SCRIPT` both run
them.

## Implementation notes

- `src/lib.rs` is the server core. It is synchronous and has no MCP types. Its tools run on a
  `Host`: the headless `Server`, or the app (`brushye-app/src/mcp_link.rs`). It is tested
  directly (`tests/server.rs`). `src/main.rs` adapts it to `rmcp` (D-016) and is built only
  with the default `server` feature; the app uses the library without it, so it doesn't link
  rmcp or tokio. `tests/protocol.rs` drives the binary over stdio with raw JSON-RPC.
- `src/attach.rs` is attach mode: the endpoint the app listens on, discovery, the wire format
  (one JSON object per line) and the client the bridge uses. `tests/attach.rs` tests it with a
  headless host standing in for the app, including the real binary in `--attach` mode.
- The tool list is fixed. Commands and queries reach clients through `run`, `command` and
  `query`, and are listed in the `command` tool's `cmd` enum and in `help`. A new command in
  the registry is available over MCP with no change to this crate.
- Document work runs on tokio's blocking pool under a mutex, so a long stroke or render doesn't
  stall the protocol (for example, cancellation and pings).
