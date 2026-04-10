# RustClaw

A lightweight Rust alternative to OpenClaw. Single binary.

## Bloat Budget (HARD LIMITS — block new work that exceeds these)

| Metric | Current (2026-04-11) | Hard ceiling |
|---|---|---|
| Release binary | 7.5 MB | **< 10 MB** |
| Idle RSS | 14 MB | **< 20 MB** |
| Lines of Rust | 5,296 | **< 7,000** |
| Cargo dependencies | 33 | **< 40** |

If a proposed change would push any metric past its ceiling, **stop and ask the user** before proceeding. Do not silently accept the regression.

### Three questions before adding any new feature or dependency

Every new feature, every new crate, every new module must answer these in the PR/commit message:

1. **Can this be a `cargo` feature flag?** — If only some users need it (e.g. Slack channel, RAG, Web UI), gate it behind `#[cfg(feature = "...")]` so the default build stays small. Default features should remain a strict subset.
2. **Is there a lighter alternative?** — Before adding heavy crates (serenity, lettre, octocrab, etc.), check for `*-minimal` variants, feature flag pruning (`default-features = false`), or a hand-rolled reqwest call. Document why the chosen crate won.
3. **What is the cost?** — Run `cargo build --release && ls -lh target/release/rustclaw` and `/usr/bin/time -l ./target/release/rustclaw gateway` (or equivalent) **before and after**, and write the delta into the commit message: `+0.4 MB binary, +2 MB RSS, +1 dep`.

### Why these limits exist

The original RustClaw vision was 6 MB binary / 7.9 MB RAM / 4,312 lines. We have already drifted +25% / +77% / +23% while adding Discord, GitHub, Email, Cron, MCP, three-tier memory, and 22 tools. That growth is justified by features delivered — but the trajectory cannot continue. The roadmap (Web UI, Slack/LINE, RAG, multi-agent, WASM plugins) **will break the budget if implemented carelessly**.

The whole point of RustClaw is "the core of an AI agent doesn't need a gigabyte of RAM." If we hit 50 MB RAM and 15 MB binary, that thesis stops being interesting.

## Architecture

### Module Structure

- `src/main.rs` — CLI dispatch (clap subcommands) + startup orchestration
- `src/cli/mod.rs` — Subcommands: `gateway`, `agent`, `status`, `health`, `github`
- `src/config.rs` — TOML config + env overlay. Priority: CLI flag > RUSTCLAW_CONFIG env > ~/.rustclaw/config.toml. Also reads ANTHROPIC_API_KEY / OPENAI_API_KEY.
- `src/gateway/` — WebSocket gateway (OpenClaw-compatible protocol on ws://bind:port/ws)
  - `server.rs` — Axum HTTP server, routes: `/ws`, `/health`. Shared `AppState` via `Arc`.
  - `protocol.rs` — Inbound/outbound frame types with serde. Tagged enum dispatch for `connect`/`auth`/`req`.
  - `connection.rs` — Per-connection lifecycle: challenge handshake → auth → req/res loop. Streams agent tokens.
- `src/agent/runner.rs` — LLM streaming + agentic loop via reqwest. Providers: Anthropic (SSE content_block_delta) and OpenAI-compatible (SSE choices delta, used for Ollama/Gemini too). `run_streaming()` with FnMut callback, `chat_stream()` returns mpsc::Receiver. 120s timeout, max 10 tool iterations, history compression after 10 messages.
- `src/channels/`
  - `telegram.rs` — Telegram bot via teloxide (long polling). Stream edit mode: send placeholder then editMessageText. Retry 3x. ACL via allowed_user_ids. Validates token via `bot.get_me()` before dispatcher starts.
  - `discord.rs` — Discord bot via serenity. @mention trigger, server management commands, `scan` / `fix issue #N` / `pr status`. Requires MESSAGE CONTENT INTENT enabled in Developer Portal.
  - `webchat.rs` — Browser WebSocket chat endpoint.
- `src/tools/` — 22 tools dispatched via `executor.rs` match arms. Subgroups: `fs` (read/list/search/patch), `exec` (run_command with 14-pattern danger blocklist), `system` (process/docker/stats/ping), `github` (scan/fix/PR), `discord` (channel/role/member ops), `email` (IMAP+SMTP), `mcp` (MCP client routing). Output truncated to 4000 chars. Patch files verified before modification.
- `src/session/`
  - `store.rs` — In-memory `Arc<RwLock<HashMap<String, Session>>>`. Sessions auto-created. Telegram uses `telegram:{chat_id}` key, Discord `discord:{guild_id}:{channel_id}`.
  - `memory.rs` — `MemoryManager` wraps `Arc<rustmem::MemoryManager>` (R-Mem crate). Mixed-mode recall across three scopes: local (session) + user (cross-channel) + global. Constant `GLOBAL_SCOPE = "global:system"`.
- `src/cron/mod.rs` — Tokio-cron-scheduler jobs: system monitoring alerts, email classification sweeps, GitHub repo scans. Notifications routed to Discord.

### Config Structure

```toml
[gateway]
port = 18789            # u16
bind = "127.0.0.1"      # String
token = ""              # Option<String>, empty = no auth

[agent]
provider = "anthropic"  # "anthropic" | "openai"
api_key = ""
base_url = ""           # empty = provider default
model = ""              # empty = provider default
system_prompt = ""

[channels.telegram]
enabled = false
bot_token = ""
allowed_user_ids = []   # Vec<u64>, empty = all
stream_edit = true
```

### Build

```bash
cargo build --release && strip target/release/rustclaw
```

### Key Design Decisions

- Single binary with clap subcommands
- Config: config crate with TOML file + env overlay (RUSTCLAW__ prefix)
- Gateway + channels share a `MemoryManager` (which wraps `SessionStore` + R-Mem) via clone (Arc internally)
- AgentRunner is cloneable via config (creates new reqwest::Client per spawn, 120s timeout)
- Telegram, Discord, cron, webchat all spawned as tokio tasks alongside gateway server
- R-Mem (`rustmem` crate) is the only memory backend — no internal embedding/extraction code
- Tool execution is understand-first ordered (read/list/search before run_command/write/patch) for better LLM tool selection
- Dangerous shell patterns blocked at the `tools/exec.rs` layer, not relying on LLM to refuse
