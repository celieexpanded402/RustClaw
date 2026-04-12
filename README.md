<div align="center">

# RustClaw

### AI Agent Framework — in Rust

**A lean replacement for [OpenClaw](https://github.com/nicepkg/OpenClaw).**<br>
**Single binary. 22 tools. Three-tier memory. Telegram + Discord + MCP.**

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.85+-orange.svg)](https://www.rust-lang.org/)
[![Built with Claude Code](https://img.shields.io/badge/Built%20with-Claude%20Code-blueviolet)](https://claude.ai)

**7.5 MB binary** · **14 MB RAM** · **5,296 lines** · **98.9% BFCL** · **95.5% T-Eval** · **4.3× faster with MoE**

[Quick Start](#-quick-start) · [Features](#-features) · [Benchmark](#-benchmark) · [Architecture](#-architecture) · [Roadmap](#-roadmap)

🌐 [繁體中文](docs/README.zh-TW.md) · [简体中文](docs/README.zh-CN.md) · [한국어](docs/README.ko.md)

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
<tr><td>📝 Code</td><td><strong>5,296 lines</strong></td><td>~430,000 lines</td></tr>
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

**🔧 Actually does things** — 98.9% on the industry-standard BFCL benchmark (1,000 questions). The bot reads your files, runs your commands, creates PRs — it doesn't just describe what it *would* do.

**🔌 MCP-ready** — Connect any MCP server. Tools auto-discovered and routed transparently. Your LLM sees one unified tool list — local and remote, no difference.

**📈 Benchmarked and proven** — 1,000-question BFCL + 2,146-question T-Eval + 500-question internal benchmark. Dual-model strategy: MoE for speed (2.6s/q), dense for accuracy (99.7%).

**⚙️ Claude Code inspired** — Understand-first tool ordering, history compression, workspace context loading, error retry hints. The same patterns that make Claude Code effective, applied to an open-source agent.

---

## 🚀 Quick Start

### One-line Install (recommended)

**macOS / Linux:**
```bash
curl -sSL https://raw.githubusercontent.com/Adaimade/RustClaw/main/install.sh | sh
```

**Windows (PowerShell):**
```powershell
irm https://raw.githubusercontent.com/Adaimade/RustClaw/main/install.ps1 | iex
```

This downloads the pre-built binary, adds it to PATH, and creates a default config. Works on macOS (Intel/Apple Silicon), Linux (x86/ARM), and Windows.

### Build from Source

| Requirement | Install |
|---|---|
| Rust 1.85+ | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` |
| LLM backend | [Ollama](https://ollama.com), [OpenAI](https://platform.openai.com), [Anthropic](https://console.anthropic.com), or [Gemini](https://ai.google.dev) |

```bash
git clone https://github.com/Adaimade/RustClaw.git && cd RustClaw
cargo build --release && strip target/release/rustclaw
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

Memory is delegated to [**R-Mem**](https://github.com/Adaimade/R-Mem) — a separate Rust crate that handles vector recall, fact extraction, contradiction resolution, and entity-relation graphs. RustClaw is a thin wrapper that adds mixed-mode scoping on top.

**Mixed-mode recall** — three scopes merged on every query:

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

Tested on the **official [Gorilla BFCL](https://github.com/ShishirPatil/gorilla)** benchmark — the industry standard for evaluating function calling. Dual-model comparison on Mac Mini 2024 (M4 Pro, 64 GB):

| Test | qwen3-coder:30b (MoE) | qwen2.5:32b (dense) | Speed diff |
|---|---|---|---|
| **simple_python** (400) | **100%** · 1.5s/q | 99.75% · 7.3s/q | 4.9× |
| **multiple** (200) | 97% · 2.4s/q | **99.5%** · 8.4s/q | 3.5× |
| **parallel** (200) | 99.5% · 2.9s/q | **100%** · 12.0s/q | 4.1× |
| **parallel_multiple** (200) | 98% · 3.4s/q | **100%** · 15.7s/q | 4.6× |
| **Overall** (1,000) | **98.9%** · 2.6s/q | **99.7%** · 10.8s/q | **4.3×** |

> MoE model trades -0.8% accuracy for 4.3× speed. Both models exceed 98% across all categories.

### T-Eval (Shanghai AI Lab)

Tested on **[T-Eval](https://github.com/open-compass/T-Eval)** — Shanghai AI Lab's tool-use evaluation suite covering planning, retrieval, review, and instruction following:

| Test | Score | Questions | Speed |
|---|---|---|---|
| **T-Eval retrieve** | **98%** (542/553) | 553 | 14.5s/q |
| **T-Eval plan** | **96%** (535/553) | 553 | 25.6s/q |
| **T-Eval review** | **96%** (472/487) | 487 | 3.5s/q |
| **T-Eval instruct** | **92%** (514/553) | 553 | 8.2s/q |

> 2,146 questions across four core categories. Average **95.5%** — strong tool selection, multi-step planning, and self-review.

### Internal Benchmark

500-question tool calling benchmark (qwen2.5:32b, local Ollama). Not yet re-tested on qwen3-coder:30b:

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
├── session/             SessionStore (history) + MemoryManager (R-Mem wrapper)
└── cron/                Scheduled jobs (system, email, GitHub)
```

**27 files · 5,296 lines · 7.5 MB binary · Zero external services**

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
| ✅ | Cross-platform install (macOS / Linux / Windows) |
| ✅ | Multi-model routing (per-channel model override via config) |
| 🔲 | Slack / LINE channels |
| 🔲 | Prometheus metrics |

Community contributions welcome — open an issue or PR.

---

<div align="center">

**MIT License** · v0.5.0

Created by [Ad Huang](https://github.com/Adaimade) with [Claude Code](https://claude.ai)

*The framework is there. The rest is up to the community.*

</div>
