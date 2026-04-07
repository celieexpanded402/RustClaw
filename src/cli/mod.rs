use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "rustclaw",
    about = "A lightweight Rust alternative to OpenClaw",
    version
)]
pub struct Cli {
    /// Path to config file
    #[arg(short, long, global = true)]
    pub config: Option<String>,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Start the gateway server (foreground, logs to stdout)
    Gateway,

    /// Send a message to the agent and print the response
    Agent {
        /// The message to send
        message: String,

        /// Stream output token by token
        #[arg(short, long, default_value_t = true)]
        stream: bool,
    },

    /// Connect to local gateway and print status
    Status,

    /// Health check against local gateway
    Health,

    /// GitHub operations
    #[command(subcommand)]
    Github(GithubCommand),
}

#[derive(Subcommand, Debug)]
pub enum GithubCommand {
    /// Scan configured repos and print open issues / PRs
    Scan,

    /// Auto-generate a PR to fix a GitHub issue
    Fix {
        /// Issue number
        issue: u64,

        /// Repository (owner/repo). Defaults to the first repo in config.
        #[arg(short, long)]
        repo: Option<String>,
    },
}
