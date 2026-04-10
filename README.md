<div align="center">

# RustClaw

### AI Agent Framework — in Rust

**A lean replacement for [OpenClaw](https://github.com/nicepkg/OpenClaw).**<br>
**Single binary. 22 tools. Three-tier memory. Telegram + Discord + MCP.**

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.85+-orange.svg)](https://www.rust-lang.org/)
[![Built with Claude Code](https://img.shields.io/badge/Built%20with-Claude%20Code-blueviolet)](https://claude.ai)

**7.5 MB binary** · **14 MB RAM** · **5,918 lines** · **99.75% BFCL score** · **0% hallucination**

[Quick Start](#-quick-start) · [Features](#-features) · [Benchmark](#-benchmark) · [Architecture](#-architecture) · [Roadmap](#-roadmap)

🌐 [繁體中文](docs/README.zh-TW.md) · [简体中文](docs/README.zh-CN.md) · [日本語](docs/README.ja.md) · [한국어](docs/README.ko.md) · [Español](docs/README.es.md) · [Português](docs/README.pt.md)

</div>

---

## Why RustClaw?

The idea started with a simple observation: someone rewrote OpenClaw in Go and cut memory usage from 1GB+ down to 35MB. That was impressive. But we asked — could we go further?

Most people don't need 430,000 lines of TypeScript. They need an agent that talks to Telegram, reads their files, runs their code, and opens a GitHub PR when something breaks. That's it.

RustClaw is the 80/20 version of OpenClaw — the features that matter, in a single `cargo build`.

<table>
<tr><td></td><td><strong>RustClaw</strong></td><td><strong>OpenClaw</strong></td></tr>
<tr><td>📦 Binary</td><td><strong>7.5 MB</strong> static</td><td>requires Node.js 24 + npm</td></tr>
<tr><td>💾 Idle RAM</td><td><strong>14 MB</strong></td><td>1 GB+</td></tr>
<tr><td>⚡ Startup</td><td><strong>&lt; 100 ms</strong></td><td>5–10 s</td></tr>
<tr><td>📝 Code</td><td><strong>5,918 lines</strong></td><td>~430,000 lines</td></tr>
<tr><td>🧠 Memory</td><td>Three-tier (vector + graph + history)</td><td>Basic session</td></tr>
<tr><td>🔧 Tools</td><td>22 built-in + MCP</td><td>Plugin system</td></tr>
<tr><td>🤖 LLM</td><td>Anthropic, OpenAI, Ollama, Gemini</td><td>OpenAI</td></tr>
<tr><td>📱 Channels</td><td>Telegram, Discord, WebSocket</td><td>Web UI</td></tr>
</table>

> [!NOTE]
> RustClaw is not trying to replace OpenClaw. It's proof that the core of what makes an AI agent useful doesn't require a gigabyte of RAM. It requires good architecture, the right language, and the willingness to start over with clearer constraints.

Built entirely with [Claude Code](https://claude.ai/code) by [Ad Huang](https://github.com/Adaimade). Zero human-written code.

---

## 💡 Key Advantages

**🪶 Runs anywhere** — 7.5 MB binary, 14 MB RAM. Raspberry Pi, $5 VPS, or your laptop. No Node.js, no Python, no Docker required.

**🧠 Remembers everything** — Three-tier memory (vector + graph + history) with mixed-mode scoping. Tell the bot your name in Telegram, it remembers in Discord. Facts auto-extracted, contradictions auto-resolved.

**🛡️ Safe by design** — 14 dangerous command patterns blocked. Tool output truncated. Patch files verified before modification. Error retry with auto-recovery. 120s timeout with graceful fallback.

**🔧 Actually does things** — 97% tool accuracy on 500-question benchmark. 0% hallucination rate. The bot reads your files, runs your commands, creates PRs — it doesn't just describe what it *would* do.

**🔌 MCP-ready** — Connect any MCP server. Tools auto-discovered and routed transparently. Your LLM sees one unified tool list — local and remote, no difference.

**📈 Benchmarked and proven** — 500-question professional benchmark covering daily ops, coding, system administration, and adversarial prompts. v3→v5 improvement: 81% → 97%. Zero timeouts.

**⚙️ Claude Code inspired** — Understand-first tool ordering, history compression, workspace context loading, error retry hints. The same patterns that make Claude Code effective, applied to an open-source agent.

---

## 🚀 Quick Start

### Prerequisites

| Requirement | Install |
|---|---|
| Rust 1.85+ | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` |
| LLM backend | [Ollama](https://ollama.com), [OpenAI](https://platform.openai.com), [Anthropic](https://console.anthropic.com), or [Gemini](https://ai.google.dev) |

### Build & Run

```bash
git clone https://github.com/Adaimade/RustClaw.git && cd RustClaw
cargo build --release
# → target/release/rustclaw (7.5 MB)
```

### Configure

```bash
mkdir -p ~/.rustclaw
cp config.example.toml ~/.rustclaw/config.toml
```

<table>
<tr>
<td><strong>Ollama (local)</strong></td>
<td><strong>Anthropic</strong></td>
<td><strong>Gemini</strong></td>
</tr>
<tr>
<td>

```toml
[agent]
provider = "openai"
api_key = "ollama"
base_url = "http://127.0.0.1:11434"
model = "qwen2.5:32b"
```

</td>
<td>

```toml
[agent]
provider = "anthropic"
api_key = "sk-ant-..."
model = "claude-sonnet-4-20250514"
```

</td>
<td>

```toml
[agent]
provider = "openai"
api_key = "your-key"
base_url = "https://generativelanguage.googleapis.com/v1beta/openai"
model = "gemini-2.5-flash"
```

</td>
</tr>
</table>

> **Security:** RustClaw binds to `0.0.0.0` by default for cloud deploy. Never put API keys in code — use `~/.rustclaw/config.toml` (gitignored) or environment variables (`RUSTCLAW__AGENT__API_KEY`).

### Run

```bash
# Start everything (gateway + channels + cron + memory)
rustclaw gateway

# One-shot agent call with tool access
rustclaw agent "List all .rs files and count total lines of code"

# GitHub operations
rustclaw github scan
rustclaw github fix 123
```

---

## ✨ Features

### 🔧 Tool Calling (Agentic Loop)

22 built-in tools with autonomous execution. Supports Anthropic and OpenAI function calling. Max 10 iterations per request.

**Layered tool loading** — understand first, then act, then check:

```
👁️ Understand              ⚡ Act                    🔍 Check
├── read_file              ├── run_command           ├── process_check
├── list_dir               ├── write_file            ├── docker_status
└── search_code            └── patch_file            ├── system_stats
                                                     ├── http_ping
💬 Discord (on-demand)     📧 Email (on-demand)      ├── pm2_status
├── create/delete channel  ├── fetch_inbox           └── process_list
├── create_role/set_topic  ├── read_email
└── kick/ban_member        └── send_email
```

**Safety:** 14 dangerous patterns blocked · output truncated to 4000c · patch verification · error retry hints · 120s graceful timeout

### 🧠 Three-Tier Memory

Powered by [R-Mem](https://github.com/Adaimade/R-Mem) architecture.

```
├─ 📝 Short-term ── conversation history (SQLite)
├─ 📦 Long-term ─── LLM fact extraction → dedup → ADD/UPDATE/DELETE/NONE
│    └── Integer ID mapping · contradiction detection · semantic dedup
└─ 🕸️ Graph ─────── entity + relation extraction with soft-delete
```

**Mixed-mode recall** — three scopes merged:

| Scope | Example | Shared across |
|---|---|---|
| Local | `telegram:-100xxx` | Single group |
| User | `user:12345` | All channels for one person |
| Global | `global:system` | Everyone |

### 📱 Channels

| Channel | Features |
|---|---|
| **Telegram** | Long polling · streaming edit · ACL · session history |
| **Discord** | @mention · server management · `scan` / `fix issue #N` / `pr status` |
| **Gateway** | OpenClaw-compatible WebSocket on `:18789/ws` |

### 🔌 MCP Client

```toml
[mcp]
servers = [
  { name = "fs", command = "npx @modelcontextprotocol/server-filesystem /tmp" },
]
```

### 🐙 GitHub · ⏰ Cron · 📧 Email

Auto-scan repos · auto-PR from issues · system monitoring alerts · email classification — all scheduled via cron, notifications to Discord.

---

## 📊 Benchmark

### Berkeley Function Calling Leaderboard (BFCL)

Tested on the **official [Gorilla BFCL](https://github.com/ShishirPatil/gorilla)** benchmark — the industry standard for evaluating function calling:

| Test | Score | Questions | Speed |
|---|---|---|---|
| **BFCL simple_python** | **99.75%** (399/400) | 400 | 7.3s/q |
| **BFCL multiple** | **99.5%** (199/200) | 200 | 8.4s/q |
| **BFCL parallel** | **100%** (200/200) | 200 | 12.0s/q |
| **BFCL parallel_multiple** | **100%** (200/200) | 200 | 15.7s/q |

> 1,000 questions on the official BFCL benchmark. Two perfect scores on parallel function calling.

### Internal Benchmark

500-question tool calling benchmark (qwen2.5:32b, local Ollama):

| Version | Total | Timeout | Speed |
|---|---|---|---|
| v3 baseline | 81% | 74 | 44s/q |
| v4 timeout fix | 85% | 3 | 36s/q |
| **v5 optimized** | **97%** | **0** | **38s/q** |

| Category | v5 Score |
|---|---|
| Core operations | 92% |
| Basic tools | 95% |
| Medium tasks | **100%** |
| Advanced reasoning | 98% |
| Hallucination traps | **100%** |
| Multi-step chains | 99% |

> Benchmark questions available at [AI-Bench](https://github.com/Adaimade/AI-Bench).

---

## 🏗️ Architecture

```
src/
├── main.rs              CLI dispatch + startup
├── cli/mod.rs           clap subcommands
├── config.rs            TOML + env config
├── gateway/             WebSocket server + protocol + handshake
├── agent/runner.rs      LLM streaming + agentic loop + history compression
├── channels/            Telegram (teloxide) + Discord (serenity)
├── tools/               22 tools: fs, shell, search, discord, email, system, github, mcp
├── session/             MemoryManager + SQLite store + graph + embedding + extraction
└── cron/                Scheduled jobs (system, email, GitHub)
```

**30 files · 5,918 lines · 7.5 MB binary · Zero external services**

---

## 🗺️ Roadmap

| Status | Feature |
|---|---|
| ✅ | Tool calling (22 tools + agentic loop) |
| ✅ | Three-tier memory (vector + graph + mixed scope) |
| ✅ | Telegram + Discord channels |
| ✅ | MCP client (transparent tool routing) |
| ✅ | GitHub integration (scan + auto-PR) |
| ✅ | System monitoring + cron alerts |
| ✅ | Email (IMAP + SMTP) |
| ✅ | SQLite persistence |
| 🔲 | Web UI dashboard |
| 🔲 | Slack / LINE channels |
| 🔲 | RAG (document search) |
| 🔲 | Multi-agent routing |
| 🔲 | WASM plugin system |
| 🔲 | Prometheus metrics |

Community contributions welcome — open an issue or PR.

---

<div align="center">

**MIT License** · v0.4.0

Created by [Ad Huang](https://github.com/Adaimade) with [Claude Code](https://claude.ai)

*The framework is there. The rest is up to the community.*

</div>
