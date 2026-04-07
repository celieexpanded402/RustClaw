# RustClaw

A lightweight Rust alternative to OpenClaw. Single binary, < 5MB, < 20MB RAM.

## Architecture

### Module Structure

- `src/main.rs` — CLI dispatch (clap subcommands) + startup orchestration
- `src/cli/mod.rs` — Subcommands: `gateway`, `agent`, `status`, `health`
- `src/config.rs` — TOML config + env overlay. Priority: CLI flag > RUSTCLAW_CONFIG env > ~/.rustclaw/config.toml. Also reads ANTHROPIC_API_KEY / OPENAI_API_KEY.
- `src/gateway/` — WebSocket gateway (OpenClaw-compatible protocol on ws://bind:port/ws)
  - `server.rs` — Axum HTTP server, routes: `/ws`, `/health`. Shared `AppState` via `Arc`.
  - `protocol.rs` — Inbound/outbound frame types with serde. Tagged enum dispatch for `connect`/`auth`/`req`.
  - `connection.rs` — Per-connection lifecycle: challenge handshake → auth → req/res loop. Streams agent tokens.
- `src/agent/runner.rs` — LLM streaming via reqwest. Providers: Anthropic (SSE content_block_delta) and OpenAI-compatible (SSE choices delta). `run_streaming()` with FnMut callback, `chat_stream()` returns mpsc::Receiver.
- `src/channels/telegram.rs` — Telegram bot via teloxide (long polling). Stream edit mode: send placeholder then editMessageText. Retry 3x. ACL via allowed_user_ids.
- `src/session/store.rs` — In-memory `Arc<RwLock<HashMap<String, Session>>>`. Sessions auto-created. Telegram uses `telegram:{chat_id}` key.

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
- Gateway + channels share a SessionStore via clone (Arc internally)
- AgentRunner is cloneable via config (creates new reqwest::Client per spawn)
- Telegram channel spawned as tokio task alongside gateway server
