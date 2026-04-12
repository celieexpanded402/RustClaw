use std::collections::HashMap;
use std::sync::Arc;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::Mutex;
use tracing::{debug, error, info};

use crate::config::McpServerConfig;

// ── JSON-RPC types ───────────────────────────────────────────────────

#[derive(Serialize)]
struct JsonRpcRequest {
    jsonrpc: &'static str,
    id: u64,
    method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    params: Option<serde_json::Value>,
}

#[derive(Deserialize)]
struct JsonRpcResponse {
    #[allow(dead_code)]
    id: Option<u64>,
    result: Option<serde_json::Value>,
    error: Option<JsonRpcError>,
}

#[derive(Deserialize, Debug)]
struct JsonRpcError {
    #[allow(dead_code)]
    code: i64,
    message: String,
}

// ── MCP tool definition from server ──────────────────────────────────

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct McpToolDef {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(rename = "inputSchema", default)]
    pub input_schema: serde_json::Value,
}

// ── Single MCP server connection ─────────────────────────────────────

struct McpConnection {
    #[allow(dead_code)]
    child: Child,
    stdin: tokio::process::ChildStdin,
    reader: BufReader<tokio::process::ChildStdout>,
    next_id: u64,
}

impl McpConnection {
    async fn send_request(
        &mut self,
        method: &str,
        params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value> {
        let req = JsonRpcRequest {
            jsonrpc: "2.0",
            id: self.next_id,
            method: method.to_string(),
            params,
        };
        self.next_id += 1;

        let mut line = serde_json::to_string(&req)?;
        line.push('\n');

        self.stdin
            .write_all(line.as_bytes())
            .await
            .context("Failed to write to MCP server stdin")?;
        self.stdin.flush().await?;

        // Read response line
        let mut resp_line = String::new();
        self.reader
            .read_line(&mut resp_line)
            .await
            .context("Failed to read from MCP server stdout")?;

        let resp: JsonRpcResponse =
            serde_json::from_str(&resp_line).context("Failed to parse MCP server response")?;

        if let Some(err) = resp.error {
            anyhow::bail!("MCP error: {}", err.message);
        }

        resp.result
            .ok_or_else(|| anyhow::anyhow!("MCP response has no result"))
    }
}

// ── MCP Client Manager ──────────────────────────────────────────────

pub struct McpManager {
    /// server_name → connection
    connections: HashMap<String, Arc<Mutex<McpConnection>>>,
    /// tool_name → server_name (routing table)
    tool_routes: HashMap<String, String>,
    /// All discovered MCP tool definitions
    tools: Vec<McpToolDef>,
}

impl McpManager {
    /// Connect to all configured MCP servers and discover their tools.
    pub async fn start(configs: &[McpServerConfig]) -> Result<Self> {
        let mut connections = HashMap::new();
        let mut tool_routes = HashMap::new();
        let mut tools = Vec::new();

        for cfg in configs {
            match Self::connect_server(cfg).await {
                Ok((conn, server_tools)) => {
                    info!(
                        server = %cfg.name,
                        tool_count = server_tools.len(),
                        "MCP server connected"
                    );

                    for tool in &server_tools {
                        // Prefix tool name with server name to avoid collisions
                        let qualified_name = format!("mcp_{}_{}", cfg.name, tool.name);
                        tool_routes.insert(qualified_name.clone(), cfg.name.clone());

                        tools.push(McpToolDef {
                            name: qualified_name,
                            description: tool.description.clone(),
                            input_schema: tool.input_schema.clone(),
                        });
                    }

                    connections.insert(cfg.name.clone(), Arc::new(Mutex::new(conn)));
                }
                Err(e) => {
                    error!(server = %cfg.name, %e, "Failed to connect MCP server");
                }
            }
        }

        Ok(Self {
            connections,
            tool_routes,
            tools,
        })
    }

    async fn connect_server(cfg: &McpServerConfig) -> Result<(McpConnection, Vec<McpToolDef>)> {
        let parts: Vec<&str> = cfg.command.split_whitespace().collect();
        if parts.is_empty() {
            anyhow::bail!("Empty MCP server command");
        }

        let mut cmd = Command::new(parts[0]);
        for arg in &parts[1..] {
            cmd.arg(arg);
        }

        // Pass environment variables
        for (k, v) in &cfg.env {
            cmd.env(k, v);
        }

        let mut child = cmd
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .spawn()
            .with_context(|| format!("Failed to spawn MCP server: {}", cfg.command))?;

        let stdin = child.stdin.take().context("No stdin")?;
        let stdout = child.stdout.take().context("No stdout")?;
        let reader = BufReader::new(stdout);

        let mut conn = McpConnection {
            child,
            stdin,
            reader,
            next_id: 1,
        };

        // Initialize
        let _init = conn
            .send_request(
                "initialize",
                Some(serde_json::json!({
                    "protocolVersion": "2024-11-05",
                    "capabilities": {},
                    "clientInfo": {
                        "name": "rustclaw",
                        "version": "0.1.0"
                    }
                })),
            )
            .await
            .context("MCP initialize failed")?;

        // Send initialized notification (no response expected, but send as request for simplicity)
        let _ = conn
            .send_request("notifications/initialized", None)
            .await;

        // Discover tools
        let tools_result = conn
            .send_request("tools/list", None)
            .await
            .context("MCP tools/list failed")?;

        let tool_list: Vec<McpToolDef> = tools_result
            .get("tools")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default();

        Ok((conn, tool_list))
    }

    /// Get all discovered tool definitions (for merging into tool_definitions).
    pub fn tool_definitions(&self) -> Vec<serde_json::Value> {
        self.tools
            .iter()
            .map(|t| {
                serde_json::json!({
                    "name": t.name,
                    "description": t.description.as_deref().unwrap_or(""),
                    "input_schema": t.input_schema,
                })
            })
            .collect()
    }

    /// Check if a tool name is an MCP tool.
    pub fn is_mcp_tool(&self, name: &str) -> bool {
        self.tool_routes.contains_key(name)
    }

    /// Execute an MCP tool call.
    pub async fn call_tool(&self, tool_name: &str, arguments: &serde_json::Value) -> Result<String> {
        let server_name = self
            .tool_routes
            .get(tool_name)
            .context("Unknown MCP tool")?;

        let conn = self
            .connections
            .get(server_name)
            .context("MCP server not connected")?;

        // Strip the mcp_{server}_ prefix to get original tool name
        let prefix = format!("mcp_{}_", server_name);
        let original_name = tool_name.strip_prefix(&prefix).unwrap_or(tool_name);

        debug!(server = %server_name, tool = %original_name, "Calling MCP tool");

        let mut conn = conn.lock().await;
        let result = conn
            .send_request(
                "tools/call",
                Some(serde_json::json!({
                    "name": original_name,
                    "arguments": arguments,
                })),
            )
            .await?;

        // Extract text content from MCP response
        let content = result
            .get("content")
            .and_then(|c| c.as_array())
            .map(|blocks| {
                blocks
                    .iter()
                    .filter_map(|b| {
                        if b.get("type")?.as_str()? == "text" {
                            b.get("text")?.as_str().map(String::from)
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            })
            .unwrap_or_else(|| result.to_string());

        Ok(content)
    }

    /// Number of connected servers.
    pub fn server_count(&self) -> usize {
        self.connections.len()
    }

    /// Number of discovered tools.
    pub fn tool_count(&self) -> usize {
        self.tools.len()
    }
}
