🌐 [English](../README.md) · [简体中文](README.zh-CN.md) · [日本語](README.ja.md) · [한국어](README.ko.md) · [Español](README.es.md) · [Português](README.pt.md)

# RustClaw

**OpenClaw 的 Rust 精簡替代品。** 單一 binary。不需要 runtime。完整 agent 能力。

|                   | **RustClaw**       | **OpenClaw**              |
|-------------------|--------------------|---------------------------|
| Binary / Runtime  | **6 MB** 靜態連結   | 需要 Node.js 24 + npm     |
| 閒置記憶體 (RSS)   | **7.9 MB**         | 1 GB+                     |
| 啟動時間           | **< 100 ms**       | 5-10 秒                   |
| 程式碼行數         | **~4,000**         | ~430,000                  |
| 依賴管理           | 編譯時打包          | npm install...             |

---

## 為什麼做這個

OpenClaw 功能很多，但對大多數場景來說太重了。如果你只需要一個能串接 Telegram、Discord、GitHub 的 LLM agent——有 tool 呼叫能力和 WebSocket 控制平面——你不需要 43 萬行 TypeScript 和 1GB 的記憶體開銷。

RustClaw 是 80/20 法則的產物：把真正重要的功能裝進一個 `cargo build`。

完全使用 [Claude Code](https://claude.ai/code) 構建，零人工撰寫程式碼。

---

## 快速開始

### 前置需求

- Rust 工具鏈（`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`）
- LLM 後端：[Ollama](https://ollama.com)（本地）或 [Anthropic API key](https://console.anthropic.com)

### 編譯

```bash
git clone https://github.com/Adaimade/RustClaw.git && cd RustClaw
cargo build --release
# Binary: target/release/rustclaw (6 MB)
```

### 設定

```bash
mkdir -p ~/.rustclaw
cp config.example.toml ~/.rustclaw/config.toml
```

Ollama 最小設定：

```toml
[agent]
provider = "openai"
api_key = "ollama"
base_url = "http://127.0.0.1:11434"
model = "qwen2.5:32b"
system_prompt = "You are a coding assistant with tool access."
```

Anthropic 設定：

```toml
[agent]
provider = "anthropic"
api_key = "sk-ant-..."
model = "claude-sonnet-4-20250514"
```

### 啟動

```bash
# 啟動所有服務（gateway + channels + cron）
rustclaw gateway

# 或直接跟 agent 對話
rustclaw agent "列出 src/ 目錄下的所有 .rs 檔案"
```

---

## 功能

### Gateway（WebSocket 控制平面）

相容 OpenClaw 的 WebSocket 協議，端點 `ws://127.0.0.1:18789/ws`。

完整握手流程：`connect` -> `challenge` -> `auth` -> `hello-ok`，之後是 request/response 搭配 streaming agent events。

```bash
rustclaw gateway    # 啟動伺服器
rustclaw health     # HTTP 健康檢查
rustclaw status     # WebSocket 握手測試
```

### Channels

#### Telegram

透過 teloxide 長輪詢。以漸進式訊息編輯模擬 streaming 回應。

```toml
[channels.telegram]
enabled = true
bot_token = "123456:ABC-..."
allowed_user_ids = []    # 空 = 允許所有人
stream_edit = true       # 即時編輯模擬串流
```

#### Discord

基於 Serenity 的 bot，回應 @mention 和私訊，內建伺服器管理工具。

```toml
[channels.discord]
enabled = true
bot_token = "your-token"
allowed_guild_ids = []   # 空 = 所有伺服器
mention_only = true      # 只回應 @mention 和私訊
```

設定方式：[Discord Developer Portal](https://discord.com/developers/applications) -> 建立應用程式 -> Bot 分頁 -> 開啟 **MESSAGE CONTENT INTENT** -> 使用 `permissions=274877975552&scope=bot` 邀請。

### Tool Calling（Agentic Loop）

Agent 可自主使用工具。支援 Anthropic 和 OpenAI function calling 格式。每次請求最多 10 輪 tool 迭代。

**內建工具：**

| Tool | 說明 |
|---|---|
| `read_file` | 讀取檔案內容（> 100KB 自動截斷）|
| `write_file` | 寫入/建立檔案（自動建立目錄）|
| `patch_file` | 檔案內的尋找與替換 |
| `list_dir` | 目錄樹（限制深度）|
| `run_command` | Shell 執行（限制在 workspace 內，有 timeout）|
| `search_code` | 類 grep 程式碼搜尋（純 Rust）|
| `discord_create_channel` | 建立 text/voice/category 頻道 |
| `discord_delete_channel` | 刪除頻道 |
| `discord_create_role` | 建立角色（含顏色）|
| `discord_set_channel_topic` | 設定頻道主題 |
| `discord_kick_member` | 踢出成員 |
| `discord_ban_member` | 封禁成員 |

```toml
[tools]
enabled = true
workspace_dir = "."
allow_exec = true
exec_timeout_secs = 30
```

### GitHub 整合

掃描 repo，用 LLM 分析 issue 並自動產生 PR。

```toml
[github]
token = "ghp_..."
repos = ["owner/repo"]
notify_discord_channel = "123456789"
```

```bash
rustclaw github scan          # 掃描所有設定的 repo
rustclaw github fix 123       # 為 issue #123 自動產生 PR
```

**Auto-PR 流程：** 取得 issue -> LLM 產生修正 -> 建立分支 `rustclaw/fix-issue-N` -> commit -> 開 PR。

Token 需要 `repo` scope（純公開 repo 用 `public_repo` 即可）。

### Cron 排程

```toml
[cron]
github_scan = "0 0 9 * * *"  # 每天 09:00（秒 分 時 日 月 週）
```

掃描結果會記錄到 log，也可選擇發送到 Discord 頻道。

---

## Discord 指令

@mention bot 時可用：

```
@RustClaw scan                    # GitHub repo 掃描報告
@RustClaw fix issue #42           # 為 issue 42 自動產生 PR
@RustClaw pr status               # 列出 bot 建立的 PR
@RustClaw 讀取 src/main.rs 並摘要
@RustClaw 建立一個叫 announcements 的頻道
@RustClaw 這個專案有哪些檔案？
@RustClaw 跑 cargo test 告訴我哪裡失敗
```

---

## CLI

```
rustclaw [OPTIONS] <COMMAND>

Commands:
  gateway              啟動 gateway + 所有啟用的 channels + cron
  agent <MESSAGE>      傳送訊息給 agent（含 tool 存取）
  health               對本地 gateway 做 HTTP 健康檢查
  status               WebSocket 狀態檢查
  github scan          掃描設定的 repo
  github fix <N>       為 issue N 自動產生 PR

Options:
  -c, --config <PATH>  設定檔路徑
  -h, --help           顯示說明
  -V, --version        顯示版本
```

---

## 架構

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

---

## 擴充指南

### 新增工具

1. 在 `src/tools/` 實作你的函式
2. 在 `src/tools/executor.rs` 的 `execute_inner()` 加入 match arm
3. 在 `tool_definitions()` 加入 JSON Schema
4. 完成。LLM 會自動發現並使用它。

### 新增 Channel

1. 建立 `src/channels/my_channel.rs`
2. 實作一個 struct 帶 `pub async fn start(self, runner: Arc<AgentRunner>) -> Result<()>`
3. 在 `src/config.rs` 的 `ChannelsConfig` 加入設定
4. 在 `src/main.rs` 的 `cmd_gateway()` 中 spawn 它

---

## Roadmap

- [ ] **MCP client** — Model Context Protocol 支援外部 tool server
- [ ] **Web UI** — 輕量瀏覽器面板，管理 session 和 log
- [ ] **Slack channel** — Slack bot 整合
- [ ] **LINE channel** — LINE Messaging API
- [ ] **持久化 session** — SQLite 儲存對話歷史
- [ ] **多 agent** — 不同 channel 使用不同 model / prompt
- [ ] **Plugin 系統** — 透過 WASM 或 shared lib 動態載入 tool
- [ ] **Metrics** — Prometheus `/metrics` 端點

歡迎社群貢獻，開 issue 或 PR 即可。

---

## License

MIT
