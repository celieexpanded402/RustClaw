use anyhow::{Context, Result};
use serde::Serialize;

use super::exec;

// ── Process monitoring ───────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct ProcessInfo {
    pub pid: String,
    pub name: String,
    pub cpu: String,
    pub mem: String,
    pub status: String,
}

/// List all running processes (summary).
pub async fn process_list() -> Result<String> {
    let result = exec::run_command("ps aux --sort=-%mem | head -20", "/", 10, "/").await?;
    Ok(result.stdout)
}

/// Check if a specific process is running.
pub async fn process_check(name: &str) -> Result<String> {
    let result = exec::run_command(
        &format!("pgrep -fl '{name}' || echo 'NOT RUNNING'"),
        "/",
        5,
        "/",
    )
    .await?;
    let running = !result.stdout.contains("NOT RUNNING");
    Ok(format!(
        "Process '{}': {}\n{}",
        name,
        if running { "RUNNING" } else { "NOT RUNNING" },
        result.stdout.trim()
    ))
}

// ── Docker monitoring ────────────────────────────────────────────────

/// List all Docker containers with status.
pub async fn docker_status() -> Result<String> {
    let result = exec::run_command(
        "docker ps -a --format 'table {{.Names}}\\t{{.Status}}\\t{{.Ports}}' 2>&1",
        "/",
        10,
        "/",
    )
    .await?;

    if result.exit_code != 0 || result.stdout.contains("Cannot connect") {
        return Ok("Docker: not running or not installed".to_string());
    }

    Ok(format!("Docker containers:\n{}", result.stdout))
}

/// Check a specific Docker container.
pub async fn docker_inspect(container: &str) -> Result<String> {
    let result = exec::run_command(
        &format!(
            "docker inspect --format '{{{{.State.Status}}}} | {{{{.State.Health.Status}}}}' '{container}' 2>&1"
        ),
        "/",
        5,
        "/",
    )
    .await?;
    Ok(format!("Container '{container}': {}", result.stdout.trim()))
}

// ── PM2 monitoring ───────────────────────────────────────────────────

/// List all PM2 processes with status.
pub async fn pm2_status() -> Result<String> {
    let result = exec::run_command("pm2 jlist 2>/dev/null", "/", 10, "/").await?;

    if result.exit_code != 0 || result.stdout.trim().is_empty() {
        return Ok("PM2: not running or not installed".to_string());
    }

    // Parse JSON output from pm2
    let processes: Vec<serde_json::Value> =
        serde_json::from_str(&result.stdout).unwrap_or_default();

    if processes.is_empty() {
        return Ok("PM2: no processes".to_string());
    }

    let mut out = String::from("PM2 processes:\n");
    for p in &processes {
        let name = p.get("name").and_then(|v| v.as_str()).unwrap_or("?");
        let status = p
            .get("pm2_env")
            .and_then(|e| e.get("status"))
            .and_then(|v| v.as_str())
            .unwrap_or("?");
        let memory = p
            .get("monit")
            .and_then(|m| m.get("memory"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let cpu = p
            .get("monit")
            .and_then(|m| m.get("cpu"))
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        let restarts = p
            .get("pm2_env")
            .and_then(|e| e.get("restart_time"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        out.push_str(&format!(
            "  {name}: {status} | CPU: {cpu:.1}% | MEM: {:.1}MB | restarts: {restarts}\n",
            memory as f64 / 1024.0 / 1024.0
        ));
    }

    Ok(out)
}

// ── HTTP health probe ────────────────────────────────────────────────

/// Ping an HTTP endpoint and return status.
pub async fn http_ping(url: &str) -> Result<String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .context("Failed to build HTTP client")?;

    match client.get(url).send().await {
        Ok(resp) => {
            let status = resp.status();
            let latency = "ok"; // reqwest doesn't expose timing easily
            Ok(format!("{url}: HTTP {status} ({latency})"))
        }
        Err(e) => Ok(format!("{url}: UNREACHABLE ({e})")),
    }
}

// ── System stats ─────────────────────────────────────────────────────

/// Get system resource summary.
pub async fn system_stats() -> Result<String> {
    // macOS and Linux compatible
    let uptime = exec::run_command("uptime", "/", 5, "/").await?;
    let disk = exec::run_command("df -h / | tail -1", "/", 5, "/").await?;
    let mem = exec::run_command(
        "vm_stat 2>/dev/null | head -5 || free -h 2>/dev/null | head -3",
        "/",
        5,
        "/",
    )
    .await?;

    Ok(format!(
        "Uptime: {}\nDisk: {}\nMemory:\n{}",
        uptime.stdout.trim(),
        disk.stdout.trim(),
        mem.stdout.trim()
    ))
}
