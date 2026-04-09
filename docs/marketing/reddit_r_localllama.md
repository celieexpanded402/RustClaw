# r/LocalLLaMA post

**Title:** Built an AI agent framework that actually uses tool calling with local models — 97% accuracy with qwen2.5:32b on Ollama

**Body:**

I've been frustrated with AI agent frameworks that only work well with GPT-4 or Claude. So I built one specifically designed to work with local models via Ollama, and benchmarked it thoroughly.

### The benchmark results

500 questions, running against **qwen2.5:32b** on Ollama (local, Apple Silicon):

| Category | Accuracy |
|---|---|
| Core operations | 92% |
| Basic tool use | 95% |
| Medium tasks | **100%** |
| Advanced reasoning | 98% |
| Hallucination traps | **100%** |
| Multi-step chains | 99% |
| **Overall** | **97%** |

**0 timeouts. 0 hallucinations on adversarial prompts. Average 38 seconds per question.**

I also tested gemma4 (E4b and 26b) — both scored **0%** because they don't support Ollama's function calling format. Model matters a lot.

### How I got to 97%

Started at 81%. Here's what made the difference:

1. **System prompt with few-shot examples** (24 examples, Chinese + English) — biggest single improvement, especially for English queries
2. **"Understand first" tool ordering** — read_file and search_code appear before write_file and run_command in the tool definitions. The model naturally reads before acting.
3. **Negative constraints in tool descriptions** — "Do NOT use read_file when you need to find which file. Use search_code first." This dramatically reduced wrong tool selection.
4. **History compression** — after 10 messages, old ones get summarized. Keeps the context window manageable for 32B models.
5. **120s timeout with graceful fallback** — instead of crashing on slow responses, return partial results.
6. **Error retry hints** — when a tool fails, the error message includes "Try a different approach or use another tool to investigate first." The model actually recovers.

### What is it?

RustClaw is an AI agent framework in Rust. Single binary (7.5 MB), runs on 14 MB RAM. Connects to Telegram, Discord, and WebSocket.

22 built-in tools: file read/write/patch, shell execution, code search, Docker/PM2 monitoring, email (IMAP/SMTP), GitHub integration, and MCP client support.

Three-tier memory system: conversation history + LLM-extracted facts with deduplication + knowledge graph. All in SQLite, no vector database needed.

### Ollama-first design

- Uses OpenAI-compatible `/v1/chat/completions` with `tools` parameter
- Function calling format matches what Ollama expects
- Embedding via `nomic-embed-text` for the memory system
- Works offline — no cloud API needed
- Tested with: qwen2.5:32b ✅, qwen2.5:14b (untested), llama3 (untested), gemma4 ❌

### The system prompt that works

The key insight: local models need **more explicit few-shot examples** than cloud models. My system prompt has:

- 5 hard rules ("NEVER ask permission", "NEVER guess file contents")
- Tool descriptions with "USE when" and "DO NOT USE when" for each tool
- 9 Chinese + 15 English few-shot examples showing exact tool call → response patterns
- Response rules ("minimum 2 sentences with specific data from tool results")

This turned qwen2.5:32b from "sometimes calls tools" to "almost always calls the right tool."

### Try it

```bash
git clone https://github.com/Adaimade/RustClaw.git
cd RustClaw && cargo build --release
# Set up Ollama + qwen2.5 model, then:
./target/release/rustclaw agent "List all files in src/ and count total lines"
```

**GitHub:** https://github.com/Adaimade/RustClaw

The 500-question benchmark is open-sourced separately. Happy to share the full system prompt if anyone wants to adapt it for their own agent.

---

*If your local model doesn't call tools, it's probably the prompt, not the model.*
