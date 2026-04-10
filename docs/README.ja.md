<div align="center">

# RustClaw

### Rust で書かれた AI エージェントフレームワーク

**[OpenClaw](https://github.com/nicepkg/OpenClaw) の軽量な代替。**<br>
**シングルバイナリ。22 個のツール。3 層メモリ。Telegram + Discord + MCP。**

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](../LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.85+-orange.svg)](https://www.rust-lang.org/)
[![Built with Claude Code](https://img.shields.io/badge/Built%20with-Claude%20Code-blueviolet)](https://claude.ai)

**7.5 MB バイナリ** · **14 MB メモリ** · **5,918 行** · **99.7% BFCL** · **95.5% T-Eval** · **0% ハルシネーション**

[クイックスタート](#-クイックスタート) · [機能](#-機能) · [ベンチマーク](#-ベンチマーク) · [アーキテクチャ](#-アーキテクチャ) · [ロードマップ](#-ロードマップ)

🌐 [English](../README.md) · [繁體中文](README.zh-TW.md) · [简体中文](README.zh-CN.md) · [한국어](README.ko.md) · [Español](README.es.md) · [Português](README.pt.md)

</div>

---

## なぜ RustClaw を作ったのか？

出発点はとてもシンプルです。誰かが OpenClaw を Go で書き直し、メモリ使用量を 1GB+ から 35MB にまで削減しました。素晴らしい成果です。しかし私たちは考えました——さらに先に進めないだろうか？

ほとんどの人は 43 万行の TypeScript を必要としていません。必要なのは、Telegram で会話し、ファイルを読み、コードを実行し、何かが壊れたら GitHub で PR を開いてくれるエージェントです。それだけです。

RustClaw は OpenClaw の 80/20 版——本当に重要な機能を 1 回の `cargo build` に詰め込んだものです。

<table>
<tr><td></td><td><strong>RustClaw</strong></td><td><strong>OpenClaw</strong></td></tr>
<tr><td>📦 バイナリ</td><td><strong>7.5 MB</strong> 静的</td><td>Node.js 24 + npm が必要</td></tr>
<tr><td>💾 アイドル時メモリ</td><td><strong>14 MB</strong></td><td>1 GB+</td></tr>
<tr><td>⚡ 起動時間</td><td><strong>&lt; 100 ms</strong></td><td>5–10 秒</td></tr>
<tr><td>📝 コード</td><td><strong>5,918 行</strong></td><td>約 430,000 行</td></tr>
<tr><td>🧠 メモリ</td><td>3 層（ベクトル + グラフ + 履歴）</td><td>基本的なセッション</td></tr>
<tr><td>🔧 ツール</td><td>22 個の組み込み + MCP</td><td>プラグインシステム</td></tr>
<tr><td>🤖 LLM</td><td>Anthropic、OpenAI、Ollama、Gemini</td><td>OpenAI</td></tr>
<tr><td>📱 チャネル</td><td>Telegram、Discord、WebSocket</td><td>Web UI</td></tr>
</table>

> [!NOTE]
> RustClaw は OpenClaw を置き換えようとしているわけではありません。AI エージェントを本当に有用にするコアは、1 ギガバイトのメモリを必要としないことを証明しようとしているのです。必要なのは、優れたアーキテクチャ、適切な言語、そしてより明確な制約でやり直す意志です。

完全に [Claude Code](https://claude.ai/code) を使用して [Ad Huang](https://github.com/Adaimade) によって構築されました。人間が書いたコードはゼロです。

---

## 💡 主な利点

**🪶 どこでも動く** — 7.5 MB バイナリ、14 MB メモリ。Raspberry Pi、5 ドルの VPS、あなたのノート PC。Node.js も Python も Docker も不要です。

**🧠 すべてを記憶する** — 3 層メモリ（ベクトル + グラフ + 履歴）とミックスモードスコープ。Telegram でボットに名前を伝えれば、Discord でもそれを覚えています。事実は自動抽出され、矛盾は自動解決されます。

**🛡️ 設計から安全** — 14 種類の危険なコマンドパターンをブロック。ツール出力を切り詰め。パッチファイルは変更前に検証。エラー時は自動リカバリ付きリトライ。120 秒タイムアウトと優雅なフォールバック。

**🔧 実際に行動する** — 500 問のベンチマークでツール呼び出し精度 97%。ハルシネーション率 0%。ボットは本当にファイルを読み、コマンドを実行し、PR を作成します——「やるつもり」を説明するだけではありません。

**🔌 MCP 対応** — 任意の MCP サーバーに接続。ツールは自動検出され、透過的にルーティングされます。LLM には統一されたツールリストが見えます——ローカルとリモートの区別はありません。

**📈 ベンチマークで実証済み** — 日常運用、コーディング、システム管理、敵対的プロンプトをカバーする 500 問のプロフェッショナルベンチマーク。v3→v5 で改善: 81% → 97%。タイムアウトゼロ。

**⚙️ Claude Code にインスパイア** — 理解優先のツール順序、履歴圧縮、ワークスペースコンテキスト読み込み、エラーリトライヒント。Claude Code を効果的にしているのと同じパターンを、オープンソースエージェントに適用しました。

---

## 🚀 クイックスタート

### 前提条件

| 要件 | インストール |
|---|---|
| Rust 1.85+ | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` |
| LLM バックエンド | [Ollama](https://ollama.com)、[OpenAI](https://platform.openai.com)、[Anthropic](https://console.anthropic.com)、または [Gemini](https://ai.google.dev) |

### ビルドと実行

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
<td><strong>Ollama（ローカル）</strong></td>
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

> **セキュリティ注意：** RustClaw はクラウドデプロイの利便性のため、デフォルトで `0.0.0.0` にバインドされます。API キーを決してコードに書かないでください——`~/.rustclaw/config.toml`（gitignored 済み）または環境変数（`RUSTCLAW__AGENT__API_KEY`）を使用してください。

### 実行

```bash
# すべてを起動（gateway + チャネル + cron + メモリ）
rustclaw gateway

# ツールアクセス付きの単発エージェント呼び出し
rustclaw agent "すべての .rs ファイルをリストして合計行数を数える"

# GitHub 操作
rustclaw github scan
rustclaw github fix 123
```

---

## ✨ 機能

### 🔧 ツール呼び出し（Agentic Loop）

22 個の組み込みツールが自律的に実行されます。Anthropic と OpenAI の function calling をサポート。リクエストごとに最大 10 回の反復。

**階層型ツールロード**——まず理解、次に行動、最後に確認：

```
👁️ 理解                    ⚡ 行動                  🔍 確認
├── read_file              ├── run_command           ├── process_check
├── list_dir               ├── write_file            ├── docker_status
└── search_code            └── patch_file            ├── system_stats
                                                     ├── http_ping
💬 Discord（オンデマンド） 📧 Email（オンデマンド）  ├── pm2_status
├── チャンネル作成/削除    ├── fetch_inbox           └── process_list
├── ロール作成/トピック設定├── read_email
└── キック/BAN             └── send_email
```

**安全性：** 14 種類の危険なパターンをブロック · 出力は 4000 文字に切り詰め · パッチ検証 · エラーリトライヒント · 120 秒の優雅なタイムアウト

### 🧠 3 層メモリ

[R-Mem](https://github.com/Adaimade/R-Mem) アーキテクチャによって駆動されます。

```
├─ 📝 短期 ──── 会話履歴（SQLite）
├─ 📦 長期 ──── LLM による事実抽出 → 重複排除 → ADD/UPDATE/DELETE/NONE
│    └── 整数 ID マッピング · 矛盾検出 · 意味論的重複排除
└─ 🕸️ グラフ ── ソフト削除付きエンティティ + 関係抽出
```

**ミックスモードリコール** — 3 つのスコープをマージ：

| スコープ | 例 | 共有範囲 |
|---|---|---|
| Local | `telegram:-100xxx` | 単一のグループ |
| User | `user:12345` | すべてのチャネルでの 1 人 |
| Global | `global:system` | すべての人 |

### 📱 チャネル

| チャネル | 機能 |
|---|---|
| **Telegram** | ロングポーリング · ストリーミング編集 · ACL · セッション履歴 |
| **Discord** | @mention · サーバー管理 · `scan` / `fix issue #N` / `pr status` |
| **Gateway** | OpenClaw 互換 WebSocket（`:18789/ws`） |

### 🔌 MCP Client

```toml
[mcp]
servers = [
  { name = "fs", command = "npx @modelcontextprotocol/server-filesystem /tmp" },
]
```

### 🐙 GitHub · ⏰ Cron · 📧 Email

リポジトリの自動スキャン · issue からの自動 PR · システム監視アラート · メール分類——すべて cron でスケジュールされ、Discord に通知されます。

---

## 📊 ベンチマーク

### Berkeley Function Calling Leaderboard (BFCL)

**公式 [Gorilla BFCL](https://github.com/ShishirPatil/gorilla)** ベンチマークでテスト——function calling 評価の業界標準です：

| テスト | スコア | 問題数 | 速度 |
|---|---|---|---|
| **BFCL simple_python** | **99.75%** (399/400) | 400 | 7.3s/問 |
| **BFCL multiple** | **99.5%** (199/200) | 200 | 8.4s/問 |
| **BFCL parallel** | **100%** (200/200) | 200 | 12.0s/問 |
| **BFCL parallel_multiple** | **100%** (200/200) | 200 | 15.7s/問 |

> 公式 BFCL ベンチマークの 1,000 問。並列 function calling で 2 つの満点。

### T-Eval（上海 AI Lab）

**[T-Eval](https://github.com/open-compass/T-Eval)** でテスト——上海 AI Lab のツール利用評価スイートで、計画、検索、レビュー、指示追従をカバーします：

| テスト | スコア | 問題数 | 速度 |
|---|---|---|---|
| **T-Eval retrieve** | **98%** (542/553) | 553 | 14.5s/問 |
| **T-Eval plan** | **96%** (535/553) | 553 | 25.6s/問 |
| **T-Eval review** | **96%** (472/487) | 487 | 3.5s/問 |
| **T-Eval instruct** | **92%** (514/553) | 553 | 8.2s/問 |

> 4 つのコアカテゴリにわたる 2,146 問。平均 **95.5%** ——強力なツール選択、マルチステップ計画、自己レビュー。

### 内部ベンチマーク

500 問のツール呼び出しベンチマーク（qwen2.5:32b、ローカル Ollama）：

| バージョン | 合計 | タイムアウト | 速度 |
|---|---|---|---|
| v3 baseline | 81% | 74 | 44s/問 |
| v4 timeout fix | 85% | 3 | 36s/問 |
| **v5 optimized** | **97%** | **0** | **38s/問** |

| カテゴリ | v5 スコア |
|---|---|
| コア操作 | 92% |
| 基本ツール | 95% |
| 中程度のタスク | **100%** |
| 高度な推論 | 98% |
| ハルシネーションの罠 | **100%** |
| マルチステップチェーン | 99% |

> ベンチマーク問題は [AI-Bench](https://github.com/Adaimade/AI-Bench) にあります。

---

## 🏗️ アーキテクチャ

```
src/
├── main.rs              CLI ディスパッチ + 起動
├── cli/mod.rs           clap サブコマンド
├── config.rs            TOML + env 設定
├── gateway/             WebSocket サーバー + プロトコル + ハンドシェイク
├── agent/runner.rs      LLM ストリーミング + agentic loop + 履歴圧縮
├── channels/            Telegram (teloxide) + Discord (serenity)
├── tools/               22 個のツール: fs、shell、search、discord、email、system、github、mcp
├── session/             MemoryManager + SQLite store + グラフ + embedding + 抽出
└── cron/                スケジュールされたジョブ（system、email、GitHub）
```

**30 ファイル · 5,918 行 · 7.5 MB バイナリ · 外部サービス不要**

---

## 🗺️ ロードマップ

| ステータス | 機能 |
|---|---|
| ✅ | ツール呼び出し（22 個のツール + agentic loop） |
| ✅ | 3 層メモリ（ベクトル + グラフ + ミックススコープ） |
| ✅ | Telegram + Discord チャネル |
| ✅ | MCP client（透過的なツールルーティング） |
| ✅ | GitHub 統合（スキャン + 自動 PR） |
| ✅ | システム監視 + cron アラート |
| ✅ | Email（IMAP + SMTP） |
| ✅ | SQLite 永続化 |
| 🔲 | Web UI ダッシュボード |
| 🔲 | Slack / LINE チャネル |
| 🔲 | RAG（ドキュメント検索） |
| 🔲 | マルチエージェントルーティング |
| 🔲 | WASM プラグインシステム |
| 🔲 | Prometheus メトリクス |

コミュニティからの貢献を歓迎します——issue や PR をお気軽にどうぞ。

---

<div align="center">

**MIT License** · v0.4.0

[Ad Huang](https://github.com/Adaimade) が [Claude Code](https://claude.ai) を使用して作成

*フレームワークはここにあります。あとはコミュニティ次第です。*

</div>
