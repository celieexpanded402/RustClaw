<div align="center">

# RustClaw

### Rust 寫的 AI Agent 框架

**[OpenClaw](https://github.com/nicepkg/OpenClaw) 的精簡替代品。**<br>
**單一 binary。22 個工具。三層記憶。Telegram + Discord + MCP。**

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](../LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.85+-orange.svg)](https://www.rust-lang.org/)
[![Built with Claude Code](https://img.shields.io/badge/Built%20with-Claude%20Code-blueviolet)](https://claude.ai)

**7.5 MB binary** · **14 MB 記憶體** · **5,918 行** · **99.7% BFCL** · **95.5% T-Eval** · **0% 幻覺**

[快速開始](#-快速開始) · [功能](#-功能) · [Benchmark](#-benchmark) · [架構](#-架構) · [Roadmap](#-roadmap)

🌐 [English](../README.md) · [简体中文](README.zh-CN.md) · [日本語](README.ja.md) · [한국어](README.ko.md) · [Español](README.es.md) · [Português](README.pt.md)

</div>

---

## 為什麼做 RustClaw？

起點很單純：有人把 OpenClaw 用 Go 重寫，把記憶體從 1GB+ 砍到 35MB。很厲害。但我們想——能不能再進一步？

大多數人不需要 43 萬行 TypeScript。他們需要的是一個能講 Telegram、讀檔案、跑程式、出包的時候開 GitHub PR 的 agent。就這樣。

RustClaw 是 OpenClaw 的 80/20 版本——把真正重要的功能裝進一個 `cargo build`。

<table>
<tr><td></td><td><strong>RustClaw</strong></td><td><strong>OpenClaw</strong></td></tr>
<tr><td>📦 Binary</td><td><strong>7.5 MB</strong> 靜態</td><td>需要 Node.js 24 + npm</td></tr>
<tr><td>💾 閒置記憶體</td><td><strong>14 MB</strong></td><td>1 GB+</td></tr>
<tr><td>⚡ 啟動</td><td><strong>&lt; 100 ms</strong></td><td>5–10 秒</td></tr>
<tr><td>📝 程式碼</td><td><strong>5,918 行</strong></td><td>~430,000 行</td></tr>
<tr><td>🧠 記憶</td><td>三層（向量 + 圖譜 + 歷史）</td><td>基本 session</td></tr>
<tr><td>🔧 工具</td><td>22 個內建 + MCP</td><td>外掛系統</td></tr>
<tr><td>🤖 LLM</td><td>Anthropic、OpenAI、Ollama、Gemini</td><td>OpenAI</td></tr>
<tr><td>📱 通道</td><td>Telegram、Discord、WebSocket</td><td>Web UI</td></tr>
</table>

> [!NOTE]
> RustClaw 不是要取代 OpenClaw。它證明的是——AI agent 真正有用的核心，不需要一 GB 的記憶體。需要的是好的架構、對的語言、以及願意用更清楚的限制重新開始的決心。

完全使用 [Claude Code](https://claude.ai/code) 由 [Ad Huang](https://github.com/Adaimade) 構建。零人工撰寫程式碼。

---

## 💡 核心優勢

**🪶 任何地方都能跑** — 7.5 MB binary、14 MB 記憶體。樹莓派、5 美元 VPS、你的筆電。不需要 Node.js、Python、Docker。

**🧠 什麼都記得** — 三層記憶（向量 + 圖譜 + 歷史），混合範圍 scoping。你在 Telegram 告訴 bot 你的名字，它在 Discord 會記得。事實自動萃取、矛盾自動解決。

**🛡️ 安全為先** — 14 種危險指令模式封鎖。工具輸出截斷。Patch 檔修改前先驗證。錯誤自動重試恢復。120 秒 timeout 帶優雅 fallback。

**🔧 真的會做事** — 500 題 benchmark 工具呼叫準確率 97%。零幻覺率。Bot 真的會讀你的檔案、跑你的指令、開 PR——不是只描述它「會」做什麼。

**🔌 支援 MCP** — 連接任何 MCP server。工具自動發現、透明路由。LLM 看到的是統一的工具列表——本地跟遠端沒差別。

**📈 經過 benchmark 驗證** — 500 題專業 benchmark 涵蓋日常維運、寫程式、系統管理、對抗式 prompt。v3→v5 進步：81% → 97%。零 timeout。

**⚙️ 受 Claude Code 啟發** — 理解優先的工具排序、歷史壓縮、workspace context 載入、錯誤重試提示。讓 Claude Code 有效的同樣模式，套用到開源 agent 上。

---

## 🚀 快速開始

### 前置需求

| 需求 | 安裝 |
|---|---|
| Rust 1.85+ | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` |
| LLM 後端 | [Ollama](https://ollama.com)、[OpenAI](https://platform.openai.com)、[Anthropic](https://console.anthropic.com) 或 [Gemini](https://ai.google.dev) |

### 建置與執行

```bash
git clone https://github.com/Adaimade/RustClaw.git && cd RustClaw
cargo build --release
# → target/release/rustclaw (7.5 MB)
```

### 設定

```bash
mkdir -p ~/.rustclaw
cp config.example.toml ~/.rustclaw/config.toml
```

<table>
<tr>
<td><strong>Ollama（本地）</strong></td>
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

> **安全提醒：** RustClaw 預設綁定 `0.0.0.0` 方便雲端部署。永遠不要把 API key 寫在程式碼裡——用 `~/.rustclaw/config.toml`（已 gitignored）或環境變數（`RUSTCLAW__AGENT__API_KEY`）。

### 執行

```bash
# 啟動全部（gateway + 通道 + cron + 記憶）
rustclaw gateway

# 單次 agent 呼叫，含工具存取
rustclaw agent "列出所有 .rs 檔案並計算總行數"

# GitHub 操作
rustclaw github scan
rustclaw github fix 123
```

---

## ✨ 功能

### 🔧 工具呼叫（Agentic Loop）

22 個內建工具自主執行。支援 Anthropic 與 OpenAI function calling。每個請求最多 10 輪迭代。

**分層工具載入**——先理解、再動手、再檢查：

```
👁️ 理解                    ⚡ 動手                  🔍 檢查
├── read_file              ├── run_command           ├── process_check
├── list_dir               ├── write_file            ├── docker_status
└── search_code            └── patch_file            ├── system_stats
                                                     ├── http_ping
💬 Discord（按需）         📧 Email（按需）          ├── pm2_status
├── 建立/刪除頻道          ├── fetch_inbox           └── process_list
├── 建立角色/設定主題      ├── read_email
└── 踢人/封鎖              └── send_email
```

**安全：** 14 種危險模式封鎖 · 輸出截斷 4000 字 · patch 驗證 · 錯誤重試提示 · 120 秒優雅 timeout

### 🧠 三層記憶

由 [R-Mem](https://github.com/Adaimade/R-Mem) 架構驅動。

```
├─ 📝 短期 ──── 對話歷史（SQLite）
├─ 📦 長期 ──── LLM 事實萃取 → 去重 → ADD/UPDATE/DELETE/NONE
│    └── 整數 ID 對應 · 矛盾偵測 · 語意去重
└─ 🕸️ 圖譜 ─── 實體 + 關係萃取，含軟刪除
```

**混合範圍取回** — 三種範圍合併：

| 範圍 | 範例 | 共享於 |
|---|---|---|
| Local | `telegram:-100xxx` | 單一群組 |
| User | `user:12345` | 一個人在所有通道 |
| Global | `global:system` | 所有人 |

### 📱 通道

| 通道 | 功能 |
|---|---|
| **Telegram** | 長輪詢 · streaming 編輯 · ACL · session 歷史 |
| **Discord** | @mention · 伺服器管理 · `scan` / `fix issue #N` / `pr status` |
| **Gateway** | OpenClaw 相容 WebSocket，位於 `:18789/ws` |

### 🔌 MCP Client

```toml
[mcp]
servers = [
  { name = "fs", command = "npx @modelcontextprotocol/server-filesystem /tmp" },
]
```

### 🐙 GitHub · ⏰ Cron · 📧 Email

自動掃描 repo · 從 issue 自動 PR · 系統監控告警 · email 分類——全部透過 cron 排程，通知到 Discord。

---

## 📊 Benchmark

### Berkeley Function Calling Leaderboard (BFCL)

在**官方 [Gorilla BFCL](https://github.com/ShishirPatil/gorilla)** benchmark 上測試——業界 function calling 評估的標竿：

| 測試 | 分數 | 題數 | 速度 |
|---|---|---|---|
| **BFCL simple_python** | **99.75%** (399/400) | 400 | 7.3s/題 |
| **BFCL multiple** | **99.5%** (199/200) | 200 | 8.4s/題 |
| **BFCL parallel** | **100%** (200/200) | 200 | 12.0s/題 |
| **BFCL parallel_multiple** | **100%** (200/200) | 200 | 15.7s/題 |

> 官方 BFCL 1,000 題。平行 function calling 兩個滿分。

### T-Eval（上海 AI Lab）

在 **[T-Eval](https://github.com/open-compass/T-Eval)** 上測試——上海 AI Lab 的工具使用評估套件，涵蓋規劃、檢索、檢查與指令跟隨：

| 測試 | 分數 | 題數 | 速度 |
|---|---|---|---|
| **T-Eval retrieve** | **98%** (542/553) | 553 | 14.5s/題 |
| **T-Eval plan** | **96%** (535/553) | 553 | 25.6s/題 |
| **T-Eval review** | **96%** (472/487) | 487 | 3.5s/題 |
| **T-Eval instruct** | **92%** (514/553) | 553 | 8.2s/題 |

> 四個核心類別共 2,146 題。平均 **95.5%** —— 工具選擇、多步規劃、自我檢查皆強。

### 內部 Benchmark

500 題工具呼叫 benchmark（qwen2.5:32b、本地 Ollama）：

| 版本 | 總分 | Timeout | 速度 |
|---|---|---|---|
| v3 baseline | 81% | 74 | 44s/題 |
| v4 timeout fix | 85% | 3 | 36s/題 |
| **v5 optimized** | **97%** | **0** | **38s/題** |

| 類別 | v5 分數 |
|---|---|
| 核心操作 | 92% |
| 基本工具 | 95% |
| 中等任務 | **100%** |
| 進階推理 | 98% |
| 幻覺陷阱 | **100%** |
| 多步驟連鎖 | 99% |

> Benchmark 題目於 [AI-Bench](https://github.com/Adaimade/AI-Bench)。

---

## 🏗️ 架構

```
src/
├── main.rs              CLI dispatch + 啟動
├── cli/mod.rs           clap subcommands
├── config.rs            TOML + env 設定
├── gateway/             WebSocket server + 協定 + handshake
├── agent/runner.rs      LLM streaming + agentic loop + 歷史壓縮
├── channels/            Telegram (teloxide) + Discord (serenity)
├── tools/               22 個工具：fs、shell、search、discord、email、system、github、mcp
├── session/             MemoryManager + SQLite store + 圖譜 + embedding + 萃取
└── cron/                排程任務（system、email、GitHub）
```

**30 個檔案 · 5,918 行 · 7.5 MB binary · 零外部服務**

---

## 🗺️ Roadmap

| 狀態 | 功能 |
|---|---|
| ✅ | 工具呼叫（22 個工具 + agentic loop） |
| ✅ | 三層記憶（向量 + 圖譜 + 混合範圍） |
| ✅ | Telegram + Discord 通道 |
| ✅ | MCP client（透明工具路由） |
| ✅ | GitHub 整合（掃描 + 自動 PR） |
| ✅ | 系統監控 + cron 告警 |
| ✅ | Email（IMAP + SMTP） |
| ✅ | SQLite 持久化 |
| 🔲 | Web UI dashboard |
| 🔲 | Slack / LINE 通道 |
| 🔲 | RAG（文件搜尋） |
| 🔲 | 多 agent 路由 |
| 🔲 | WASM 外掛系統 |
| 🔲 | Prometheus metrics |

歡迎社群貢獻——開 issue 或 PR。

---

<div align="center">

**MIT License** · v0.4.0

由 [Ad Huang](https://github.com/Adaimade) 使用 [Claude Code](https://claude.ai) 創建

*框架在這裡。剩下的交給社群。*

</div>
