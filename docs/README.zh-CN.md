🌐 [English](../README.md) · [繁體中文](README.zh-TW.md) · [日本語](README.ja.md) · [한국어](README.ko.md) · [Español](README.es.md) · [Português](README.pt.md)

# RustClaw

**OpenClaw 的 Rust 精简替代品。** 单一 binary。无需 runtime。完整 agent 能力。

|                   | **RustClaw**       | **OpenClaw**              |
|-------------------|--------------------|---------------------------|
| Binary / Runtime  | **6 MB** 静态链接   | 需要 Node.js 24 + npm     |
| 空闲内存 (RSS)     | **7.9 MB**         | 1 GB+                     |
| 启动时间           | **< 100 ms**       | 5-10 秒                   |
| 代码行数           | **~4,000**         | ~430,000                  |
| 依赖管理           | 编译时打包          | npm install...             |

---

## 为什么

OpenClaw 功能很多，但对大多数场景来说太重了。如果你只需要一个能对接 Telegram、Discord、GitHub 的 LLM agent——有 tool 调用能力和 WebSocket 控制平面——你不需要 43 万行 TypeScript 和 1GB 的内存开销。

RustClaw 是 80/20 法则的产物：把真正重要的功能装进一个 `cargo build`。

完全使用 [Claude Code](https://claude.ai/code) 构建，零人工编写代码。

---

## 快速开始

### 前置要求

- Rust 工具链（`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`）
- LLM 后端：[Ollama](https://ollama.com)（本地）或 [Anthropic API key](https://console.anthropic.com)

### 编译

```bash
git clone https://github.com/Adaimade/RustClaw.git && cd RustClaw
cargo build --release
# Binary: target/release/rustclaw (6 MB)
```

### 配置

```bash
mkdir -p ~/.rustclaw
cp config.example.toml ~/.rustclaw/config.toml
```

Ollama 最小配置：

```toml
[agent]
provider = "openai"
api_key = "ollama"
base_url = "http://127.0.0.1:11434"
model = "qwen2.5:32b"
system_prompt = "You are a coding assistant with tool access."
```

Anthropic 配置：

```toml
[agent]
provider = "anthropic"
api_key = "sk-ant-..."
model = "claude-sonnet-4-20250514"
```

### 启动

```bash
# 启动所有服务（gateway + channels + cron）
rustclaw gateway

# 或直接和 agent 对话
rustclaw agent "列出 src/ 目录下的所有 .rs 文件"
```

---

## 功能

### Gateway（WebSocket 控制平面）

兼容 OpenClaw 的 WebSocket 协议，端点 `ws://127.0.0.1:18789/ws`。

完整握手流程：`connect` -> `challenge` -> `auth` -> `hello-ok`，之后是 request/response 配合 streaming agent events。

```bash
rustclaw gateway    # 启动服务器
rustclaw health     # HTTP 健康检查
rustclaw status     # WebSocket 握手测试
```

### Channels

#### Telegram

通过 teloxide 长轮询。以渐进式消息编辑模拟 streaming 响应。

```toml
[channels.telegram]
enabled = true
bot_token = "123456:ABC-..."
allowed_user_ids = []    # 空 = 允许所有人
stream_edit = true       # 即时编辑模拟流式输出
```

#### Discord

基于 Serenity 的 bot，响应 @mention 和私信，内置服务器管理工具。

```toml
[channels.discord]
enabled = true
bot_token = "your-token"
allowed_guild_ids = []   # 空 = 所有服务器
mention_only = true      # 只响应 @mention 和私信
```

设置方式：[Discord Developer Portal](https://discord.com/developers/applications) -> 创建应用 -> Bot 标签 -> 开启 **MESSAGE CONTENT INTENT** -> 用 `permissions=274877975552&scope=bot` 邀请。

### Tool Calling（Agentic Loop）

Agent 可自主使用工具。支持 Anthropic 和 OpenAI function calling 格式。每次请求最多 10 轮 tool 迭代。

**内置工具：**

| Tool | 说明 |
|---|---|
| `read_file` | 读取文件内容（> 100KB 自动截断）|
| `write_file` | 写入/创建文件（自动创建目录）|
| `patch_file` | 文件内查找与替换 |
| `list_dir` | 目录树（限制深度）|
| `run_command` | Shell 执行（限制在 workspace 内，有超时）|
| `search_code` | 类 grep 代码搜索（纯 Rust）|
| `discord_create_channel` | 创建 text/voice/category 频道 |
| `discord_delete_channel` | 删除频道 |
| `discord_create_role` | 创建角色（含颜色）|
| `discord_set_channel_topic` | 设置频道主题 |
| `discord_kick_member` | 踢出成员 |
| `discord_ban_member` | 封禁成员 |

```toml
[tools]
enabled = true
workspace_dir = "."
allow_exec = true
exec_timeout_secs = 30
```

### GitHub 集成

扫描 repo，用 LLM 分析 issue 并自动生成 PR。

```bash
rustclaw github scan          # 扫描所有配置的 repo
rustclaw github fix 123       # 为 issue #123 自动生成 PR
```

**Auto-PR 流程：** 获取 issue -> LLM 生成修复 -> 创建分支 `rustclaw/fix-issue-N` -> commit -> 开 PR。

### Cron 调度

```toml
[cron]
github_scan = "0 0 9 * * *"  # 每天 09:00
```

---

## Discord 指令

@mention bot 时可用：

```
@RustClaw scan                    # GitHub repo 扫描报告
@RustClaw fix issue #42           # 为 issue 42 自动生成 PR
@RustClaw pr status               # 列出 bot 创建的 PR
@RustClaw 读取 src/main.rs 并总结
@RustClaw 创建一个叫 announcements 的频道
@RustClaw 跑 cargo test 告诉我哪里失败
```

---

## CLI

```
rustclaw [OPTIONS] <COMMAND>

Commands:
  gateway              启动 gateway + 所有启用的 channels + cron
  agent <MESSAGE>      发送消息给 agent（含 tool 访问）
  health               HTTP 健康检查
  status               WebSocket 状态检查
  github scan          扫描配置的 repo
  github fix <N>       为 issue N 自动生成 PR

Options:
  -c, --config <PATH>  配置文件路径
  -h, --help           显示帮助
  -V, --version        显示版本
```

---

## 架构

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

## Roadmap

- [ ] **MCP client** — Model Context Protocol 支持外部 tool server
- [ ] **Web UI** — 轻量浏览器面板
- [ ] **Slack channel** — Slack bot 集成
- [ ] **LINE channel** — LINE Messaging API
- [ ] **持久化 session** — SQLite 存储
- [ ] **多 agent** — 不同 channel 使用不同 model / prompt
- [ ] **Plugin 系统** — WASM 或 shared lib 动态加载
- [ ] **Metrics** — Prometheus `/metrics` 端点

欢迎社区贡献。

---

## License

MIT
