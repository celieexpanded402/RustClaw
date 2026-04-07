# RustClaw

**A lean, mean Rust replacement for OpenClaw.** Single binary. No runtime. Full agent capabilities.

|                   | **RustClaw**       | **OpenClaw**         |
|-------------------|--------------------|----------------------|
| Binary / Runtime  | **6 MB** static    | requires Node.js 24 + npm |
| Idle Memory (RSS) | **7.9 MB**         | 1 GB+                |
| Startup Time      | **< 100 ms**       | 5-10 s               |
| Lines of Code     | **~4,000**         | ~430,000             |
| Dependencies      | Compiled in        | npm install...       |

---

## Why

OpenClaw does a lot. Too much, for most use cases. If you just need an LLM agent that talks to Telegram, Discord, and GitHub — with tool access and a WebSocket control plane — you don't need 430K lines of TypeScript and a 1GB memory footprint.

RustClaw is the 80/20 version: the features that matter, in a single `cargo build`.

Built entirely with [Claude Code](https://claude.ai/code). Zero human-written code.

---

## Quick Start

### Prerequisites

- Rust toolchain (`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`)
- An LLM backend: [Ollama](https://ollama.com) (local) or an [Anthropic API key](https://console.anthropic.com)

### Build

```bash
git clone https://github.com/Adaimade/RustClaw.git && cd RustClaw
cargo build --release
# Binary: target/release/rustclaw (6 MB)
```

### Configure

```bash
mkdir -p ~/.rustclaw
cp config.example.toml ~/.rustclaw/config.toml
```

Minimal config for Ollama:

```toml
[agent]
provider = "openai"
api_key = "ollama"
base_url = "http://127.0.0.1:11434"
model = "qwen2.5:32b"
system_prompt = "You are a coding assistant with tool access."
```

Or for Anthropic:

```toml
[agent]
provider = "anthropic"
api_key = "sk-ant-..."
model = "claude-sonnet-4-20250514"
```

### Run

```bash
# Start everything (gateway + channels + cron)
rustclaw gateway

# Or just talk to the agent directly
rustclaw agent "List all .rs files in src/"
```

---

## Features

### Gateway (WebSocket Control Plane)

OpenClaw-compatible WebSocket protocol on `ws://127.0.0.1:18789/ws`.

Full handshake: `connect` -> `challenge` -> `auth` -> `hello-ok`, then request/response with streaming agent events.

```bash
rustclaw gateway    # start server
rustclaw health     # HTTP health check
rustclaw status     # WebSocket handshake test
```

### Channels

#### Telegram

Long polling via teloxide. Streaming responses with progressive message editing.

```toml
[channels.telegram]
enabled = true
bot_token = "123456:ABC-..."
allowed_user_ids = []    # empty = allow all
stream_edit = true       # edit-in-place streaming
```

#### Discord

Serenity-based bot. Responds to @mentions and DMs. Has server management tools.

```toml
[channels.discord]
enabled = true
bot_token = "your-token"
allowed_guild_ids = []   # empty = all servers
mention_only = true      # only respond to @mentions and DMs
```

Setup: [Discord Developer Portal](https://discord.com/developers/applications) -> create app -> Bot tab -> enable **MESSAGE CONTENT INTENT** -> invite with `permissions=274877975552&scope=bot`.

### Tool Calling (Agentic Loop)

The agent has access to tools and uses them autonomously. Supports both Anthropic and OpenAI function calling formats. Max 10 tool iterations per request.

**Built-in tools:**

| Tool | Description |
|---|---|
| `read_file` | Read file contents (auto-truncates >100KB) |
| `write_file` | Write/create files (auto-creates directories) |
| `patch_file` | Find-and-replace in files |
| `list_dir` | Directory tree (depth-limited) |
| `run_command` | Shell execution (sandboxed to workspace, timeout) |
| `search_code` | Grep-like code search (pure Rust) |
| `discord_create_channel` | Create text/voice/category channel |
| `discord_delete_channel` | Delete a channel |
| `discord_create_role` | Create a role with color |
| `discord_set_channel_topic` | Set channel topic |
| `discord_kick_member` | Kick a guild member |
| `discord_ban_member` | Ban a guild member |

```toml
[tools]
enabled = true
workspace_dir = "."
allow_exec = true
exec_timeout_secs = 30
```

### GitHub Integration

Scan repos, auto-generate PRs from issues using LLM analysis.

```toml
[github]
token = "ghp_..."
repos = ["owner/repo"]
notify_discord_channel = "123456789"
```

```bash
rustclaw github scan          # scan all configured repos
rustclaw github fix 123       # auto-PR for issue #123
```

**Auto-PR flow:** fetch issue -> LLM generates fix -> create branch `rustclaw/fix-issue-N` -> commit -> open PR.

Token needs: `repo` scope (or `public_repo` for public only).

### Cron

```toml
[cron]
github_scan = "0 0 9 * * *"  # daily at 09:00 (sec min hr day mon dow)
```

Scan results are logged and optionally posted to a Discord channel.

---

## Discord Commands

When the bot is @mentioned:

```
@RustClaw scan                    # GitHub repo scan report
@RustClaw fix issue #42           # Auto-generate a PR for issue 42
@RustClaw pr status               # List bot-created PRs
@RustClaw read the file src/main.rs and summarize it
@RustClaw create a channel called announcements
@RustClaw what files are in this project?
@RustClaw run cargo test and tell me what fails
```

---

## CLI

```
rustclaw [OPTIONS] <COMMAND>

Commands:
  gateway              Start gateway + all enabled channels + cron
  agent <MESSAGE>      Send a message to the agent (with tool access)
  health               HTTP health check against local gateway
  status               WebSocket status check
  github scan          Scan configured repos
  github fix <N>       Auto-PR for issue N

Options:
  -c, --config <PATH>  Config file path
  -h, --help           Print help
  -V, --version        Print version
```

---

## Architecture

```
                          rustclaw gateway
                               |
              +----------------+----------------+
              |                |                |
         WebSocket        Telegram          Discord
        :18789/ws        (teloxide)       (serenity)
              |                |                |
              +--------+-------+--------+-------+
                       |                |
                  AgentRunner      SessionStore
                  (streaming)      (in-memory)
                       |
              +--------+--------+
              |                 |
         Anthropic API    OpenAI / Ollama
              |
        Tool Executor
              |
    +---------+---------+---------+
    |         |         |         |
  files    shell    search    discord
  r/w/p   exec+to   code     mgmt
```

```
src/
├── main.rs                 # CLI dispatch, startup orchestration
├── cli/mod.rs              # clap subcommands
├── config.rs               # TOML + env config, priority loading
├── gateway/
│   ├── server.rs           # Axum HTTP/WS server
│   ├── protocol.rs         # Inbound/outbound frame types
│   └── connection.rs       # WS lifecycle: handshake -> req/res loop
├── agent/
│   └── runner.rs           # LLM calls: streaming + agentic tool loop
├── channels/
│   ├── telegram.rs         # Telegram bot (long polling, edit streaming)
│   ├── discord.rs          # Discord bot (mentions, commands, tools)
│   └── webchat.rs          # WebChat (stub)
├── tools/
│   ├── executor.rs         # Unified tool dispatch + JSON Schema defs
│   ├── fs.rs               # read, write, patch, list_dir
│   ├── exec.rs             # Shell execution (sandboxed, timeout)
│   ├── search.rs           # Code search (pure Rust)
│   ├── discord.rs          # Discord management API
│   └── github.rs           # GitHub API (octocrab)
├── cron/
│   └── mod.rs              # Scheduled jobs (github_scan, auto_pr)
└── session/
    └── store.rs            # In-memory session history
```

---

## Extending RustClaw

### Adding a new tool

1. Implement your function in `src/tools/` (e.g. `src/tools/my_tool.rs`)
2. Add a match arm in `src/tools/executor.rs` -> `execute_inner()`
3. Add the JSON Schema to `tool_definitions()`
4. Done. The LLM will discover and use it automatically.

### Adding a new channel

1. Create `src/channels/my_channel.rs`
2. Implement a struct with `pub async fn start(self, runner: Arc<AgentRunner>) -> Result<()>`
3. Add config in `src/config.rs` under `ChannelsConfig`
4. Spawn it in `cmd_gateway()` in `src/main.rs`

---

## Configuration Reference

Config is loaded from (in order): `--config` flag, `RUSTCLAW_CONFIG` env, `~/.rustclaw/config.toml`.

All values can be overridden via env vars with `RUSTCLAW__` prefix: `RUSTCLAW__AGENT__API_KEY=sk-...`

Bare `ANTHROPIC_API_KEY` and `OPENAI_API_KEY` env vars are also recognized.

See [`config.example.toml`](config.example.toml) for the full reference.

---

## Development

```bash
cargo build                              # debug build
cargo clippy -- -W clippy::all           # lint
cargo build --release                    # optimized (6 MB)
RUST_LOG=rustclaw=debug rustclaw gateway # verbose logging
```

---

## Roadmap

- [ ] **MCP client** — Model Context Protocol support for external tool servers
- [ ] **Web UI** — lightweight browser dashboard for sessions and logs
- [ ] **Slack channel** — Slack bot integration
- [ ] **LINE channel** — LINE Messaging API
- [ ] **Persistent sessions** — SQLite-backed session store
- [ ] **Multi-agent** — route different channels to different models/prompts
- [ ] **Plugin system** — dynamic tool loading via WASM or shared libs
- [ ] **Metrics** — Prometheus `/metrics` endpoint

Community contributions welcome. Open an issue or PR.

---

## License

MIT
