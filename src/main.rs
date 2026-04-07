mod agent;
mod channels;
mod cli;
mod config;
mod cron;
mod gateway;
mod session;
mod tools;

use std::io::Write;
use std::sync::Arc;

use clap::Parser;
use tracing::info;

use crate::agent::AgentRunner;
use crate::channels::discord::DiscordChannel;
use crate::channels::telegram::TelegramChannel;
use crate::cli::{Cli, Command, GithubCommand};
use crate::config::AppConfig;
use crate::cron::CronContext;
use crate::session::store::SessionStore;
use crate::tools::github::GitHubClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let cfg = AppConfig::load(cli.config.as_deref())?;

    match cli.command {
        Command::Gateway => cmd_gateway(cfg).await,
        Command::Agent { message, stream } => cmd_agent(cfg, &message, stream).await,
        Command::Status => cmd_status(&cfg).await,
        Command::Health => cmd_health(&cfg).await,
        Command::Github(sub) => cmd_github(cfg, sub).await,
    }
}

// ── Gateway ──────────────────────────────────────────────────────────

async fn cmd_gateway(cfg: AppConfig) -> anyhow::Result<()> {
    init_tracing();

    let listen = cfg.gateway.listen_addr();
    info!("Starting rustclaw gateway on {listen}");

    let sessions = SessionStore::new();
    let runner = Arc::new(AgentRunner::new(cfg.agent.clone()));

    // GitHub client
    let github = if !cfg.github.token.is_empty() {
        match GitHubClient::new(&cfg.github.token) {
            Ok(gh) => {
                info!("GitHub client initialized");
                Some(gh)
            }
            Err(e) => {
                tracing::warn!(%e, "Failed to init GitHub client");
                None
            }
        }
    } else {
        None
    };

    // Shared cron context
    let cron_ctx = Arc::new(CronContext {
        github,
        github_config: cfg.github.clone(),
        cron_config: cfg.cron.clone(),
        discord_http: None,
    });

    // Telegram
    if cfg.channels.telegram.enabled {
        info!("Starting Telegram channel");
        let tg = TelegramChannel::new(cfg.channels.telegram.clone(), sessions.clone());
        let tg_runner = Arc::clone(&runner);
        tokio::spawn(async move {
            if let Err(e) = tg.start(tg_runner).await {
                tracing::error!(%e, "Telegram channel exited with error");
            }
        });
    }

    // Discord
    if cfg.channels.discord.enabled {
        info!("Starting Discord channel");
        let dc = DiscordChannel::new(cfg.channels.discord.clone(), sessions.clone());
        let dc_runner = Arc::clone(&runner);
        let dc_cron = if cron_ctx.github.is_some() {
            Some(Arc::clone(&cron_ctx))
        } else {
            None
        };
        let dc_tools = cfg.tools.clone();
        tokio::spawn(async move {
            if let Err(e) = dc.start(dc_runner, dc_cron, dc_tools).await {
                tracing::error!(%e, "Discord channel exited with error");
            }
        });
    }

    // Cron scheduler
    if cron_ctx.github.is_some() && !cfg.github.repos.is_empty() {
        info!("Starting cron scheduler");
        let cron_ctx_clone = Arc::clone(&cron_ctx);
        tokio::spawn(async move {
            if let Err(e) = crate::cron::start(cron_ctx_clone).await {
                tracing::error!(%e, "Cron scheduler exited with error");
            }
        });
    }

    gateway::server::run_with_sessions(cfg, sessions).await
}

// ── Agent ────────────────────────────────────────────────────────────

async fn cmd_agent(cfg: AppConfig, message: &str, stream: bool) -> anyhow::Result<()> {
    let runner = AgentRunner::new(cfg.agent);
    let tools_config = cfg.tools;

    if tools_config.enabled {
        // Agentic mode with tool calling
        let mut stdout = std::io::stdout();
        let no_discord = None;
        let result = runner
            .run_agentic(message, &[], &tools_config, &no_discord, |token| {
                let _ = write!(stdout, "{token}");
                let _ = stdout.flush();
            })
            .await?;
        if !result.ends_with('\n') {
            println!();
        }
    } else if stream {
        let mut stdout = std::io::stdout();
        let result = runner
            .run_streaming(message, &[], |token| {
                let _ = write!(stdout, "{token}");
                let _ = stdout.flush();
            })
            .await?;
        if !result.ends_with('\n') {
            println!();
        }
    } else {
        let result = runner.run_streaming(message, &[], |_| {}).await?;
        println!("{result}");
    }

    Ok(())
}

// ── GitHub ───────────────────────────────────────────────────────────

async fn cmd_github(cfg: AppConfig, sub: GithubCommand) -> anyhow::Result<()> {
    init_tracing();

    if cfg.github.token.is_empty() {
        anyhow::bail!("GitHub token not configured. Set [github] token in config.toml or RUSTCLAW__GITHUB__TOKEN env.");
    }

    let github = GitHubClient::new(&cfg.github.token)?;
    let cron_ctx = CronContext {
        github: Some(github),
        github_config: cfg.github.clone(),
        cron_config: cfg.cron.clone(),
        discord_http: None,
    };

    match sub {
        GithubCommand::Scan => {
            if cfg.github.repos.is_empty() {
                anyhow::bail!("No repos configured in [github] repos.");
            }
            crate::cron::run_github_scan(&cron_ctx).await;
            Ok(())
        }
        GithubCommand::Fix { issue, repo } => {
            let repo = repo
                .or_else(|| cfg.github.repos.first().cloned())
                .ok_or_else(|| anyhow::anyhow!("No repo specified and none in config."))?;

            let runner = AgentRunner::new(cfg.agent);
            let pr_url = crate::cron::run_auto_pr(&cron_ctx, &runner, &repo, issue).await?;
            println!("PR created: {pr_url}");
            Ok(())
        }
    }
}

// ── Status / Health ──────────────────────────────────────────────────

async fn cmd_status(cfg: &AppConfig) -> anyhow::Result<()> {
    let addr = cfg.gateway.listen_addr();
    let url = format!("ws://{addr}/ws");

    let connect_result = tokio_tungstenite::connect_async(&url).await;
    match connect_result {
        Ok((mut ws, _)) => {
            use futures_util::SinkExt;
            use tokio_tungstenite::tungstenite::Message;

            let connect = serde_json::json!({
                "type": "connect",
                "params": { "role": "cli" }
            });
            ws.send(Message::Text(connect.to_string())).await?;

            use futures_util::StreamExt;
            if let Some(Ok(msg)) = ws.next().await {
                println!("Gateway response: {msg}");
            }

            let auth = serde_json::json!({
                "type": "auth",
                "nonce": "",
                "token": ""
            });
            ws.send(Message::Text(auth.to_string())).await?;

            if let Some(Ok(msg)) = ws.next().await {
                println!("Auth response: {msg}");
            }

            let health = serde_json::json!({
                "type": "req",
                "id": "status-1",
                "method": "health",
                "params": {}
            });
            ws.send(Message::Text(health.to_string())).await?;

            if let Some(Ok(msg)) = ws.next().await {
                println!("Status: {msg}");
            }

            ws.close(None).await?;
            Ok(())
        }
        Err(e) => {
            eprintln!("Cannot connect to gateway at {url}: {e}");
            std::process::exit(1);
        }
    }
}

async fn cmd_health(cfg: &AppConfig) -> anyhow::Result<()> {
    let addr = cfg.gateway.listen_addr();
    let url = format!("http://{addr}/health");

    match reqwest::get(&url).await {
        Ok(resp) => {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            if status.is_success() {
                println!("ok ({body})");
            } else {
                eprintln!("unhealthy: HTTP {status} — {body}");
                std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("Cannot reach gateway at {url}: {e}");
            std::process::exit(1);
        }
    }

    Ok(())
}

fn init_tracing() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("rustclaw=info".parse().unwrap()),
        )
        .init();
}
