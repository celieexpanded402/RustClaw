use anyhow::{bail, Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tracing::{debug, info, warn};

use crate::config::{AgentConfig, EmailConfig, ToolsConfig};
use crate::tools::executor::{self, DiscordHttp, ToolCall};

const MAX_TOOL_ITERATIONS: usize = 10;

// ── Public types ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

pub struct AgentRunner {
    config: AgentConfig,
    client: Client,
}

impl AgentRunner {
    pub fn new(config: AgentConfig) -> Self {
        Self {
            config,
            client: Client::new(),
        }
    }

    pub fn config(&self) -> &AgentConfig {
        &self.config
    }

    /// Streaming call (no tool use) — invokes callback for each token.
    pub async fn run_streaming(
        &self,
        input: &str,
        history: &[Message],
        on_token: impl FnMut(String) + Send,
    ) -> Result<String> {
        match self.config.provider.as_str() {
            "anthropic" => self.stream_anthropic(input, history, on_token).await,
            "openai" => self.stream_openai(input, history, on_token).await,
            other => bail!("unknown provider: {other}"),
        }
    }

    /// Agentic loop with tool calling.
    /// Sends messages, executes tool calls, loops until done or max iterations.
    pub async fn run_agentic(
        &self,
        input: &str,
        history: &[Message],
        tools_config: &ToolsConfig,
        discord_http: &DiscordHttp,
        email_config: &Option<EmailConfig>,
        mut on_token: impl FnMut(String) + Send,
    ) -> Result<String> {
        match self.config.provider.as_str() {
            "anthropic" => {
                self.agentic_anthropic(input, history, tools_config, discord_http, email_config, &mut on_token)
                    .await
            }
            "openai" => {
                self.agentic_openai(input, history, tools_config, discord_http, email_config, &mut on_token)
                    .await
            }
            other => bail!("unknown provider: {other}"),
        }
    }

    // ── Anthropic agentic loop ───────────────────────────────────────

    async fn agentic_anthropic(
        &self,
        input: &str,
        history: &[Message],
        tools_config: &ToolsConfig,
        discord_http: &DiscordHttp,
        email_config: &Option<EmailConfig>,
        on_token: &mut (impl FnMut(String) + Send),
    ) -> Result<String> {
        let base = if self.config.base_url.is_empty() {
            "https://api.anthropic.com"
        } else {
            self.config.base_url.trim_end_matches('/')
        };
        let url = format!("{base}/v1/messages");
        let model = if self.config.model.is_empty() {
            "claude-sonnet-4-20250514"
        } else {
            &self.config.model
        };

        let mut messages: Vec<serde_json::Value> = Vec::with_capacity(history.len() + 1);
        for msg in history {
            messages.push(serde_json::json!({ "role": msg.role, "content": msg.content }));
        }
        messages.push(serde_json::json!({ "role": "user", "content": input }));

        let tools = executor::tool_definitions();
        let mut full_response = String::new();

        for iteration in 0..MAX_TOOL_ITERATIONS {
            debug!(iteration, "Anthropic agentic loop");

            let mut body = serde_json::json!({
                "model": model,
                "max_tokens": 4096,
                "messages": messages,
                "tools": tools,
            });
            if !self.config.system_prompt.is_empty() {
                body["system"] = serde_json::json!(self.config.system_prompt);
            }

            let resp = self
                .client
                .post(&url)
                .header("x-api-key", &self.config.api_key)
                .header("anthropic-version", "2023-06-01")
                .header("content-type", "application/json")
                .body(serde_json::to_string(&body)?)
                .send()
                .await
                .context("Anthropic request failed")?;

            if !resp.status().is_success() {
                let status = resp.status();
                let text = resp.text().await.unwrap_or_default();
                bail!("Anthropic API error {status}: {text}");
            }

            let resp_body: serde_json::Value =
                resp.json().await.context("Failed to parse response")?;

            let stop_reason = resp_body
                .get("stop_reason")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            let content = resp_body
                .get("content")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default();

            let mut text_parts = String::new();
            let mut tool_uses: Vec<(String, String, serde_json::Value)> = Vec::new();

            for block in &content {
                match block.get("type").and_then(|v| v.as_str()) {
                    Some("text") => {
                        if let Some(t) = block.get("text").and_then(|v| v.as_str()) {
                            text_parts.push_str(t);
                            on_token(t.to_string());
                        }
                    }
                    Some("tool_use") => {
                        let id = block.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        let name = block.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        let args = block.get("input").cloned().unwrap_or(serde_json::json!({}));
                        tool_uses.push((id, name, args));
                    }
                    _ => {}
                }
            }

            full_response.push_str(&text_parts);

            if tool_uses.is_empty() || stop_reason == "end_turn" {
                break;
            }

            messages.push(serde_json::json!({ "role": "assistant", "content": content }));

            let mut tool_results: Vec<serde_json::Value> = Vec::new();
            for (id, name, arguments) in tool_uses {
                info!(tool = %name, "Executing tool");
                let call = ToolCall { name, arguments };
                let result = executor::execute_tool(&call, &id, tools_config, discord_http, email_config).await;
                tool_results.push(serde_json::json!({
                    "type": "tool_result",
                    "tool_use_id": result.tool_use_id,
                    "content": result.content,
                }));
            }

            messages.push(serde_json::json!({ "role": "user", "content": tool_results }));
        }

        Ok(full_response)
    }

    // ── OpenAI agentic loop ──────────────────────────────────────────

    async fn agentic_openai(
        &self,
        input: &str,
        history: &[Message],
        tools_config: &ToolsConfig,
        discord_http: &DiscordHttp,
        email_config: &Option<EmailConfig>,
        on_token: &mut (impl FnMut(String) + Send),
    ) -> Result<String> {
        let base = if self.config.base_url.is_empty() {
            "http://127.0.0.1:11434"
        } else {
            self.config.base_url.trim_end_matches('/')
        };
        let url = format!("{base}/v1/chat/completions");
        let model = if self.config.model.is_empty() {
            "llama3"
        } else {
            &self.config.model
        };

        let mut messages: Vec<serde_json::Value> = Vec::new();
        if !self.config.system_prompt.is_empty() {
            messages.push(serde_json::json!({
                "role": "system",
                "content": self.config.system_prompt,
            }));
        }
        for msg in history {
            messages.push(serde_json::json!({ "role": msg.role, "content": msg.content }));
        }
        messages.push(serde_json::json!({ "role": "user", "content": input }));

        // Convert tool definitions to OpenAI format
        let tools = openai_tool_definitions();
        let mut full_response = String::new();

        for iteration in 0..MAX_TOOL_ITERATIONS {
            debug!(iteration, "OpenAI agentic loop");

            let body = serde_json::json!({
                "model": model,
                "messages": messages,
                "tools": tools,
            });

            let mut req = self
                .client
                .post(&url)
                .header("content-type", "application/json");
            if !self.config.api_key.is_empty() {
                req = req.header("authorization", format!("Bearer {}", self.config.api_key));
            }

            let resp = req
                .body(serde_json::to_string(&body)?)
                .send()
                .await
                .context("OpenAI request failed")?;

            if !resp.status().is_success() {
                let status = resp.status();
                let text = resp.text().await.unwrap_or_default();
                bail!("OpenAI API error {status}: {text}");
            }

            let resp_body: serde_json::Value =
                resp.json().await.context("Failed to parse response")?;

            let choice = resp_body
                .get("choices")
                .and_then(|c| c.get(0))
                .cloned()
                .unwrap_or(serde_json::json!({}));

            let finish_reason = choice
                .get("finish_reason")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            let message = choice.get("message").cloned().unwrap_or(serde_json::json!({}));

            // Extract text content
            if let Some(content) = message.get("content").and_then(|v| v.as_str()) {
                if !content.is_empty() {
                    full_response.push_str(content);
                    on_token(content.to_string());
                }
            }

            // Extract tool calls
            let tool_calls = message
                .get("tool_calls")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default();

            if tool_calls.is_empty() || finish_reason == "stop" {
                break;
            }

            // Append assistant message
            messages.push(serde_json::json!({ "role": "assistant", "content": message.get("content"), "tool_calls": tool_calls }));

            // Execute each tool call
            for tc in &tool_calls {
                let id = tc.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let func = tc.get("function").cloned().unwrap_or(serde_json::json!({}));
                let name = func.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let args_str = func.get("arguments").and_then(|v| v.as_str()).unwrap_or("{}");
                let arguments: serde_json::Value =
                    serde_json::from_str(args_str).unwrap_or(serde_json::json!({}));

                info!(tool = %name, "Executing tool");
                let call = ToolCall { name, arguments };
                let result = executor::execute_tool(&call, &id, tools_config, discord_http, email_config).await;

                messages.push(serde_json::json!({
                    "role": "tool",
                    "tool_call_id": id,
                    "content": result.content,
                }));
            }
        }

        Ok(full_response)
    }

    /// Convenience wrapper that returns an mpsc receiver (used by gateway connection).
    pub async fn chat_stream(&self, input: &str, history: &[Message]) -> mpsc::Receiver<String> {
        let (tx, rx) = mpsc::channel(64);
        let input = input.to_string();
        let history = history.to_vec();
        let config = self.config.clone();

        tokio::spawn(async move {
            let runner = AgentRunner {
                config,
                client: Client::new(),
            };
            let tx2 = tx.clone();
            let result = runner
                .run_streaming(&input, &history, move |token| {
                    let _ = tx2.try_send(token);
                })
                .await;

            if let Err(e) = result {
                warn!(%e, "Agent streaming error");
            }
        });

        rx
    }

    // ── Anthropic streaming ──────────────────────────────────────────

    async fn stream_anthropic(
        &self,
        input: &str,
        history: &[Message],
        on_token: impl FnMut(String) + Send,
    ) -> Result<String> {
        let base = if self.config.base_url.is_empty() {
            "https://api.anthropic.com"
        } else {
            self.config.base_url.trim_end_matches('/')
        };
        let url = format!("{base}/v1/messages");

        let model = if self.config.model.is_empty() {
            "claude-sonnet-4-20250514"
        } else {
            &self.config.model
        };

        let mut messages = Vec::with_capacity(history.len() + 1);
        for msg in history {
            messages.push(serde_json::json!({
                "role": msg.role,
                "content": msg.content,
            }));
        }
        messages.push(serde_json::json!({
            "role": "user",
            "content": input,
        }));

        let mut body = serde_json::json!({
            "model": model,
            "max_tokens": 4096,
            "stream": true,
            "messages": messages,
        });

        if !self.config.system_prompt.is_empty() {
            body["system"] = serde_json::json!(self.config.system_prompt);
        }

        let resp = self
            .client
            .post(&url)
            .header("x-api-key", &self.config.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .body(serde_json::to_string(&body)?)
            .send()
            .await
            .context("Anthropic request failed")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            bail!("Anthropic API error {status}: {text}");
        }

        parse_sse_stream(resp, SseFormat::Anthropic, on_token).await
    }

    // ── OpenAI-compatible streaming ──────────────────────────────────

    async fn stream_openai(
        &self,
        input: &str,
        history: &[Message],
        on_token: impl FnMut(String) + Send,
    ) -> Result<String> {
        let base = if self.config.base_url.is_empty() {
            "http://127.0.0.1:11434"
        } else {
            self.config.base_url.trim_end_matches('/')
        };
        let url = format!("{base}/v1/chat/completions");

        let model = if self.config.model.is_empty() {
            "llama3"
        } else {
            &self.config.model
        };

        let mut messages = Vec::with_capacity(history.len() + 2);
        if !self.config.system_prompt.is_empty() {
            messages.push(serde_json::json!({
                "role": "system",
                "content": self.config.system_prompt,
            }));
        }
        for msg in history {
            messages.push(serde_json::json!({
                "role": msg.role,
                "content": msg.content,
            }));
        }
        messages.push(serde_json::json!({
            "role": "user",
            "content": input,
        }));

        let body = serde_json::json!({
            "model": model,
            "stream": true,
            "messages": messages,
        });

        let mut req = self
            .client
            .post(&url)
            .header("content-type", "application/json");

        if !self.config.api_key.is_empty() {
            req = req.header("authorization", format!("Bearer {}", self.config.api_key));
        }

        let resp = req
            .body(serde_json::to_string(&body)?)
            .send()
            .await
            .context("OpenAI request failed")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            bail!("OpenAI API error {status}: {text}");
        }

        parse_sse_stream(resp, SseFormat::OpenAI, on_token).await
    }
}

// ── SSE parsing ──────────────────────────────────────────────────────

enum SseFormat {
    Anthropic,
    OpenAI,
}

async fn parse_sse_stream(
    resp: reqwest::Response,
    format: SseFormat,
    mut on_token: impl FnMut(String) + Send,
) -> Result<String> {
    let mut full = String::new();
    let mut buf = String::new();
    let mut stream = resp.bytes_stream();

    use tokio_stream::StreamExt;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.context("SSE read error")?;
        buf.push_str(&String::from_utf8_lossy(&chunk));

        while let Some(pos) = buf.find('\n') {
            let line = buf[..pos].trim_end_matches('\r').to_string();
            buf = buf[pos + 1..].to_string();

            if !line.starts_with("data: ") {
                continue;
            }
            let data = &line[6..];

            if data == "[DONE]" {
                return Ok(full);
            }

            let parsed: serde_json::Value = match serde_json::from_str(data) {
                Ok(v) => v,
                Err(e) => {
                    debug!(%e, %data, "Skipping unparseable SSE data");
                    continue;
                }
            };

            let token = match format {
                SseFormat::Anthropic => extract_anthropic_delta(&parsed),
                SseFormat::OpenAI => extract_openai_delta(&parsed),
            };

            if let Some(t) = token {
                full.push_str(&t);
                on_token(t);
            }
        }
    }

    Ok(full)
}

fn extract_anthropic_delta(v: &serde_json::Value) -> Option<String> {
    if v.get("type")?.as_str()? == "content_block_delta" {
        v.get("delta")?.get("text")?.as_str().map(String::from)
    } else {
        None
    }
}

fn extract_openai_delta(v: &serde_json::Value) -> Option<String> {
    v.get("choices")?
        .get(0)?
        .get("delta")?
        .get("content")?
        .as_str()
        .map(String::from)
}

/// Convert tool definitions to OpenAI function calling format.
fn openai_tool_definitions() -> serde_json::Value {
    let anthropic_tools = executor::tool_definitions();
    let tools: Vec<serde_json::Value> = anthropic_tools
        .as_array()
        .unwrap()
        .iter()
        .map(|t| {
            serde_json::json!({
                "type": "function",
                "function": {
                    "name": t["name"],
                    "description": t["description"],
                    "parameters": t["input_schema"],
                }
            })
        })
        .collect();
    serde_json::json!(tools)
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AgentConfig;

    #[tokio::test]
    #[ignore]
    async fn test_anthropic_streaming() {
        let api_key = std::env::var("ANTHROPIC_API_KEY").expect("ANTHROPIC_API_KEY not set");
        let config = AgentConfig {
            provider: "anthropic".to_string(),
            api_key,
            base_url: String::new(),
            model: "claude-sonnet-4-20250514".to_string(),
            system_prompt: "Reply in one short sentence.".to_string(),
        };

        let runner = AgentRunner::new(config);
        let mut tokens = Vec::new();
        let result = runner
            .run_streaming("What is 2+2?", &[], |t| {
                print!("{t}");
                tokens.push(t);
            })
            .await
            .expect("Anthropic streaming failed");

        println!("\n--- full response ---\n{result}");
        assert!(!result.is_empty());
        assert!(!tokens.is_empty());
    }

    #[tokio::test]
    #[ignore]
    async fn test_openai_streaming() {
        let config = AgentConfig {
            provider: "openai".to_string(),
            api_key: std::env::var("OPENAI_API_KEY").unwrap_or_default(),
            base_url: std::env::var("OPENAI_BASE_URL")
                .unwrap_or_else(|_| "http://127.0.0.1:11434".to_string()),
            model: std::env::var("OPENAI_MODEL").unwrap_or_else(|_| "llama3".to_string()),
            system_prompt: "Reply in one short sentence.".to_string(),
        };

        let runner = AgentRunner::new(config);
        let mut tokens = Vec::new();
        let result = runner
            .run_streaming("What is 2+2?", &[], |t| {
                print!("{t}");
                tokens.push(t);
            })
            .await
            .expect("OpenAI streaming failed");

        println!("\n--- full response ---\n{result}");
        assert!(!result.is_empty());
        assert!(!tokens.is_empty());
    }
}
