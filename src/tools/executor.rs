use std::sync::Arc;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::config::{EmailConfig, ToolsConfig};

use super::{discord as discord_tools, email, exec, fs, mcp::McpManager, search, system};

#[derive(Debug, Deserialize)]
pub struct ToolCall {
    pub name: String,
    pub arguments: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct ToolResult {
    pub tool_use_id: String,
    pub content: String,
}

/// Optional Discord HTTP client for Discord management tools.
pub type DiscordHttp = Option<Arc<serenity::http::Http>>;

/// Execute a tool call and return the result.
pub async fn execute_tool(
    call: &ToolCall,
    tool_id: &str,
    config: &ToolsConfig,
    discord_http: &DiscordHttp,
    email_config: &Option<EmailConfig>,
    mcp: &Option<Arc<McpManager>>,
) -> ToolResult {
    // Route MCP tools first
    if let Some(mcp_mgr) = mcp {
        if mcp_mgr.is_mcp_tool(&call.name) {
            let content = match mcp_mgr.call_tool(&call.name, &call.arguments).await {
                Ok(c) => c,
                Err(e) => format!("MCP Error: {e}"),
            };
            info!(tool = %call.name, "MCP tool executed");
            return ToolResult {
                tool_use_id: tool_id.to_string(),
                content,
            };
        }
    }

    let content = match execute_inner(call, config, discord_http, email_config).await {
        Ok(c) => truncate_result(&c),
        Err(e) => format!(
            "Error: {e}\n\nHint: The tool failed. Try a different approach or use another tool to investigate the issue first."
        ),
    };

    info!(tool = %call.name, "Tool executed");

    ToolResult {
        tool_use_id: tool_id.to_string(),
        content,
    }
}

const MAX_RESULT_CHARS: usize = 4000;

/// Truncate tool output to avoid flooding the LLM context.
fn truncate_result(s: &str) -> String {
    if s.len() <= MAX_RESULT_CHARS {
        return s.to_string();
    }
    let lines: Vec<&str> = s.lines().collect();
    let truncated = &s[..MAX_RESULT_CHARS];
    format!(
        "{truncated}\n\n... (truncated, showing {MAX_RESULT_CHARS}/{} chars, {}/{} lines)",
        s.len(),
        truncated.lines().count(),
        lines.len()
    )
}

const DANGEROUS_PATTERNS: &[&str] = &[
    "rm -rf /", "rm -rf /*", "mkfs", "dd if=", "> /dev/sd",
    "chmod -R 777 /", ":(){ :|:& };:", "shutdown", "reboot",
    "init 0", "init 6", "kill -9 1", "DROP DATABASE", "DROP TABLE",
];

async fn execute_inner(
    call: &ToolCall,
    config: &ToolsConfig,
    discord_http: &DiscordHttp,
    email_config: &Option<EmailConfig>,
) -> Result<String> {
    let args = &call.arguments;

    match call.name.as_str() {
        // ── File system tools ────────────────────────────────────────
        "read_file" => {
            let path = arg_str(args, "path")?;
            fs::read_file(&path)
        }

        "write_file" => {
            let path = arg_str(args, "path")?;
            let content = arg_str(args, "content")?;
            fs::write_file(&path, &content)?;
            Ok(format!("Written to {path}"))
        }

        "list_dir" => {
            let path = arg_str(args, "path")?;
            let depth = args
                .get("depth")
                .and_then(|v| v.as_u64())
                .unwrap_or(2) as u8;
            fs::list_dir(&path, depth)
        }

        "patch_file" => {
            let path = arg_str(args, "path")?;
            let old = arg_str(args, "old")?;
            let new = arg_str(args, "new")?;
            // Safety: verify the old string exists before patching
            let content = fs::read_file(&path)?;
            if !content.contains(&old) {
                anyhow::bail!(
                    "patch_file: pattern not found in {path}. File has {} lines. Use read_file first to check content.",
                    content.lines().count()
                );
            }
            fs::patch_file(&path, &old, &new)?;
            Ok(format!("Patched {path}: replaced '{}' → '{}'",
                if old.len() > 50 { format!("{}...", &old[..50]) } else { old },
                if new.len() > 50 { format!("{}...", &new[..50]) } else { new },
            ))
        }

        // ── Shell execution ──────────────────────────────────────────
        "run_command" => {
            if !config.allow_exec {
                anyhow::bail!("Command execution is disabled in config");
            }
            let cmd = arg_str(args, "cmd")?;
            // Block dangerous commands
            let cmd_lower = cmd.to_lowercase();
            for pattern in DANGEROUS_PATTERNS {
                if cmd_lower.contains(pattern) {
                    anyhow::bail!("Blocked dangerous command: {cmd}");
                }
            }
            let cwd = args
                .get("cwd")
                .and_then(|v| v.as_str())
                .unwrap_or(&config.workspace_dir);
            let result =
                exec::run_command(&cmd, cwd, config.exec_timeout_secs, &config.workspace_dir)
                    .await?;
            Ok(format!(
                "exit_code: {}\n--- stdout ---\n{}\n--- stderr ---\n{}",
                result.exit_code, result.stdout, result.stderr
            ))
        }

        // ── Code search ──────────────────────────────────────────────
        "search_code" => {
            let dir = arg_str(args, "dir")?;
            let pattern = arg_str(args, "pattern")?;
            let matches = search::search_in_dir(&dir, &pattern)?;
            if matches.is_empty() {
                Ok("No matches found.".to_string())
            } else {
                let mut out = String::new();
                for m in &matches {
                    out.push_str(&format!("{}:{}: {}\n", m.file, m.line_number, m.line));
                }
                Ok(out)
            }
        }

        // ── Discord management tools ─────────────────────────────────
        "discord_create_channel" => {
            let http = require_discord(discord_http)?;
            let guild_id = arg_u64(args, "guild_id")?;
            let name = arg_str(args, "name")?;
            let kind = args
                .get("kind")
                .and_then(|v| v.as_str())
                .unwrap_or("text");
            discord_tools::create_channel(http, guild_id, &name, kind).await
        }

        "discord_delete_channel" => {
            let http = require_discord(discord_http)?;
            let channel_id = arg_u64(args, "channel_id")?;
            discord_tools::delete_channel(http, channel_id).await
        }

        "discord_create_role" => {
            let http = require_discord(discord_http)?;
            let guild_id = arg_u64(args, "guild_id")?;
            let name = arg_str(args, "name")?;
            let color = args
                .get("color")
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as u32;
            discord_tools::create_role(http, guild_id, &name, color).await
        }

        "discord_set_channel_topic" => {
            let http = require_discord(discord_http)?;
            let channel_id = arg_u64(args, "channel_id")?;
            let topic = arg_str(args, "topic")?;
            discord_tools::set_channel_topic(http, channel_id, &topic).await
        }

        "discord_kick_member" => {
            let http = require_discord(discord_http)?;
            let guild_id = arg_u64(args, "guild_id")?;
            let user_id = arg_u64(args, "user_id")?;
            discord_tools::kick_member(http, guild_id, user_id).await
        }

        "discord_ban_member" => {
            let http = require_discord(discord_http)?;
            let guild_id = arg_u64(args, "guild_id")?;
            let user_id = arg_u64(args, "user_id")?;
            let reason = args
                .get("reason")
                .and_then(|v| v.as_str())
                .unwrap_or("No reason provided");
            discord_tools::ban_member(http, guild_id, user_id, reason).await
        }

        // ── System monitoring tools ───────────────────────────────────
        "process_list" => system::process_list().await,

        "process_check" => {
            let name = arg_str(args, "name")?;
            system::process_check(&name).await
        }

        "docker_status" => system::docker_status().await,

        "docker_inspect" => {
            let container = arg_str(args, "container")?;
            system::docker_inspect(&container).await
        }

        "pm2_status" => system::pm2_status().await,

        "http_ping" => {
            let url = arg_str(args, "url")?;
            system::http_ping(&url).await
        }

        "system_stats" => system::system_stats().await,

        // ── Email tools ──────────────────────────────────────────────
        "fetch_inbox" => {
            let cfg = require_email(email_config)?;
            let count = args.get("count").and_then(|v| v.as_u64()).unwrap_or(10) as u32;
            let emails = email::fetch_inbox(cfg, count).await?;
            Ok(email::format_inbox_report(&emails))
        }

        "read_email" => {
            let cfg = require_email(email_config)?;
            let uid = arg_str(args, "uid")?;
            let detail = email::read_email(cfg, &uid).await?;
            Ok(format!(
                "From: {}\nTo: {}\nSubject: {}\nDate: {}\n\n{}",
                detail.from, detail.to, detail.subject, detail.date, detail.body
            ))
        }

        "send_email" => {
            let cfg = require_email(email_config)?;
            let to = arg_str(args, "to")?;
            let subject = arg_str(args, "subject")?;
            let body = arg_str(args, "body")?;
            email::send_email(cfg, &to, &subject, &body).await
        }

        _ => Ok(format!("Unknown tool: {}", call.name)),
    }
}

fn require_email(config: &Option<EmailConfig>) -> Result<&EmailConfig> {
    config
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("Email not configured"))
}

fn require_discord(http: &DiscordHttp) -> Result<&Arc<serenity::http::Http>> {
    http.as_ref()
        .ok_or_else(|| anyhow::anyhow!("Discord not connected — cannot use Discord tools"))
}

fn arg_str(args: &serde_json::Value, key: &str) -> Result<String> {
    args.get(key)
        .and_then(|v| v.as_str())
        .map(String::from)
        .ok_or_else(|| anyhow::anyhow!("Missing argument: {key}"))
}

fn arg_u64(args: &serde_json::Value, key: &str) -> Result<u64> {
    args.get(key)
        .and_then(|v| v.as_u64().or_else(|| v.as_str().and_then(|s| s.parse().ok())))
        .ok_or_else(|| anyhow::anyhow!("Missing or invalid argument: {key}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn dangerous_patterns_block_rm_rf_root() {
        let cmd = "rm -rf /";
        assert!(DANGEROUS_PATTERNS.iter().any(|p| cmd.contains(p)));
    }

    #[test]
    fn dangerous_patterns_block_fork_bomb() {
        let cmd = ":(){ :|:& };:";
        assert!(DANGEROUS_PATTERNS.iter().any(|p| cmd.contains(p)));
    }

    #[test]
    fn dangerous_patterns_block_drop_database() {
        let cmd = "mysql -e 'DROP DATABASE production'";
        assert!(DANGEROUS_PATTERNS.iter().any(|p| cmd.contains(p)));
    }

    #[test]
    fn dangerous_patterns_allow_safe_rm() {
        let cmd = "rm file.txt";
        assert!(!DANGEROUS_PATTERNS.iter().any(|p| cmd.contains(p)));
    }

    #[test]
    fn dangerous_patterns_allow_safe_commands() {
        for cmd in &["ls -la", "cat README.md", "cargo build", "git status"] {
            assert!(!DANGEROUS_PATTERNS.iter().any(|p| cmd.contains(p)), "falsely blocked: {cmd}");
        }
    }

    #[test]
    fn truncate_short_string_unchanged() {
        let s = "hello world";
        assert_eq!(truncate_result(s), s);
    }

    #[test]
    fn truncate_long_string() {
        let s = "a".repeat(5000);
        let result = truncate_result(&s);
        assert!(result.contains("truncated"));
        assert!(result.contains("4000/5000"));
    }

    #[test]
    fn truncate_at_boundary() {
        let s = "a".repeat(4000);
        assert_eq!(truncate_result(&s), s);
    }

    #[test]
    fn arg_str_extracts_value() {
        let args = json!({"path": "/tmp/file.txt"});
        assert_eq!(arg_str(&args, "path").unwrap(), "/tmp/file.txt");
    }

    #[test]
    fn arg_str_missing_key_errors() {
        let args = json!({"other": "value"});
        assert!(arg_str(&args, "path").is_err());
    }

    #[test]
    fn arg_u64_from_number() {
        let args = json!({"id": 42});
        assert_eq!(arg_u64(&args, "id").unwrap(), 42);
    }

    #[test]
    fn arg_u64_from_string() {
        let args = json!({"id": "123"});
        assert_eq!(arg_u64(&args, "id").unwrap(), 123);
    }

    #[test]
    fn arg_u64_invalid_errors() {
        let args = json!({"id": "not_a_number"});
        assert!(arg_u64(&args, "id").is_err());
    }
}

/// All tool definitions: local + MCP. This is what gets sent to the LLM.
pub fn all_tool_definitions(mcp: &Option<Arc<McpManager>>) -> serde_json::Value {
    let mut local = tool_definitions();
    if let Some(mcp_mgr) = mcp {
        let mcp_tools = mcp_mgr.tool_definitions();
        if let Some(arr) = local.as_array_mut() {
            arr.extend(mcp_tools);
        }
    }
    local
}

/// JSON Schema definitions for local (built-in) tools.
pub fn tool_definitions() -> serde_json::Value {
    serde_json::json!([
        {
            "name": "read_file",
            "description": "Read the contents of a file",
            "input_schema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "File path to read" }
                },
                "required": ["path"]
            }
        },
        {
            "name": "write_file",
            "description": "Write content to a file, creating parent directories as needed",
            "input_schema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "File path to write" },
                    "content": { "type": "string", "description": "Content to write" }
                },
                "required": ["path", "content"]
            }
        },
        {
            "name": "list_dir",
            "description": "List directory contents in tree format",
            "input_schema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Directory path" },
                    "depth": { "type": "integer", "description": "Max depth (1-3, default 2)" }
                },
                "required": ["path"]
            }
        },
        {
            "name": "patch_file",
            "description": "Replace the first occurrence of a string in a file",
            "input_schema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "File path to patch" },
                    "old": { "type": "string", "description": "Text to find" },
                    "new": { "type": "string", "description": "Replacement text" }
                },
                "required": ["path", "old", "new"]
            }
        },
        {
            "name": "run_command",
            "description": "Execute a shell command",
            "input_schema": {
                "type": "object",
                "properties": {
                    "cmd": { "type": "string", "description": "Shell command to execute" },
                    "cwd": { "type": "string", "description": "Working directory (optional)" }
                },
                "required": ["cmd"]
            }
        },
        {
            "name": "search_code",
            "description": "Search for a pattern in files within a directory",
            "input_schema": {
                "type": "object",
                "properties": {
                    "dir": { "type": "string", "description": "Directory to search in" },
                    "pattern": { "type": "string", "description": "Search pattern (substring)" }
                },
                "required": ["dir", "pattern"]
            }
        },
        {
            "name": "discord_create_channel",
            "description": "Create a new Discord channel in a guild",
            "input_schema": {
                "type": "object",
                "properties": {
                    "guild_id": { "type": "string", "description": "Guild (server) ID" },
                    "name": { "type": "string", "description": "Channel name" },
                    "kind": { "type": "string", "description": "Channel type: text, voice, or category", "enum": ["text", "voice", "category"] }
                },
                "required": ["guild_id", "name"]
            }
        },
        {
            "name": "discord_delete_channel",
            "description": "Delete a Discord channel",
            "input_schema": {
                "type": "object",
                "properties": {
                    "channel_id": { "type": "string", "description": "Channel ID to delete" }
                },
                "required": ["channel_id"]
            }
        },
        {
            "name": "discord_create_role",
            "description": "Create a new role in a Discord guild",
            "input_schema": {
                "type": "object",
                "properties": {
                    "guild_id": { "type": "string", "description": "Guild (server) ID" },
                    "name": { "type": "string", "description": "Role name" },
                    "color": { "type": "integer", "description": "Role color as decimal (e.g. 16711680 for red)" }
                },
                "required": ["guild_id", "name"]
            }
        },
        {
            "name": "discord_set_channel_topic",
            "description": "Set the topic of a Discord text channel",
            "input_schema": {
                "type": "object",
                "properties": {
                    "channel_id": { "type": "string", "description": "Channel ID" },
                    "topic": { "type": "string", "description": "New topic text" }
                },
                "required": ["channel_id", "topic"]
            }
        },
        {
            "name": "discord_kick_member",
            "description": "Kick a member from a Discord guild",
            "input_schema": {
                "type": "object",
                "properties": {
                    "guild_id": { "type": "string", "description": "Guild (server) ID" },
                    "user_id": { "type": "string", "description": "User ID to kick" }
                },
                "required": ["guild_id", "user_id"]
            }
        },
        {
            "name": "discord_ban_member",
            "description": "Ban a member from a Discord guild",
            "input_schema": {
                "type": "object",
                "properties": {
                    "guild_id": { "type": "string", "description": "Guild (server) ID" },
                    "user_id": { "type": "string", "description": "User ID to ban" },
                    "reason": { "type": "string", "description": "Reason for ban (optional)" }
                },
                "required": ["guild_id", "user_id"]
            }
        },
        // ── System monitoring tools
        {
            "name": "process_list",
            "description": "List running processes sorted by memory usage",
            "input_schema": { "type": "object", "properties": {} }
        },
        {
            "name": "process_check",
            "description": "Check if a specific process is running",
            "input_schema": {
                "type": "object",
                "properties": {
                    "name": { "type": "string", "description": "Process name to check" }
                },
                "required": ["name"]
            }
        },
        {
            "name": "docker_status",
            "description": "List all Docker containers with their status",
            "input_schema": { "type": "object", "properties": {} }
        },
        {
            "name": "docker_inspect",
            "description": "Inspect a specific Docker container's status",
            "input_schema": {
                "type": "object",
                "properties": {
                    "container": { "type": "string", "description": "Container name or ID" }
                },
                "required": ["container"]
            }
        },
        {
            "name": "pm2_status",
            "description": "List all PM2 managed processes with status, CPU, memory",
            "input_schema": { "type": "object", "properties": {} }
        },
        {
            "name": "http_ping",
            "description": "Check if an HTTP endpoint is reachable and return status code",
            "input_schema": {
                "type": "object",
                "properties": {
                    "url": { "type": "string", "description": "URL to ping" }
                },
                "required": ["url"]
            }
        },
        {
            "name": "system_stats",
            "description": "Get system resource summary: uptime, disk, memory",
            "input_schema": { "type": "object", "properties": {} }
        },
        // ── Email tools
        {
            "name": "fetch_inbox",
            "description": "Fetch latest emails from inbox via IMAP",
            "input_schema": {
                "type": "object",
                "properties": {
                    "count": { "type": "integer", "description": "Number of emails to fetch (default 10)" }
                }
            }
        },
        {
            "name": "read_email",
            "description": "Read the full content of a specific email by UID",
            "input_schema": {
                "type": "object",
                "properties": {
                    "uid": { "type": "string", "description": "Email UID/sequence number" }
                },
                "required": ["uid"]
            }
        },
        {
            "name": "send_email",
            "description": "Send an email via SMTP",
            "input_schema": {
                "type": "object",
                "properties": {
                    "to": { "type": "string", "description": "Recipient email address" },
                    "subject": { "type": "string", "description": "Email subject" },
                    "body": { "type": "string", "description": "Email body text" }
                },
                "required": ["to", "subject", "body"]
            }
        }
    ])
}
