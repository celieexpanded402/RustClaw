# RustClaw Performance Benchmark Report

**Date:** 2026-04-08
**Platform:** macOS arm64 (Apple Silicon)
**LLM:** Ollama qwen2.5:32b (local)

## System Metrics

| Metric | Value | Note |
|---|---|---|
| **Binary size** | 7.5 MB | release + strip + LTO |
| **Source code** | 30 files / 5,912 lines | |
| **Idle RSS** | 14 MB | gateway + Telegram + Discord |
| **Startup time** | < 100 ms | to listening state |
| **HTTP latency** | ~23 ms | /health endpoint |
| **Agent latency** | ~3-5 sec | warm model, simple query |
| **Agent latency (cold)** | ~38 sec | first request, model loading |
| **Direct dependencies** | 24 crates | |
| **DB file** | 24 KB | empty sessions.db |

## vs OpenClaw

| Metric | RustClaw | OpenClaw |
|---|---|---|
| Binary / Runtime | 7.5 MB static | requires Node.js 24 + npm |
| Idle Memory | 14 MB | 1 GB+ |
| Startup | < 100 ms | 5-10 sec |
| Code | 5,912 lines | ~430,000 lines |

## Tool Calling Benchmark (50 questions)

| Model | Size | Score | Time |
|---|---|---|---|
| gemma4:E4b | 9.6 GB | 0/50 (0%) | N/A (no function calling) |
| gemma4:26b | 17 GB | 0/50 (0%) | N/A (no function calling) |
| **qwen2.5:32b** | 19 GB | **45/50 (90%)** | 20 min |

### By category (qwen2.5:32b):
| Category | Score |
|---|---|
| Daily (1-10) | 10/10 (100%) |
| Basic (11-20) | 7/10 (70%) |
| Medium (21-30) | 8/10 (80%) |
| Advanced (31-40) | 10/10 (100%) |
| Hallucination traps (41-50) | 10/10 (100%) |

## Features

- 22 built-in tools (file, shell, search, Discord mgmt, system monitor, email)
- MCP client support
- Three-tier memory (short-term + long-term vector + graph)
- Mixed-mode recall (local + user + global scopes)
- 3 channels (Telegram, Discord, WebSocket gateway)
- GitHub integration (scan + auto-PR)
- Cron scheduling
