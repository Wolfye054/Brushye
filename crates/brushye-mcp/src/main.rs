//! `brushye-mcp` — the Brushye MCP server over stdio (M9).
//!
//! Configure an MCP client to start this binary; it serves one headless Brushye session.
//! The tools are listed in `docs/mcp.md`. Logs go to stderr; stdout carries the protocol.

#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::sync::{Arc, Mutex};

use brushye_mcp::{INSTRUCTIONS, Out, Server, png_base64, tools};
use rmcp::model::{
    CallToolRequestParams, CallToolResponse, CallToolResult, ContentBlock, Implementation,
    ListToolsResult, PaginatedRequestParams, ServerCapabilities, ServerConfig, Tool,
    ToolAnnotations,
};
use rmcp::service::RequestContext;
use rmcp::{ErrorData as McpError, RoleServer, ServerHandler, ServiceExt as _};

#[derive(Clone)]
struct Handler {
    server: Arc<Mutex<Server>>,
}

impl ServerHandler for Handler {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(
                Implementation::new("brushye", env!("CARGO_PKG_VERSION")).with_title("Brushye"),
            )
            .with_instructions(INSTRUCTIONS)
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
        let server = Arc::clone(&self.server);
        let name = request.name.to_string();
        let args = request.arguments.unwrap_or_default();
        // Document work (strokes, rendering) is CPU-bound: keep it off the protocol thread.
        let result = tokio::task::spawn_blocking(move || {
            let mut s = server
                .lock()
                .map_err(|_| "the session was poisoned by an earlier panic".to_owned())?;
            s.call(&name, &args)
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

fn main() -> std::process::ExitCode {
    if std::env::args().nth(1).as_deref() == Some("--version") {
        println!("brushye-mcp {}", env!("CARGO_PKG_VERSION"));
        return std::process::ExitCode::SUCCESS;
    }
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
        server: Arc::new(Mutex::new(Server::new())),
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
