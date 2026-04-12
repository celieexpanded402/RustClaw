use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use tokio::sync::mpsc;
use tracing::{error, info, warn};

use crate::agent::runner::ToolContext;
use crate::agent::{AgentRunner, Message as AgentMessage};
use crate::config::{DiscordConfig, EmailConfig, ToolsConfig};
use crate::cron::CronContext;
use crate::session::memory::MemoryManager;
use crate::tools::executor::DiscordHttp;
use crate::tools::mcp::McpManager;

const MAX_RETRIES: u32 = 3;
const EDIT_INTERVAL: Duration = Duration::from_millis(1000);
const MAX_MSG_LEN: usize = 2000;

pub struct DiscordChannel {
    config: DiscordConfig,
    memory: MemoryManager,
}

impl DiscordChannel {
    pub fn new(config: DiscordConfig, memory: MemoryManager) -> Self {
        Self { config, memory }
    }

    pub async fn start(
        self,
        runner: Arc<AgentRunner>,
        cron_ctx: Option<Arc<CronContext>>,
        tools_config: ToolsConfig,
        email_config: Option<EmailConfig>,
        mcp: Option<Arc<McpManager>>,
    ) -> Result<()> {
        if self.config.bot_token.is_empty() {
            anyhow::bail!("Discord bot_token is empty");
        }

        // Create an Http client from the token so tools can use it
        let discord_http: DiscordHttp =
            Some(Arc::new(serenity::http::Http::new(&self.config.bot_token)));

        let intents = GatewayIntents::GUILD_MESSAGES
            | GatewayIntents::DIRECT_MESSAGES
            | GatewayIntents::MESSAGE_CONTENT;

        let handler = Handler {
            runner,
            memory: self.memory,
            config: Arc::new(self.config),
            cron_ctx,
            tools_config,
            discord_http,
            email_config,
            mcp,
        };

        let mut client = Client::builder(&handler.config.bot_token, intents)
            .event_handler(handler)
            .await?;

        info!("Starting Discord bot");
        client.start().await?;

        Ok(())
    }
}

struct Handler {
    runner: Arc<AgentRunner>,
    memory: MemoryManager,
    config: Arc<DiscordConfig>,
    cron_ctx: Option<Arc<CronContext>>,
    tools_config: ToolsConfig,
    discord_http: DiscordHttp,
    email_config: Option<EmailConfig>,
    mcp: Option<Arc<McpManager>>,
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _ctx: Context, ready: Ready) {
        info!(bot_name = %ready.user.name, "Discord bot connected");
    }

    async fn message(&self, ctx: Context, msg: Message) {
        if msg.author.bot {
            return;
        }

        // Guild ACL
        if !self.config.allowed_guild_ids.is_empty() {
            if let Some(guild_id) = msg.guild_id {
                if !self.config.allowed_guild_ids.contains(&guild_id.get()) {
                    return;
                }
            }
        }

        let is_dm = msg.guild_id.is_none();
        let is_mentioned = msg.mentions_me(&ctx.http).await.unwrap_or(false);

        if self.config.mention_only && !is_dm && !is_mentioned {
            return;
        }

        let text = strip_mention(&msg.content, &ctx).await;
        if text.trim().is_empty() {
            return;
        }

        // ── Check for bot commands ───────────────────────────────────
        let trimmed = text.trim();
        if let Some(response) = self.handle_command(&ctx, &msg, trimmed).await {
            let display = truncate_for_discord(&response);
            if let Err(e) = retry_send(&ctx, msg.channel_id, &display).await {
                error!(%e, "Failed to send command response");
            }
            return;
        }

        // ── Normal chat flow ─────────────────────────────────────────
        let channel_id = msg.channel_id;
        let user_id = msg.author.id;
        let session_id = format!("discord:{}:{}", channel_id, user_id);

        self.memory.get_or_create(&session_id).await;
        let history = self.memory.get_history(&session_id).await;
        let recalled = self.memory.recall(&session_id, trimmed).await;

        let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S %Z");
        let mut context_parts = Vec::new();
        if history.is_empty() {
            context_parts.push(format!("[Current time: {now}]"));
        }
        if !recalled.is_empty() {
            context_parts.push(format!("[Memory]\n{recalled}"));
        }
        let input = if context_parts.is_empty() {
            trimmed.to_string()
        } else {
            context_parts.push(trimmed.to_string());
            context_parts.join("\n\n")
        };

        self.memory
            .push_message(
                &session_id,
                AgentMessage {
                    role: "user".to_string(),
                    content: trimmed.to_string(),
                },
            )
            .await;

        // Send "thinking..." placeholder
        let placeholder = match retry_send(&ctx, channel_id, "thinking...").await {
            Ok(m) => m,
            Err(e) => {
                error!(%e, "Failed to send placeholder");
                return;
            }
        };

        // Stream response (with tool calling if enabled)
        let (tx, mut rx) = mpsc::channel::<String>(64);
        let runner_config = self.runner.config().clone();
        let input_owned = input.clone();
        let history_owned = history.clone();
        let tc = ToolContext {
            config: self.tools_config.clone(),
            discord_http: self.discord_http.clone(),
            email_config: self.email_config.clone(),
            mcp: self.mcp.clone(),
        };

        tokio::spawn(async move {
            let r = AgentRunner::new(runner_config);
            let tx2 = tx.clone();
            if tc.config.enabled {
                let result = r
                    .run_agentic(&input_owned, &history_owned, &tc, move |token| {
                        let _ = tx2.try_send(token);
                    })
                    .await;
                if let Err(e) = result {
                    error!(%e, "Agent agentic error in Discord handler");
                }
            } else {
                let result = r
                    .run_streaming(&input_owned, &history_owned, move |token| {
                        let _ = tx2.try_send(token);
                    })
                    .await;
                if let Err(e) = result {
                    error!(%e, "Agent streaming error in Discord handler");
                }
            }
        });

        let mut full = String::new();
        let mut last_edit = tokio::time::Instant::now();

        loop {
            tokio::select! {
                token = rx.recv() => {
                    match token {
                        Some(t) => {
                            full.push_str(&t);
                            if last_edit.elapsed() >= EDIT_INTERVAL {
                                let display = truncate_for_discord(&format!("{full}..."));
                                retry_edit(&ctx, &placeholder, &display).await;
                                last_edit = tokio::time::Instant::now();
                            }
                        }
                        None => break,
                    }
                }
            }
        }

        if full.is_empty() {
            full.push_str("(no response)");
        }
        let final_text = truncate_for_discord(&full);
        retry_edit(&ctx, &placeholder, &final_text).await;

        self.memory
            .push_message(
                &session_id,
                AgentMessage {
                    role: "assistant".to_string(),
                    content: full.clone(),
                },
            )
            .await;
        self.memory.learn(&session_id, trimmed, &full).await;
    }
}

impl Handler {
    /// Handle bot commands. Returns Some(response) if a command was matched.
    async fn handle_command(
        &self,
        ctx: &Context,
        msg: &Message,
        text: &str,
    ) -> Option<String> {
        let lower = text.to_lowercase();

        if lower == "scan" {
            return Some(self.cmd_scan().await);
        }

        if let Some(rest) = lower.strip_prefix("fix issue #") {
            if let Ok(num) = rest.trim().parse::<u64>() {
                return Some(self.cmd_fix_issue(ctx, msg, num).await);
            }
        }
        // Also support "fix issue 123" without #
        if let Some(rest) = lower.strip_prefix("fix issue ") {
            let cleaned = rest.trim().trim_start_matches('#');
            if let Ok(num) = cleaned.parse::<u64>() {
                return Some(self.cmd_fix_issue(ctx, msg, num).await);
            }
        }

        if lower == "pr status" {
            return Some(self.cmd_pr_status().await);
        }

        None
    }

    /// `scan` command: immediately run github_scan.
    async fn cmd_scan(&self) -> String {
        let ctx = match &self.cron_ctx {
            Some(c) => c,
            None => return "GitHub integration not configured.".to_string(),
        };

        let github = match &ctx.github {
            Some(gh) => gh,
            None => return "GitHub token not configured.".to_string(),
        };

        let mut results = Vec::new();
        for repo in &ctx.github_config.repos {
            match github.scan_repo(repo).await {
                Ok(r) => results.push(r),
                Err(e) => return format!("Error scanning {repo}: {e}"),
            }
        }

        crate::tools::github::GitHubClient::format_scan_report(&results)
    }

    /// `fix issue #N` command: auto-PR for an issue.
    async fn cmd_fix_issue(&self, ctx: &Context, msg: &Message, issue_number: u64) -> String {
        let cron_ctx = match &self.cron_ctx {
            Some(c) => c,
            None => return "GitHub integration not configured.".to_string(),
        };

        if cron_ctx.github_config.repos.is_empty() {
            return "No repos configured.".to_string();
        }

        // Use first configured repo by default
        let repo = &cron_ctx.github_config.repos[0];

        // Send a progress message
        let _ = msg.channel_id.say(&ctx.http, format!("Working on issue #{issue_number} in {repo}...")).await;

        match crate::cron::run_auto_pr(cron_ctx, &self.runner, repo, issue_number).await {
            Ok(url) => format!("PR created: {url}"),
            Err(e) => format!("Failed to create auto-PR: {e}"),
        }
    }

    /// `pr status` command: list open PRs created by the bot.
    async fn cmd_pr_status(&self) -> String {
        let ctx = match &self.cron_ctx {
            Some(c) => c,
            None => return "GitHub integration not configured.".to_string(),
        };

        let github = match &ctx.github {
            Some(gh) => gh,
            None => return "GitHub token not configured.".to_string(),
        };

        let mut out = String::from("**Open PRs**\n");

        for repo in &ctx.github_config.repos {
            match github.list_open_prs(repo).await {
                Ok(prs) => {
                    let bot_prs: Vec<_> = prs
                        .iter()
                        .filter(|pr| {
                            pr.head
                                .ref_field
                                .starts_with("rustclaw/")
                        })
                        .collect();

                    if bot_prs.is_empty() {
                        out.push_str(&format!("📦 {repo}: no rustclaw PRs\n"));
                    } else {
                        out.push_str(&format!("📦 {repo}:\n"));
                        for pr in bot_prs {
                            let title = pr.title.as_deref().unwrap_or("(untitled)");
                            let url = pr
                                .html_url
                                .as_ref()
                                .map(|u| u.as_str())
                                .unwrap_or("");
                            out.push_str(&format!(
                                "  #{} {} — {}\n",
                                pr.number, title, url
                            ));
                        }
                    }
                }
                Err(e) => {
                    out.push_str(&format!("📦 {repo}: error — {e}\n"));
                }
            }
        }

        out
    }
}

// ── Helpers ──────────────────────────────────────────────────────────

async fn strip_mention(content: &str, ctx: &Context) -> String {
    let current_user_id = ctx.cache.current_user().id;
    let mention_str = format!("<@{}>", current_user_id);
    let mention_nick = format!("<@!{}>", current_user_id);
    content
        .replace(&mention_str, "")
        .replace(&mention_nick, "")
        .trim()
        .to_string()
}

fn truncate_for_discord(text: &str) -> String {
    if text.len() <= MAX_MSG_LEN {
        return text.to_string();
    }
    let truncated = &text[..MAX_MSG_LEN - 15];
    format!("{truncated}\n...(truncated)")
}

async fn retry_send(
    ctx: &Context,
    channel_id: serenity::model::id::ChannelId,
    text: &str,
) -> Result<Message> {
    let mut last_err = None;
    for attempt in 0..MAX_RETRIES {
        match channel_id.say(&ctx.http, text).await {
            Ok(msg) => return Ok(msg),
            Err(e) => {
                warn!(attempt, %e, "Discord send failed, retrying");
                last_err = Some(e);
                tokio::time::sleep(Duration::from_millis(500 * u64::from(attempt + 1))).await;
            }
        }
    }
    anyhow::bail!(
        "Discord send failed after {MAX_RETRIES} retries: {}",
        last_err.unwrap()
    )
}

async fn retry_edit(ctx: &Context, msg: &Message, text: &str) {
    for attempt in 0..MAX_RETRIES {
        let result = msg
            .channel_id
            .edit_message(
                &ctx.http,
                msg.id,
                serenity::builder::EditMessage::new().content(text),
            )
            .await;
        match result {
            Ok(_) => return,
            Err(e) => {
                warn!(attempt, %e, "Discord edit failed, retrying");
                tokio::time::sleep(Duration::from_millis(500 * u64::from(attempt + 1))).await;
            }
        }
    }
}
