# M9 report: MCP server and scripting polish (2026-11-14)

**Status: done, apart from the maintainer's checks on native Windows and macOS** (attach mode in
the real app). Everything else is in and pushed, and CI is green on Windows, macOS and Linux.
Decisions: D-022 (attach mode, proposed and accepted this milestone). Dependencies:
`rmcp`, `tokio` and `interprocess` (D-016 rows); 0BSD added to the `deny.toml` allow list.

## What works
- **MCP server** (`crates/brushye-mcp`, `docs/mcp.md`). `brushye-mcp` speaks MCP over stdio
  with `rmcp` 3.10.
  - **Tools:**
    - `help`: the command syntax and full reference.
    - `run`: text command and query lines.
    - `command`: one command in its JSON form.
    - `query`.
    - `render`: returns a PNG image, so the AI can look at the canvas.
    - `stroke`: paints with the current sub tool and colour, as the app does.
    - `journal`, `doc_open`, `doc_save`, `export_png`, `replay`.
  - **Through the command layer:** every change goes through it, so everything a client does
    is journaled and replays exactly. A new command in the registry is available over MCP with
    no change to the server.
  - **Errors:** the failing line and its message come back as a tool error. Commands are
    atomic, so nothing is left half-applied.
- **Attach mode** (D-022). An AI client can drive the running app, painting on the canvas
  the user sees.
  - **Turning it on:** *File > Allow AI clients (MCP)*. It is off at every start and never
    saved. The client then runs `brushye-mcp --attach [PID]`.
  - **Execution:** calls run on the UI thread between frames, so each change is a normal
    History step and shows in undo, the Journal window and autosave. A call waits while the
    user is drawing or has a dialog open.
  - **Unsaved changes:** `doc_open` and `replay` over unsaved changes ask the user first. Cancel
    returns a tool error.
  - **Indicator:** the canvas status strip shows *AI clients allowed*, or *AI client connected*
    with a Disconnect button.
  - **Endpoint:** local only.
    - Linux/macOS: a Unix socket in a private 0700 directory. The socket is also 0600 on Linux.
    - Windows: a named pipe that rejects remote clients.
  - **Reconnecting:** the bridge connects on first use and reconnects after the app restarts
    or AI access is toggled.
- **Journal viewer**:
  - `brushye journal show JOURNAL|FILE.brsh [--json]` lists the commands. Stroke samples are
    summarised (count, duration, bounds, pressure range) and undone steps are marked.
  - In the app, *Window > Journal (Brushye)* shows the same list, live, with undone steps
    greyed and a Copy button. It sits next to Auto action, since CSP has no journal.
  - Both use `brushye_command::journal`. A test checks its undo state against the session's
    own history.
- **Script examples** (`examples/scripts/*.bcmd`): layers and blending, strokes with undo and
  redo, and sub tools and colours. A test runs every one through the MCP `run` tool.
- **Command layer:** in the JSON form, stroke samples may be `[[x,y,p,t],…]` arrays, as
  `docs/command-format.md` already said for matrices. The printer still writes the blob.
- **Release builds** include `brushye-mcp` in the Windows `.zip` and the Linux/macOS
  `.tar.gz`.

## Verification
- **Tests:** 164 workspace tests pass on all three CI platforms. For M9:
  - The server core is tested directly, with 7 tests.
  - The binary is driven over stdio with raw JSON-RPC.
  - Attach mode is tested end to end, with a headless host standing in for the app and the
    real binary in `--attach` mode. This covers discovery, two hosts, turning off and
    reconnecting.
  - The journal view model and `journal show` have their own tests.
- **In the app** (Linux, Xvfb, automated):
  - Window > Journal lists strokes and greys an undone one.
  - An MCP client added a layer, set the pen and colour, and painted a stroke. The palettes
    mirrored its tool changes.
  - The open-over-unsaved confirmation works with both Cancel and Discard.
  - Turn off removes the endpoint, and the client is told how to turn it back on.
- **CI found two macOS issues,** both fixed:
  - macOS has no socket file mode. It falls back to the 0700 directory.
  - Accepted sockets inherit non-blocking mode. Each connection is now set to blocking.

## What doesn't work yet / limits
- **Not checked natively on Windows or macOS** in the real app. CI runs the attach tests there,
  but not the app UI. On Windows the pipe keeps its default security, which gives other local
  users read-only access, so they can't send requests (D-022, implementation notes).
- **The AppImage** contains `brushye-mcp` but only starts the app. On Linux, use the binary
  from the `.tar.gz`.
- **The release workflow** with `brushye-mcp` hasn't run yet. It runs at the next tag.
- **The headless server** starts with the built-in tool set, not the app's saved sub tools
  (by design: replay starts from the defaults, D-018).
- **No MCP resources or prompts:** tools only, with `help` serving the reference.
- **Journal window:** long stroke lines scroll sideways. There is no filtering or search yet.

## Waiting on the maintainer
- **A native check of attach mode on Windows and macOS:**
  1. Turn on File > Allow AI clients.
  2. Connect `brushye-mcp --attach` from an MCP client (e.g. `claude mcp add brushye-app --
     /path/to/brushye-mcp --attach`).
  3. Paint with `stroke`.
  4. Try `doc_open` with unsaved changes.
  5. Disconnect.
- **Observation specs:** none new in M9. Still open from M8: B-014 (thin-tip AA rule, analysis
  only) and B-015 (soft-tip offset switch sizes, size 8).

## Proposed next step
**M10 part 1: layer structure.** Folders with Through mode, clip to layer below, lock
transparent pixels (already a layer flag, exposed fully), and layer masks.
- Folder/Through (L-004) and clipping (L-006) are already **observed**, so the compositor work
  can start from exact data.
- Layer masks need a new observation spec (mask painting, mask + clipping, mask on folders)
  before they're implemented.
- This comes first because `.clip` import (M11) and most other M10 features (selections, fill,
  transform) work on this structure.
- It changes the document model and the `.brsh` format (new layer kinds and fields). Per the
  working agreement, I'll write that up as a proposal for approval before coding.
