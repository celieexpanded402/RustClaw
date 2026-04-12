# Changelog

All notable changes to RustClaw are documented in this file.

## [0.5.0] - 2026-04-12

### Added
- **Cross-platform installer** — one-line install for macOS, Linux, and Windows
  - `curl -sSL .../install.sh | sh` (macOS/Linux)
  - `irm .../install.ps1 | iex` (Windows PowerShell)
- **CI/CD release pipeline** — GitHub Actions builds 5 targets on tag push (aarch64-apple-darwin, x86_64-apple-darwin, x86_64-linux, aarch64-linux, x86_64-windows)
- **BFCL benchmark for qwen3-coder:30b** — 98.9% across 1,000 questions at 2.6s/q (4.3× faster than qwen2.5:32b)
- **Gemma4 writing test** — 6-topic Chinese writing comparison across 3 models; gemma4:26b confirmed as best Chinese writing model
- **Vendored OpenSSL feature** — optional `vendored-openssl` Cargo feature for Linux ARM cross-compilation

### Changed
- Default recommended model updated to `qwen3-coder:30b` (MoE, 18 GB) from `qwen2.5:32b` (dense, 19 GB)
- README updated with dual-model BFCL benchmark table, one-line install instructions
- Translated READMEs (zh-TW, zh-CN, ko) synced with all v0.5.0 changes
- Roadmap updated: removed Web UI and WASM plugins, added cross-platform install, marked multi-model routing as in-progress

### Fixed
- Resolved all 15 compiler warnings (zero-warning build)
- Removed dead code: `MAX_HISTORY_MESSAGES`, `load_workspace_context()`, `ProcessInfo` struct, unused imports

### Removed
- Stale benchmark files (v4 timeout-fix, v5-pro, v6 failed run, redundant simple_python subset, outdated BENCHMARK_REPORT.md)
- Web UI and WASM plugin system from roadmap (Web UI covered by existing 5 interfaces; WASM would break binary size budget)

## [0.4.0] - 2026-04-10

### Added
- **BFCL 99.7%** — 1,000 questions on official Berkeley Function Calling Leaderboard (qwen2.5:32b)
  - simple_python: 99.75% (399/400)
  - multiple: 99.5% (199/200)
  - parallel: 100% (200/200)
  - parallel_multiple: 100% (200/200)
- **T-Eval 95.5%** — 2,146 questions across 4 core categories (retrieve 98%, plan 96%, review 96%, instruct 92%)
- Polished README with badges, comparison table, Key Advantages section
- Marketing docs for Reddit and awesome-list submissions

### Changed
- README redesigned with benchmark tables and architecture diagram
- Internal benchmark v3→v5 improvement: 81% → 97%

## [0.3.0] - 2026-04-08

### Added
- **R-Mem integration** — replaced internal memory system with `rustmem` crate
  - Three-tier memory: vector recall + graph (entity-relation) + audit history
  - Mixed-mode scoping: local + user + global
  - Integer ID mapping (prevents LLM UUID hallucination)
- **6 Claude Code-inspired optimizations** — understand-first tool ordering, history compression, workspace context, error retry hints
- 500-question internal benchmark with per-category scoring

### Changed
- `src/session/` reduced from ~600 lines to 219 lines (thin wrapper over rustmem)
- Memory features delegated upstream to R-Mem crate

## [0.2.0] - 2026-04-07

### Added
- **MCP client** — stdio transport, tool auto-discovery and transparent routing
- **System monitoring** — process list, docker status, PM2 status, HTTP ping, system stats
- **Email** — IMAP inbox reading + SMTP sending (Gmail compatible)
- **Cron scheduling** — system check, email scan, GitHub scan on configurable schedules
- **Enhanced GitHub tools** — repo scanning + auto-PR from issues
- **Persistent sessions** — SQLite-backed conversation history

### Changed
- Tool count expanded from 8 to 22

## [0.1.0] - 2026-04-06

### Added
- Initial release
- Gateway WebSocket server (OpenClaw-compatible protocol)
- Agent runner with Anthropic + OpenAI (Ollama) provider support
- SSE streaming with tool calling agentic loop (max 10 iterations)
- Telegram channel (long polling, streaming edit, ACL)
- Discord channel (@mention, server management commands)
- 8 built-in tools: read_file, write_file, patch_file, list_dir, search_code, run_command, process_list, process_check
- TOML config with environment variable overlay
- 14 dangerous command patterns blocked in run_command
- CLI with clap subcommands: gateway, agent, status, health, github
