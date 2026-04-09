# r/selfhosted post

**Title:** RustClaw — Self-hosted AI Agent in a single 7.5MB binary (Telegram + Discord + GitHub + Email monitoring, 14MB RAM)

**Body:**

I built a self-hosted AI agent that fits in **7.5 MB** and uses **14 MB RAM** idle. It's basically a personal AI butler that connects to your messaging apps, monitors your infrastructure, and can even open GitHub PRs automatically.

### What can it do?

**Chat channels:**
- Telegram bot (long polling, streaming responses)
- Discord bot (@mention, server management commands)
- WebSocket gateway (API access)

**Infrastructure monitoring:**
- Docker container status + auto-restart alerts
- PM2 process monitoring
- HTTP endpoint health checks
- System stats (CPU, memory, disk)
- Cron-based scheduled checks → alerts to Discord

**Productivity:**
- Email scanning (IMAP) with classification
- GitHub issue/PR scanning
- Auto-generate PRs from issues using LLM analysis
- 22 built-in tools the AI can use autonomously

**Memory:**
- Remembers conversations across restarts (SQLite)
- Extracts facts from your conversations automatically
- Knowledge graph for entity relationships
- Tell it your name in Telegram, it remembers in Discord

### Why it's great for self-hosting

| | RustClaw | Typical AI agent |
|---|---|---|
| Binary | 7.5 MB, single file | Node.js + hundreds of npm packages |
| RAM | 14 MB idle | 500 MB – 1 GB+ |
| Startup | < 100ms | 5-10 seconds |
| Dependencies | None (static binary) | Node.js, Python, Docker, etc. |
| Database | SQLite (embedded) | PostgreSQL, Redis, vector DB |

You can literally `scp` the binary to a $5 VPS and run it. Or a Raspberry Pi. Seriously.

### LLM backends

Works with anything OpenAI-compatible:
- **Ollama** (local, private, free)
- **Anthropic** (Claude)
- **OpenAI** (GPT-4o)
- **Google Gemini**
- Any local model via Ollama

### Config

One TOML file. All settings can be overridden with environment variables. Example:

```toml
[agent]
provider = "openai"
base_url = "http://127.0.0.1:11434"  # Ollama
model = "qwen2.5:32b"

[channels.telegram]
enabled = true
bot_token = "..."

[channels.discord]
enabled = true
bot_token = "..."
```

Docker support included, but honestly just running the binary directly is simpler.

**GitHub:** https://github.com/Adaimade/RustClaw

MIT licensed. Open to contributions.
