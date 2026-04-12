<div align="center">

# RustClaw

### Rust 编写的 AI Agent 框架

**[OpenClaw](https://github.com/nicepkg/OpenClaw) 的精简替代品。**<br>
**单一 binary。22 个工具。三层记忆。Telegram + Discord + MCP。**

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](../LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.85+-orange.svg)](https://www.rust-lang.org/)
[![Built with Claude Code](https://img.shields.io/badge/Built%20with-Claude%20Code-blueviolet)](https://claude.ai)

**7.5 MB binary** · **14 MB 内存** · **5,296 行** · **98.9% BFCL** · **95.5% T-Eval** · **4.3× MoE 加速**

[快速开始](#-快速开始) · [功能](#-功能) · [Benchmark](#-benchmark) · [架构](#-架构) · [Roadmap](#-roadmap)

🌐 [English](../README.md) · [繁體中文](README.zh-TW.md) · [한국어](README.ko.md)

</div>

---

## 为什么做 RustClaw？

起点很单纯:有人把 OpenClaw 用 Go 重写,把内存从 1GB+ 砍到 35MB。很厉害。但我们想——能不能再进一步?

大多数人不需要 43 万行 TypeScript。他们需要的是一个能讲 Telegram、读文件、跑程序、出包的时候开 GitHub PR 的 agent。就这样。

RustClaw 是 OpenClaw 的 80/20 版本——把真正重要的功能装进一个 `cargo build`。

<table>
<tr><td></td><td><strong>RustClaw</strong></td><td><strong>OpenClaw</strong></td></tr>
<tr><td>📦 Binary</td><td><strong>7.5 MB</strong> 静态</td><td>需要 Node.js 24 + npm</td></tr>
<tr><td>💾 空闲内存</td><td><strong>14 MB</strong></td><td>1 GB+</td></tr>
<tr><td>⚡ 启动</td><td><strong>&lt; 100 ms</strong></td><td>5–10 秒</td></tr>
<tr><td>📝 代码</td><td><strong>5,296 行</strong></td><td>~430,000 行</td></tr>
<tr><td>🧠 记忆</td><td>三层(向量 + 图谱 + 历史)</td><td>基本 session</td></tr>
<tr><td>🔧 工具</td><td>22 个内置 + MCP</td><td>插件系统</td></tr>
<tr><td>🤖 LLM</td><td>Anthropic、OpenAI、Ollama、Gemini</td><td>OpenAI</td></tr>
<tr><td>📱 通道</td><td>Telegram、Discord、WebSocket</td><td>Web UI</td></tr>
</table>

> [!NOTE]
> RustClaw 不是要取代 OpenClaw。它证明的是——AI agent 真正有用的核心,不需要一 GB 的内存。需要的是好的架构、对的语言,以及愿意用更清晰的约束重新开始的决心。

完全使用 [Claude Code](https://claude.ai/code) 由 [Ad Huang](https://github.com/Adaimade) 构建。零人工撰写代码。

---

## 💡 核心优势

**🪶 任何地方都能跑** — 7.5 MB binary、14 MB 内存。树莓派、5 美元 VPS、你的笔记本。不需要 Node.js、Python、Docker。

**🧠 什么都记得** — 三层记忆(向量 + 图谱 + 历史),混合范围 scoping。你在 Telegram 告诉 bot 你的名字,它在 Discord 会记得。事实自动抽取、矛盾自动解决。

**🛡️ 安全为先** — 14 种危险指令模式拦截。工具输出截断。Patch 文件修改前先验证。错误自动重试恢复。120 秒 timeout 带优雅 fallback。

**🔧 真的会做事** — 业界标准 BFCL benchmark 1,000 题达 98.9%。Bot 真的会读你的文件、跑你的命令、开 PR——不是只描述它「会」做什么。

**🔌 支持 MCP** — 连接任何 MCP server。工具自动发现、透明路由。LLM 看到的是统一的工具列表——本地跟远程没差别。

**📈 经过 benchmark 验证** — 1,000 题 BFCL + 2,146 题 T-Eval + 500 题内部 benchmark。双模型策略：MoE 快速（2.6s/题）、Dense 精确（99.7%）。

**⚙️ 受 Claude Code 启发** — 理解优先的工具排序、历史压缩、workspace context 加载、错误重试提示。让 Claude Code 有效的同样模式,套用到开源 agent 上。

---

## 🚀 快速开始

### 一键安装（推荐）

**macOS / Linux：**
```bash
curl -sSL https://raw.githubusercontent.com/Adaimade/RustClaw/main/install.sh | sh
```

**Windows（PowerShell）：**
```powershell
irm https://raw.githubusercontent.com/Adaimade/RustClaw/main/install.ps1 | iex
```

自动下载 pre-built binary、加入 PATH、创建默认 config。支持 macOS（Intel / Apple Silicon）、Linux（x86 / ARM）、Windows。

### 从源码构建

| 需求 | 安装 |
|---|---|
| Rust 1.85+ | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` |
| LLM 后端 | [Ollama](https://ollama.com)、[OpenAI](https://platform.openai.com)、[Anthropic](https://console.anthropic.com) 或 [Gemini](https://ai.google.dev) |

```bash
git clone https://github.com/Adaimade/RustClaw.git && cd RustClaw
cargo build --release && strip target/release/rustclaw
# → target/release/rustclaw (7.5 MB)
```

### 配置

```bash
mkdir -p ~/.rustclaw
cp config.example.toml ~/.rustclaw/config.toml
```

<table>
<tr>
<td><strong>Ollama(本地)</strong></td>
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

> **安全提醒:** RustClaw 默认绑定 `0.0.0.0` 方便云端部署。永远不要把 API key 写在代码里——用 `~/.rustclaw/config.toml`(已 gitignored)或环境变量(`RUSTCLAW__AGENT__API_KEY`)。

### 运行

```bash
# 启动全部(gateway + 通道 + cron + 记忆)
rustclaw gateway

# 单次 agent 调用,含工具访问
rustclaw agent "列出所有 .rs 文件并计算总行数"

# GitHub 操作
rustclaw github scan
rustclaw github fix 123
```

---

## ✨ 功能

### 🔧 工具调用(Agentic Loop)

22 个内置工具自主执行。支持 Anthropic 与 OpenAI function calling。每个请求最多 10 轮迭代。

**分层工具加载**——先理解、再动手、再检查:

```
👁️ 理解                    ⚡ 动手                  🔍 检查
├── read_file              ├── run_command           ├── process_check
├── list_dir               ├── write_file            ├── docker_status
└── search_code            └── patch_file            ├── system_stats
                                                     ├── http_ping
💬 Discord(按需)          📧 Email(按需)           ├── pm2_status
├── 创建/删除频道          ├── fetch_inbox           └── process_list
├── 创建角色/设置主题      ├── read_email
└── 踢人/封禁              └── send_email
```

**安全:** 14 种危险模式拦截 · 输出截断 4000 字 · patch 验证 · 错误重试提示 · 120 秒优雅 timeout

### 🧠 三层记忆

记忆系统委托给 [**R-Mem**](https://github.com/Adaimade/R-Mem)——一个独立的 Rust crate,负责向量召回、事实抽取、矛盾解决、实体关系图谱。RustClaw 只是上层的轻量包装,加上混合范围 scoping。

**混合范围召回** — 每次查询合并三种范围:

| 范围 | 示例 | 共享于 |
|---|---|---|
| Local | `telegram:-100xxx` | 单一群组 |
| User | `user:12345` | 一个人在所有通道 |
| Global | `global:system` | 所有人 |

### 📱 通道

| 通道 | 功能 |
|---|---|
| **Telegram** | 长轮询 · streaming 编辑 · ACL · session 历史 |
| **Discord** | @mention · 服务器管理 · `scan` / `fix issue #N` / `pr status` |
| **Gateway** | OpenClaw 兼容 WebSocket,位于 `:18789/ws` |

### 🔌 MCP Client

```toml
[mcp]
servers = [
  { name = "fs", command = "npx @modelcontextprotocol/server-filesystem /tmp" },
]
```

### 🐙 GitHub · ⏰ Cron · 📧 Email

自动扫描 repo · 从 issue 自动 PR · 系统监控告警 · email 分类——全部通过 cron 排程,通知到 Discord。

---

## 📊 Benchmark

### Berkeley Function Calling Leaderboard (BFCL)

在**官方 [Gorilla BFCL](https://github.com/ShishirPatil/gorilla)** benchmark 上测试——业界 function calling 评估的标杆。双模型比较（Mac Mini 2024, M4 Pro, 64 GB）：

| 测试 | qwen3-coder:30b (MoE) | qwen2.5:32b (dense) | 加速 |
|---|---|---|---|
| **simple_python** (400) | **100%** · 1.5s/题 | 99.75% · 7.3s/题 | 4.9× |
| **multiple** (200) | 97% · 2.4s/题 | **99.5%** · 8.4s/题 | 3.5× |
| **parallel** (200) | 99.5% · 2.9s/题 | **100%** · 12.0s/题 | 4.1× |
| **parallel_multiple** (200) | 98% · 3.4s/题 | **100%** · 15.7s/题 | 4.6× |
| **Overall** (1,000) | **98.9%** · 2.6s/题 | **99.7%** · 10.8s/题 | **4.3×** |

> MoE 模型以 -0.8% 准确度换取 4.3× 加速。两个模型在所有类别均超过 98%。

### T-Eval(上海 AI Lab)

在 **[T-Eval](https://github.com/open-compass/T-Eval)** 上测试——上海 AI Lab 的工具使用评估套件,涵盖规划、检索、检查与指令跟随:

| 测试 | 分数 | 题数 | 速度 |
|---|---|---|---|
| **T-Eval retrieve** | **98%** (542/553) | 553 | 14.5s/题 |
| **T-Eval plan** | **96%** (535/553) | 553 | 25.6s/题 |
| **T-Eval review** | **96%** (472/487) | 487 | 3.5s/题 |
| **T-Eval instruct** | **92%** (514/553) | 553 | 8.2s/题 |

> 四个核心类别共 2,146 题。平均 **95.5%** —— 工具选择、多步规划、自我检查皆强。

### 内部 Benchmark

500 题工具调用 benchmark(qwen2.5:32b、本地 Ollama):

| 版本 | 总分 | Timeout | 速度 |
|---|---|---|---|
| v3 baseline | 81% | 74 | 44s/题 |
| v4 timeout fix | 85% | 3 | 36s/题 |
| **v5 optimized** | **97%** | **0** | **38s/题** |

| 类别 | v5 分数 |
|---|---|
| 核心操作 | 92% |
| 基本工具 | 95% |
| 中等任务 | **100%** |
| 进阶推理 | 98% |
| 幻觉陷阱 | **100%** |
| 多步骤连锁 | 99% |

> Benchmark 题目于 [AI-Bench](https://github.com/Adaimade/AI-Bench)。

---

## 🏗️ 架构

```
src/
├── main.rs              CLI dispatch + 启动
├── cli/mod.rs           clap subcommands
├── config.rs            TOML + env 配置
├── gateway/             WebSocket server + 协议 + handshake
├── agent/runner.rs      LLM streaming + agentic loop + 历史压缩
├── channels/            Telegram (teloxide) + Discord (serenity)
├── tools/               22 个工具:fs、shell、search、discord、email、system、github、mcp
├── session/             SessionStore(历史) + MemoryManager(R-Mem 包装)
└── cron/                排程任务(system、email、GitHub)
```

**27 个文件 · 5,296 行 · 7.5 MB binary · 零外部服务**

---

## 🗺️ Roadmap

| 状态 | 功能 |
|---|---|
| ✅ | 工具调用(22 个工具 + agentic loop) |
| ✅ | 三层记忆(向量 + 图谱 + 混合范围) |
| ✅ | Telegram + Discord 通道 |
| ✅ | MCP client(透明工具路由) |
| ✅ | GitHub 集成(扫描 + 自动 PR) |
| ✅ | 系统监控 + cron 告警 |
| ✅ | Email(IMAP + SMTP) |
| ✅ | SQLite 持久化 |
| ✅ | 跨平台安装（macOS / Linux / Windows） |
| ✅ | 多模型路由（per-channel config 驱动） |
| 🔲 | Slack / LINE 通道 |
| 🔲 | Prometheus metrics |

欢迎社区贡献——开 issue 或 PR。

---

<div align="center">

**MIT License** · v0.5.0

由 [Ad Huang](https://github.com/Adaimade) 使用 [Claude Code](https://claude.ai) 创建

*框架在这里。剩下的交给社区。*

</div>
