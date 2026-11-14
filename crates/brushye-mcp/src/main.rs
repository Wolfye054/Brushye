//! `brushye-mcp` — the Brushye MCP server over stdio (M9).
//!
//! Configure an MCP client to start this binary. By default it serves one headless Brushye
//! session; with `--attach [PID]` it forwards every tool call to a running Brushye window that
//! allows AI clients (D-022). The tools are listed in `docs/mcp.md`. Logs go to stderr; stdout
//! carries the protocol.

#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::sync::{Arc, Mutex};

use brushye_mcp::attach::{self, Client};
use brushye_mcp::{Host as _, INSTRUCTIONS, Out, Server, png_base64, tools};
use rmcp::model::{
    CallToolRequestParams, CallToolResponse, CallToolResult, ContentBlock, Implementation,
    ListToolsResult, PaginatedRequestParams, ServerCapabilities, ServerConfig, Tool,
    ToolAnnotations,
};
use rmcp::service::RequestContext;
use rmcp::{ErrorData as McpError, RoleServer, ServerHandler, ServiceExt as _};

/// Where tool calls go.
enum Backend {
    /// A headless session in this process.
    Local(Server),
    /// A running app (attach mode), connected on first use and again after a disconnect.
    Attached {
        pid: Option<u32>,
        client: Option<Client>,
    },
}

impl Backend {
    fn call(
        &mut self,
        name: &str,
        args: &serde_json::Map<String, serde_json::Value>,
    ) -> Result<Vec<Out>, String> {
        match self {
            Backend::Local(s) => s.call(name, args),
            Backend::Attached { pid, client } => {
                // One retry: the app may have restarted or toggled AI access since the last call.
                for attempt in 0..2 {
                    if client.is_none() {
                        *client = Some(Client::connect(&attach::runtime_dir(), *pid)?);
                    }
                    let Some(c) = client.as_mut() else { break };
                    match c.call(name, args) {
                        Ok(r) => return r,
                        Err(e) => {
                            *client = None;
                            if attempt == 1 {
                                return Err(format!("lost the connection to Brushye: {e}"));
                            }
                        }
                    }
                }
                Err(attach::NOT_LISTENING.into())
            }
        }
    }
}

#[derive(Clone)]
struct Handler {
    backend: Arc<Mutex<Backend>>,
    attached: bool,
}

impl ServerHandler for Handler {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(
                Implementation::new("brushye", env!("CARGO_PKG_VERSION")).with_title("Brushye"),
            )
            .with_instructions(if self.attached {
                format!(
                    "{INSTRUCTIONS}\n\nThis server is attached to a running Brushye window: \
                     your commands change the user's open document and appear in its History. \
                     Opening or replaying a document asks the user first if there are unsaved \
                     changes."
                )
            } else {
                INSTRUCTIONS.to_owned()
            })
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, McpError> {
        let list = tools()
            .into_iter()
            .map(|t| {
                let schema = match t.schema {
                    serde_json::Value::Object(m) => m,
                    _ => serde_json::Map::new(),
                };
                let mut tool = Tool::new(t.name, t.description, schema);
                tool.annotations = Some(ToolAnnotations::from_raw(
                    None,
                    Some(t.read_only),
                    Some(false),
                    None,
                    Some(false),
                ));
                tool
            })
            .collect();
        Ok(ListToolsResult::with_all_items(list))
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResponse, McpError> {
        let backend = Arc::clone(&self.backend);
        let name = request.name.to_string();
        let args = request.arguments.unwrap_or_default();
        // Document work (strokes, rendering) is CPU-bound: keep it off the protocol thread.
        let result = tokio::task::spawn_blocking(move || {
            let mut b = backend
                .lock()
                .map_err(|_| "the session was poisoned by an earlier panic".to_owned())?;
            b.call(&name, &args)
        })
        .await
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;
        Ok(match result {
            Ok(out) => CallToolResult::success(
                out.into_iter()
                    .map(|o| match o {
                        Out::Text(t) => ContentBlock::text(t),
                        Out::Png(p) => ContentBlock::image(png_base64(&p), "image/png"),
                    })
                    .collect(),
            ),
            Err(msg) => CallToolResult::error(vec![ContentBlock::text(msg)]),
        }
        .into())
    }
}

const USAGE: &str = "usage: brushye-mcp [--attach [PID]] | --version
  (no option)     serve a headless Brushye session over stdio
  --attach [PID]  forward tool calls to the running Brushye window (File > Allow AI clients)";

fn main() -> std::process::ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let backend = match args
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
    {
        [] => Backend::Local(Server::new()),
        ["--version"] => {
            println!("brushye-mcp {}", env!("CARGO_PKG_VERSION"));
            return std::process::ExitCode::SUCCESS;
        }
        ["--attach"] => Backend::Attached {
            pid: None,
            client: None,
        },
        ["--attach", p] if p.parse::<u32>().is_ok() => Backend::Attached {
            pid: p.parse().ok(),
            client: None,
        },
        _ => {
            eprintln!("{USAGE}");
            return std::process::ExitCode::from(2);
        }
    };
    let attached = matches!(backend, Backend::Attached { .. });
    let rt = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(e) => {
            eprintln!("brushye-mcp: {e}");
            return std::process::ExitCode::FAILURE;
        }
    };
    let handler = Handler {
        backend: Arc::new(Mutex::new(backend)),
        attached,
    };
    let res: Result<(), String> = rt.block_on(async move {
        let running = handler
            .serve(rmcp::transport::stdio())
            .await
            .map_err(|e| e.to_string())?;
        running.waiting().await.map_err(|e| e.to_string())?;
        Ok(())
    });
    match res {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("brushye-mcp: {e}");
            std::process::ExitCode::FAILURE
        }
    }
}
