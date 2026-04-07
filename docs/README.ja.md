🌐 [English](../README.md) · [繁體中文](README.zh-TW.md) · [简体中文](README.zh-CN.md) · [한국어](README.ko.md) · [Español](README.es.md) · [Português](README.pt.md)

# RustClaw

**OpenClaw の軽量 Rust 代替品。** シングルバイナリ。ランタイム不要。フル agent 機能。

|                   | **RustClaw**       | **OpenClaw**              |
|-------------------|--------------------|---------------------------|
| Binary / Runtime  | **6 MB** 静的リンク | Node.js 24 + npm が必要    |
| アイドルメモリ (RSS) | **7.9 MB**        | 1 GB+                     |
| 起動時間           | **< 100 ms**       | 5-10 秒                   |
| コード行数         | **~4,000**         | ~430,000                  |
| 依存関係           | コンパイル時に同梱   | npm install...             |

---

## なぜ作ったのか

OpenClaw は多機能ですが、ほとんどのユースケースには過剰です。Telegram、Discord、GitHub と連携する LLM agent が欲しいだけなら——tool 呼び出しと WebSocket 制御プレーン付きで——43万行の TypeScript と 1GB のメモリフットプリントは不要です。

RustClaw は 80/20 の法則で作られました：本当に必要な機能だけを、一つの `cargo build` に。

[Claude Code](https://claude.ai/code) で完全に構築。人間が書いたコードはゼロです。

---

## クイックスタート

### 前提条件

- Rust ツールチェーン（`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`）
- LLM バックエンド：[Ollama](https://ollama.com)（ローカル）または [Anthropic API key](https://console.anthropic.com)

### ビルド

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

Ollama の最小設定：

```toml
[agent]
provider = "openai"
api_key = "ollama"
base_url = "http://127.0.0.1:11434"
model = "qwen2.5:32b"
system_prompt = "You are a coding assistant with tool access."
```

Anthropic の設定：

```toml
[agent]
provider = "anthropic"
api_key = "sk-ant-..."
model = "claude-sonnet-4-20250514"
```

### 起動

```bash
# すべて起動（gateway + channels + cron）
rustclaw gateway

# 直接 agent と対話
rustclaw agent "src/ 配下のすべての .rs ファイルを一覧表示して"
```

---

## 機能

### Gateway（WebSocket 制御プレーン）

OpenClaw 互換の WebSocket プロトコル（`ws://127.0.0.1:18789/ws`）。

完全なハンドシェイク：`connect` -> `challenge` -> `auth` -> `hello-ok`、その後 streaming agent events による request/response。

### Channels

#### Telegram

teloxide による long polling。メッセージ編集による streaming レスポンスのシミュレーション。

#### Discord

Serenity ベースの bot。@mention と DM に応答。サーバー管理ツール内蔵。

### Tool Calling（Agentic Loop）

Agent はツールを自律的に使用します。Anthropic と OpenAI の function calling 形式の両方に対応。リクエストあたり最大 10 ラウンドの tool イテレーション。

**組み込みツール：**

| Tool | 説明 |
|---|---|
| `read_file` | ファイル読み取り（100KB超は自動的に切り詰め）|
| `write_file` | ファイル書き込み・作成（ディレクトリ自動作成）|
| `patch_file` | ファイル内の検索・置換 |
| `list_dir` | ディレクトリツリー（深さ制限付き）|
| `run_command` | Shell 実行（workspace 内に制限、タイムアウト付き）|
| `search_code` | grep ライクなコード検索（純 Rust）|
| `discord_create_channel` | text/voice/category チャンネル作成 |
| `discord_delete_channel` | チャンネル削除 |
| `discord_create_role` | ロール作成（色付き）|
| `discord_set_channel_topic` | チャンネルトピック設定 |
| `discord_kick_member` | メンバーをキック |
| `discord_ban_member` | メンバーを BAN |

### GitHub 統合

リポジトリをスキャンし、LLM 分析を使って issue から PR を自動生成。

```bash
rustclaw github scan          # 設定されたすべてのリポジトリをスキャン
rustclaw github fix 123       # issue #123 の auto-PR を生成
```

### Cron スケジュール

```toml
[cron]
github_scan = "0 0 9 * * *"  # 毎日 09:00
```

---

## Discord コマンド

bot に @mention すると使えます：

```
@RustClaw scan                    # GitHub リポジトリスキャンレポート
@RustClaw fix issue #42           # issue 42 の PR を自動生成
@RustClaw pr status               # bot が作成した PR を一覧表示
@RustClaw src/main.rs を読んで要約して
@RustClaw announcements というチャンネルを作って
@RustClaw cargo test を実行して失敗箇所を教えて
```

---

## CLI

```
rustclaw [OPTIONS] <COMMAND>

Commands:
  gateway              gateway + 有効な channels + cron を起動
  agent <MESSAGE>      agent にメッセージを送信（tool アクセス付き）
  health               HTTP ヘルスチェック
  status               WebSocket ステータスチェック
  github scan          設定されたリポジトリをスキャン
  github fix <N>       issue N の PR を自動生成

Options:
  -c, --config <PATH>  設定ファイルのパス
  -h, --help           ヘルプを表示
  -V, --version        バージョンを表示
```

---

## Roadmap

- [ ] **MCP client** — 外部 tool サーバー用の Model Context Protocol 対応
- [ ] **Web UI** — セッションとログの軽量ブラウザダッシュボード
- [ ] **Slack channel** — Slack bot 統合
- [ ] **LINE channel** — LINE Messaging API
- [ ] **永続化セッション** — SQLite バックエンド
- [ ] **マルチ agent** — チャンネルごとに異なるモデル・プロンプト
- [ ] **Plugin システム** — WASM または shared lib による動的ツール読み込み
- [ ] **Metrics** — Prometheus `/metrics` エンドポイント

コミュニティの貢献を歓迎します。issue または PR をお気軽にどうぞ。

---

## License

MIT
