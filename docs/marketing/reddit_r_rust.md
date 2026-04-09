# r/rust post

**Title:** RustClaw — AI Agent Framework in 6K Lines of Rust (7.5MB binary, 14MB RAM, 97% tool accuracy)

**Body:**

Hey r/rust! I built an AI agent framework entirely in Rust that replaces a 430K-line TypeScript project (OpenClaw). Here are the numbers:

- **7.5 MB** static binary (release + strip + LTO)
- **14 MB** idle RAM with Telegram + Discord bots running
- **< 100ms** startup time
- **5,918 lines** of Rust across 30 files
- **97%** accuracy on a 500-question tool-calling benchmark
- **0%** hallucination rate on adversarial prompts

### What it does

RustClaw is a personal AI agent that connects to Telegram, Discord, and WebSocket. It has 22 built-in tools (file ops, shell execution, code search, Docker/PM2 monitoring, email, GitHub integration) and supports MCP for connecting external tool servers.

The memory system is inspired by [mem0](https://github.com/mem0ai/mem0) — three-tier architecture: short-term conversation history, long-term vector memory with LLM-powered fact extraction and deduplication, and a knowledge graph with entity-relation tracking. All backed by SQLite, zero external services.

### Technical highlights that Rustaceans might appreciate

- **tokio + axum** for async HTTP/WS
- **teloxide** for Telegram, **serenity** for Discord
- **rusqlite (bundled)** for persistence — no external DB needed
- **reqwest** with streaming SSE parsing for LLM responses
- Tool calling works with both Anthropic and OpenAI function-calling formats
- History compression kicks in after 10 messages (old messages summarized, recent kept)
- Claude Code-inspired tool ordering: understand (read/list/search) → act (run/write/patch) → check (process/docker/ping)
- 14 dangerous command patterns blocked at the executor level
- MCP client via stdio JSON-RPC — tools auto-discovered at startup

### The fun part

The entire project was written by Claude Code in a single session. Zero human-written code. The benchmark, the memory system, the Discord bot commands — all AI-generated Rust that passes `cargo clippy` with zero warnings.

**GitHub:** https://github.com/Adaimade/RustClaw

Happy to answer questions about the architecture or benchmark methodology. The 500-question benchmark is also open-sourced in a separate repo.

---

*Built with Rust because we could. Built with Claude Code because... well, why not?*
