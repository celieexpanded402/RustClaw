use std::path::Path;
use std::time::Duration;

use anyhow::{Context, Result};
use serde::Serialize;
use tokio::process::Command;

#[derive(Debug, Serialize)]
pub struct ExecResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

/// Execute a shell command in `cwd`, with a timeout.
/// `workspace_root` is the security boundary — `cwd` must be inside it.
pub async fn run_command(
    cmd: &str,
    cwd: &str,
    timeout_secs: u64,
    workspace_root: &str,
) -> Result<ExecResult> {
    // Security: ensure cwd is within workspace
    let cwd_abs = std::fs::canonicalize(cwd).unwrap_or_else(|_| Path::new(cwd).to_path_buf());
    let root_abs =
        std::fs::canonicalize(workspace_root).unwrap_or_else(|_| Path::new(workspace_root).to_path_buf());

    if !cwd_abs.starts_with(&root_abs) {
        anyhow::bail!(
            "run_command: cwd '{}' is outside workspace '{}'",
            cwd_abs.display(),
            root_abs.display()
        );
    }

    let timeout = Duration::from_secs(timeout_secs);

    let result = tokio::time::timeout(timeout, async {
        Command::new("sh")
            .arg("-lc")
            .arg(cmd)
            .current_dir(&cwd_abs)
            .output()
            .await
            .context("Failed to spawn command")
    })
    .await;

    match result {
        Ok(Ok(output)) => Ok(ExecResult {
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            exit_code: output.status.code().unwrap_or(-1),
        }),
        Ok(Err(e)) => Err(e),
        Err(_) => Ok(ExecResult {
            stdout: String::new(),
            stderr: format!("Command timed out after {timeout_secs}s"),
            exit_code: -1,
        }),
    }
}
