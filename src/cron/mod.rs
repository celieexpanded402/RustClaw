use std::sync::Arc;

use anyhow::Result;
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::{error, info, warn};

use crate::config::{CronConfig, EmailConfig, GitHubConfig, MonitorConfig};
use crate::tools::github::GitHubClient;

/// Shared context for cron jobs.
pub struct CronContext {
    pub github: Option<GitHubClient>,
    pub github_config: GitHubConfig,
    pub cron_config: CronConfig,
    pub monitor_config: MonitorConfig,
    pub email_config: Option<EmailConfig>,
    /// Discord HTTP client for sending notifications.
    pub discord_http: Option<Arc<serenity::http::Http>>,
    /// Discord channel ID for notifications.
    pub notify_channel_id: Option<u64>,
}

/// Start the cron scheduler.
pub async fn start(ctx: Arc<CronContext>) -> Result<()> {
    let sched = JobScheduler::new().await?;

    // ── GitHub scan job ──────────────────────────────────────────────
    if ctx.github.is_some() && !ctx.github_config.repos.is_empty() {
        let cron_expr = ctx.cron_config.github_scan.clone();
        let ctx_clone = Arc::clone(&ctx);

        let job = Job::new_async(cron_expr.as_str(), move |_uuid, _lock| {
            let ctx = Arc::clone(&ctx_clone);
            Box::pin(async move {
                run_github_scan(&ctx).await;
            })
        })?;

        sched.add(job).await?;
        info!(schedule = %cron_expr, "Scheduled github_scan");
    }

    // ── System check job ─────────────────────────────────────────────
    if !ctx.monitor_config.services.is_empty()
        || !ctx.monitor_config.endpoints.is_empty()
        || ctx.monitor_config.docker
        || ctx.monitor_config.pm2
    {
        let cron_expr = ctx.cron_config.system_check.clone();
        let ctx_clone = Arc::clone(&ctx);

        let job = Job::new_async(cron_expr.as_str(), move |_uuid, _lock| {
            let ctx = Arc::clone(&ctx_clone);
            Box::pin(async move {
                run_system_check(&ctx).await;
            })
        })?;

        sched.add(job).await?;
        info!(schedule = %cron_expr, "Scheduled system_check");
    }

    // ── Email scan job ───────────────────────────────────────────────
    if ctx.email_config.is_some() {
        let cron_expr = ctx.cron_config.email_scan.clone();
        let ctx_clone = Arc::clone(&ctx);

        let job = Job::new_async(cron_expr.as_str(), move |_uuid, _lock| {
            let ctx = Arc::clone(&ctx_clone);
            Box::pin(async move {
                run_email_scan(&ctx).await;
            })
        })?;

        sched.add(job).await?;
        info!(schedule = %cron_expr, "Scheduled email_scan");
    }

    sched.start().await?;

    loop {
        tokio::time::sleep(std::time::Duration::from_secs(3600)).await;
    }
}

// ── Discord notification helper ──────────────────────────────────────

async fn notify_discord(ctx: &CronContext, message: &str) {
    let (http, channel_id) = match (&ctx.discord_http, ctx.notify_channel_id) {
        (Some(h), Some(id)) => (h, id),
        _ => return,
    };

    let truncated = if message.len() > 2000 {
        format!("{}...(truncated)", &message[..1985])
    } else {
        message.to_string()
    };

    if let Err(e) = serenity::model::id::ChannelId::new(channel_id)
        .say(http.as_ref(), &truncated)
        .await
    {
        error!(%e, "Failed to send Discord notification");
    }
}

// ── GitHub scan ──────────────────────────────────────────────────────

pub async fn run_github_scan(ctx: &CronContext) {
    let github = match &ctx.github {
        Some(gh) => gh,
        None => return,
    };

    info!("Running GitHub scan");

    let mut results = Vec::new();
    for repo in &ctx.github_config.repos {
        match github.scan_repo(repo).await {
            Ok(result) => results.push(result),
            Err(e) => error!(%repo, %e, "Failed to scan repo"),
        }
    }

    let report = GitHubClient::format_scan_report(&results);
    info!("\n{report}");
    notify_discord(ctx, &report).await;
}

// ── System check ─────────────────────────────────────────────────────

pub async fn run_system_check(ctx: &CronContext) {
    info!("Running system check");

    let mut alerts = Vec::new();

    // Check named services
    for service in &ctx.monitor_config.services {
        match crate::tools::system::process_check(service).await {
            Ok(status) => {
                if status.contains("NOT RUNNING") {
                    alerts.push(format!("🔴 Process `{service}` is NOT RUNNING"));
                }
            }
            Err(e) => alerts.push(format!("⚠️ Failed to check `{service}`: {e}")),
        }
    }

    // Check Docker containers
    if ctx.monitor_config.docker {
        match crate::tools::system::docker_status().await {
            Ok(status) => {
                // Look for unhealthy or exited containers
                for line in status.lines() {
                    let lower = line.to_lowercase();
                    if lower.contains("exited") || lower.contains("unhealthy") {
                        alerts.push(format!("🐳 {}", line.trim()));
                    }
                }
            }
            Err(e) => alerts.push(format!("⚠️ Docker check failed: {e}")),
        }
    }

    // Check PM2 processes
    if ctx.monitor_config.pm2 {
        match crate::tools::system::pm2_status().await {
            Ok(status) => {
                for line in status.lines() {
                    let lower = line.to_lowercase();
                    if lower.contains("errored") || lower.contains("stopped") {
                        alerts.push(format!("📦 PM2: {}", line.trim()));
                    }
                }
            }
            Err(e) => alerts.push(format!("⚠️ PM2 check failed: {e}")),
        }
    }

    // Check HTTP endpoints
    for url in &ctx.monitor_config.endpoints {
        match crate::tools::system::http_ping(url).await {
            Ok(result) => {
                if result.contains("UNREACHABLE") || result.contains("50") {
                    alerts.push(format!("🌐 {result}"));
                }
            }
            Err(e) => alerts.push(format!("⚠️ HTTP ping {url} failed: {e}")),
        }
    }

    if alerts.is_empty() {
        info!("System check: all OK");
    } else {
        let report = format!(
            "**⚠️ System Alert ({} issues)**\n{}",
            alerts.len(),
            alerts.join("\n")
        );
        warn!("\n{report}");
        notify_discord(ctx, &report).await;
    }
}

// ── Email scan ───────────────────────────────────────────────────────

pub async fn run_email_scan(ctx: &CronContext) {
    let email_cfg = match &ctx.email_config {
        Some(cfg) => cfg,
        None => return,
    };

    info!("Running email scan");

    match crate::tools::email::fetch_inbox(email_cfg, 5).await {
        Ok(emails) => {
            if emails.is_empty() {
                info!("No new emails");
                return;
            }

            let report = crate::tools::email::format_inbox_report(&emails);
            info!("\n{report}");
            notify_discord(ctx, &report).await;
        }
        Err(e) => {
            error!(%e, "Email scan failed");
        }
    }
}

// ── Auto-PR (triggered by command) ───────────────────────────────────

pub async fn run_auto_pr(
    ctx: &CronContext,
    runner: &crate::agent::AgentRunner,
    repo: &str,
    issue_number: u64,
) -> Result<String> {
    let github = ctx
        .github
        .as_ref()
        .context("GitHub client not configured")?;

    let issue = github.get_issue(repo, issue_number).await?;
    let issue_title = &issue.title;
    let issue_body = issue.body.as_deref().unwrap_or("(no description)");

    info!(%repo, issue_number, %issue_title, "Starting auto-PR");

    let prompt = format!(
        "You are a senior developer. Analyze this GitHub issue and generate a fix.\n\n\
         Repository: {repo}\n\
         Issue #{issue_number}: {issue_title}\n\
         Description:\n{issue_body}\n\n\
         Respond with EXACTLY this format:\n\
         FILE: <path>\n\
         ```\n<file content>\n```\n\
         COMMIT_MESSAGE: <message>\n\n\
         Only modify one file. Keep the change minimal."
    );

    let response = runner.run_streaming(&prompt, &[], |_| {}).await?;
    let (file_path, file_content, commit_msg) = parse_fix_response(&response)?;

    let branch_name = format!("rustclaw/fix-issue-{issue_number}");
    github.create_branch(repo, &branch_name, "main").await?;

    github
        .commit_file(repo, &file_path, &file_content, &commit_msg, &branch_name)
        .await?;

    let pr_body = format!(
        "Fixes #{issue_number}\n\n\
         Auto-generated by rustclaw.\n\n\
         ## Changes\n{commit_msg}"
    );
    let pr_url = github
        .create_pr(
            repo,
            &format!("fix: {issue_title}"),
            &pr_body,
            &branch_name,
            "main",
        )
        .await?;

    Ok(pr_url)
}

fn parse_fix_response(response: &str) -> Result<(String, String, String)> {
    let file_path = response
        .lines()
        .find(|l| l.starts_with("FILE:"))
        .map(|l| l.trim_start_matches("FILE:").trim().to_string())
        .context("LLM response missing FILE: line")?;

    let content = if let Some(start) = response.find("```") {
        let after_start = &response[start + 3..];
        let code_start = after_start.find('\n').unwrap_or(0) + 1;
        let after_lang = &after_start[code_start..];
        if let Some(end) = after_lang.find("```") {
            after_lang[..end].to_string()
        } else {
            after_lang.to_string()
        }
    } else {
        anyhow::bail!("LLM response missing code block");
    };

    let commit_msg = response
        .lines()
        .find(|l| l.starts_with("COMMIT_MESSAGE:"))
        .map(|l| l.trim_start_matches("COMMIT_MESSAGE:").trim().to_string())
        .unwrap_or_else(|| "fix: address issue (auto-generated)".to_string());

    Ok((file_path, content, commit_msg))
}

use anyhow::Context;
