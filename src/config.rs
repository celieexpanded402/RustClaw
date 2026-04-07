use std::path::PathBuf;

use serde::Deserialize;

#[derive(Debug, Deserialize, Clone, Default)]
pub struct AppConfig {
    #[serde(default)]
    pub gateway: GatewayConfig,
    #[serde(default)]
    pub agent: AgentConfig,
    #[serde(default)]
    pub channels: ChannelsConfig,
    #[serde(default)]
    pub github: GitHubConfig,
    #[serde(default)]
    pub cron: CronConfig,
    #[serde(default)]
    pub tools: ToolsConfig,
    #[serde(default)]
    pub email: EmailConfig,
    #[serde(default)]
    pub monitor: MonitorConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GatewayConfig {
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_bind")]
    pub bind: String,
    #[serde(default)]
    pub token: Option<String>,
}

impl Default for GatewayConfig {
    fn default() -> Self {
        Self {
            port: default_port(),
            bind: default_bind(),
            token: None,
        }
    }
}

impl GatewayConfig {
    pub fn listen_addr(&self) -> String {
        format!("{}:{}", self.bind, self.port)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct AgentConfig {
    #[serde(default = "default_provider")]
    pub provider: String,
    #[serde(default)]
    pub api_key: String,
    #[serde(default)]
    pub base_url: String,
    #[serde(default)]
    pub model: String,
    #[serde(default)]
    pub system_prompt: String,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            provider: default_provider(),
            api_key: String::new(),
            base_url: String::new(),
            model: String::new(),
            system_prompt: String::new(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct ChannelsConfig {
    #[serde(default)]
    pub telegram: TelegramConfig,
    #[serde(default)]
    pub discord: DiscordConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DiscordConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub bot_token: String,
    #[serde(default)]
    pub allowed_guild_ids: Vec<u64>,
    #[serde(default = "default_true")]
    pub mention_only: bool,
}

impl Default for DiscordConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            bot_token: String::new(),
            allowed_guild_ids: Vec::new(),
            mention_only: true,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct TelegramConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub bot_token: String,
    #[serde(default)]
    pub allowed_user_ids: Vec<u64>,
    #[serde(default = "default_true")]
    pub stream_edit: bool,
}

impl Default for TelegramConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            bot_token: String::new(),
            allowed_user_ids: Vec::new(),
            stream_edit: true,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct GitHubConfig {
    #[serde(default)]
    pub token: String,
    #[serde(default)]
    pub repos: Vec<String>,
    #[serde(default)]
    pub notify_discord_channel: String,
}

impl GitHubConfig {
    /// Parse the notify channel ID as u64.
    pub fn notify_discord_channel_id(&self) -> Option<u64> {
        if self.notify_discord_channel.is_empty() {
            None
        } else {
            self.notify_discord_channel.parse().ok()
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct CronConfig {
    #[serde(default = "default_github_scan_cron")]
    pub github_scan: String,
    #[serde(default)]
    pub auto_pr: bool,
    #[serde(default = "default_system_check_cron")]
    pub system_check: String,
    #[serde(default = "default_email_scan_cron")]
    pub email_scan: String,
}

impl Default for CronConfig {
    fn default() -> Self {
        Self {
            github_scan: default_github_scan_cron(),
            auto_pr: false,
            system_check: default_system_check_cron(),
            email_scan: default_email_scan_cron(),
        }
    }
}

fn default_system_check_cron() -> String {
    "0 */5 * * * *".to_string()
}

fn default_email_scan_cron() -> String {
    "0 */15 * * * *".to_string()
}

#[derive(Debug, Clone, Deserialize)]
pub struct ToolsConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_workspace_dir")]
    pub workspace_dir: String,
    #[serde(default = "default_true")]
    pub allow_exec: bool,
    #[serde(default = "default_exec_timeout")]
    pub exec_timeout_secs: u64,
}

impl Default for ToolsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            workspace_dir: default_workspace_dir(),
            allow_exec: true,
            exec_timeout_secs: default_exec_timeout(),
        }
    }
}

fn default_workspace_dir() -> String {
    ".".to_string()
}

fn default_exec_timeout() -> u64 {
    30
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct EmailConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_imap_host")]
    pub imap_host: String,
    #[serde(default = "default_imap_port")]
    pub imap_port: u16,
    #[serde(default = "default_smtp_host")]
    pub smtp_host: String,
    #[serde(default = "default_smtp_port")]
    pub smtp_port: u16,
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub password: String,
}

fn default_imap_host() -> String {
    "imap.gmail.com".to_string()
}
fn default_imap_port() -> u16 {
    993
}
fn default_smtp_host() -> String {
    "smtp.gmail.com".to_string()
}
fn default_smtp_port() -> u16 {
    587
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct MonitorConfig {
    #[serde(default)]
    pub services: Vec<String>,
    #[serde(default)]
    pub endpoints: Vec<String>,
    #[serde(default)]
    pub docker: bool,
    #[serde(default)]
    pub pm2: bool,
}

fn default_github_scan_cron() -> String {
    "0 0 9 * * *".to_string()
}

fn default_port() -> u16 {
    18789
}

fn default_bind() -> String {
    "0.0.0.0".to_string()
}

fn default_provider() -> String {
    "anthropic".to_string()
}

fn default_true() -> bool {
    true
}

/// Resolve the config file path in priority order:
/// 1. CLI --config flag
/// 2. RUSTCLAW_CONFIG env var
/// 3. ~/.rustclaw/config.toml
fn resolve_config_path(cli_path: Option<&str>) -> PathBuf {
    if let Some(p) = cli_path {
        return PathBuf::from(p);
    }
    if let Ok(p) = std::env::var("RUSTCLAW_CONFIG") {
        return PathBuf::from(p);
    }
    let mut home = dirs_path();
    home.push("config.toml");
    home
}

fn dirs_path() -> PathBuf {
    if let Some(home) = std::env::var_os("HOME") {
        let mut p = PathBuf::from(home);
        p.push(".rustclaw");
        p
    } else {
        PathBuf::from(".rustclaw")
    }
}

impl AppConfig {
    pub fn load(cli_path: Option<&str>) -> anyhow::Result<Self> {
        let path = resolve_config_path(cli_path);

        let mut builder = config::Config::builder();

        // File source (optional — missing file is fine)
        if path.exists() {
            builder = builder.add_source(
                config::File::from(path.as_ref()).required(false),
            );
        }

        // Environment overlay: RUSTCLAW__AGENT__API_KEY etc.
        builder = builder.add_source(
            config::Environment::with_prefix("RUSTCLAW").separator("__"),
        );

        // Special: support bare ANTHROPIC_API_KEY / OPENAI_API_KEY env vars
        if let Ok(key) = std::env::var("ANTHROPIC_API_KEY") {
            builder = builder.set_override("agent.api_key", key)?;
        } else if let Ok(key) = std::env::var("OPENAI_API_KEY") {
            builder = builder
                .set_override("agent.api_key", key)?
                .set_override("agent.provider", "openai")?;
        }

        let settings = builder.build()?;

        let cfg: AppConfig = settings.try_deserialize().unwrap_or_default();

        Ok(cfg)
    }
}
