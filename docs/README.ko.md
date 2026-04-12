<div align="center">

# RustClaw

### Rust로 작성된 AI 에이전트 프레임워크

**[OpenClaw](https://github.com/nicepkg/OpenClaw)의 경량 대체품.**<br>
**단일 바이너리. 22개 도구. 3계층 메모리. Telegram + Discord + MCP.**

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](../LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.85+-orange.svg)](https://www.rust-lang.org/)
[![Built with Claude Code](https://img.shields.io/badge/Built%20with-Claude%20Code-blueviolet)](https://claude.ai)

**7.5 MB 바이너리** · **14 MB 메모리** · **5,296 라인** · **98.9% BFCL** · **95.5% T-Eval** · **4.3× MoE 가속**

[빠른 시작](#-빠른-시작) · [기능](#-기능) · [Benchmark](#-benchmark) · [아키텍처](#-아키텍처) · [Roadmap](#-roadmap)

🌐 [English](../README.md) · [繁體中文](README.zh-TW.md) · [简体中文](README.zh-CN.md)

</div>

---

## 왜 RustClaw인가?

시작은 단순했습니다. 누군가 OpenClaw를 Go로 다시 작성해서 메모리 사용량을 1GB+에서 35MB로 줄였습니다. 대단합니다. 하지만 우리는 더 나아갈 수 있지 않을까 생각했습니다.

대부분의 사람들은 43만 줄의 TypeScript가 필요하지 않습니다. 그들에게 필요한 것은 Telegram으로 대화하고, 파일을 읽고, 명령을 실행하고, 문제가 생기면 GitHub PR을 여는 에이전트입니다. 그게 전부입니다.

RustClaw는 OpenClaw의 80/20 버전입니다. 정말 중요한 기능만을 하나의 `cargo build`에 담았습니다.

<table>
<tr><td></td><td><strong>RustClaw</strong></td><td><strong>OpenClaw</strong></td></tr>
<tr><td>📦 바이너리</td><td><strong>7.5 MB</strong> 정적</td><td>Node.js 24 + npm 필요</td></tr>
<tr><td>💾 유휴 메모리</td><td><strong>14 MB</strong></td><td>1 GB+</td></tr>
<tr><td>⚡ 시작</td><td><strong>&lt; 100 ms</strong></td><td>5–10초</td></tr>
<tr><td>📝 코드</td><td><strong>5,296 라인</strong></td><td>~430,000 라인</td></tr>
<tr><td>🧠 메모리</td><td>3계층(벡터 + 그래프 + 히스토리)</td><td>기본 세션</td></tr>
<tr><td>🔧 도구</td><td>22개 내장 + MCP</td><td>플러그인 시스템</td></tr>
<tr><td>🤖 LLM</td><td>Anthropic, OpenAI, Ollama, Gemini</td><td>OpenAI</td></tr>
<tr><td>📱 채널</td><td>Telegram, Discord, WebSocket</td><td>Web UI</td></tr>
</table>

> [!NOTE]
> RustClaw는 OpenClaw를 대체하려는 것이 아닙니다. AI 에이전트의 정말 유용한 핵심은 1GB의 메모리가 필요하지 않다는 것을 증명합니다. 필요한 것은 좋은 아키텍처, 올바른 언어, 그리고 더 명확한 제약 조건으로 다시 시작하려는 의지입니다.

[Ad Huang](https://github.com/Adaimade)이 [Claude Code](https://claude.ai/code)만을 사용해 구축했습니다. 사람이 작성한 코드는 한 줄도 없습니다.

---

## 💡 핵심 장점

**🪶 어디서나 실행** — 7.5 MB 바이너리, 14 MB 메모리. 라즈베리 파이, 5달러 VPS, 노트북에서 실행. Node.js, Python, Docker 불필요.

**🧠 모든 것을 기억** — 3계층 메모리(벡터 + 그래프 + 히스토리), 하이브리드 범위 스코핑. Telegram에서 봇에게 이름을 말하면 Discord에서도 기억합니다. 사실 자동 추출, 모순 자동 해결.

**🛡️ 보안 우선** — 14가지 위험한 명령 패턴 차단. 도구 출력 잘림. 패치 파일은 수정 전에 검증. 오류 자동 재시도 복구. 120초 타임아웃과 우아한 폴백.

**🔧 실제로 작업 수행** — 98.9% BFCL 1,000문항. 듀얼 모델 전략(qwen3-coder + qwen2.5)으로 정확도와 속도를 최적화. 봇은 실제로 파일을 읽고, 명령을 실행하고, PR을 엽니다. "할 수 있다"고 설명만 하지 않습니다.

**🔌 MCP 지원** — 모든 MCP 서버에 연결. 도구 자동 발견, 투명한 라우팅. LLM은 통합된 도구 목록을 봅니다. 로컬과 원격의 차이가 없습니다.

**📈 벤치마크 검증 완료** — 일상 운영, 코딩, 시스템 관리, 적대적 프롬프트를 다루는 500문항 전문 벤치마크. v3→v5 발전: 81% → 97%. 제로 타임아웃.

**⚙️ Claude Code에서 영감** — 이해 우선 도구 순서, 히스토리 압축, 워크스페이스 컨텍스트 로딩, 오류 재시도 힌트. Claude Code를 효과적으로 만드는 동일한 패턴을 오픈 소스 에이전트에 적용했습니다.

---

## 🚀 빠른 시작

### 사전 요구사항

| 요구사항 | 설치 |
|---|---|
| Rust 1.85+ | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` |
| LLM 백엔드 | [Ollama](https://ollama.com), [OpenAI](https://platform.openai.com), [Anthropic](https://console.anthropic.com) 또는 [Gemini](https://ai.google.dev) |

### 원라인 설치

```bash
# macOS / Linux
curl -sSL https://raw.githubusercontent.com/Adaimade/RustClaw/main/install.sh | sh

# Windows (PowerShell)
irm https://raw.githubusercontent.com/Adaimade/RustClaw/main/install.ps1 | iex
```

> 사전 빌드된 바이너리를 자동으로 다운로드하고 PATH에 추가하며 기본 설정을 생성합니다. macOS(Intel/Apple Silicon), Linux(x86/ARM), Windows를 지원합니다.

### 소스에서 빌드

```bash
git clone https://github.com/Adaimade/RustClaw.git && cd RustClaw
cargo build --release && strip target/release/rustclaw
# → target/release/rustclaw (7.5 MB)
```

### 설정

```bash
mkdir -p ~/.rustclaw
cp config.example.toml ~/.rustclaw/config.toml
```

<table>
<tr>
<td><strong>Ollama (로컬)</strong></td>
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

> **보안 알림:** RustClaw는 클라우드 배포 편의성을 위해 기본적으로 `0.0.0.0`에 바인딩합니다. API 키를 절대 코드에 하드코딩하지 마세요. `~/.rustclaw/config.toml`(gitignored) 또는 환경 변수(`RUSTCLAW__AGENT__API_KEY`)를 사용하세요.

### 실행

```bash
# 전체 시작 (gateway + 채널 + cron + 메모리)
rustclaw gateway

# 도구 접근이 포함된 단일 에이전트 호출
rustclaw agent "모든 .rs 파일을 나열하고 총 라인 수를 계산하세요"

# GitHub 작업
rustclaw github scan
rustclaw github fix 123
```

---

## ✨ 기능

### 🔧 도구 호출 (Agentic Loop)

22개의 내장 도구가 자율적으로 실행됩니다. Anthropic 및 OpenAI 함수 호출 지원. 요청당 최대 10회 반복.

**계층형 도구 로딩** — 먼저 이해하고, 그다음 행동하고, 마지막으로 검사:

```
👁️ 이해                    ⚡ 행동                  🔍 검사
├── read_file              ├── run_command           ├── process_check
├── list_dir               ├── write_file            ├── docker_status
└── search_code            └── patch_file            ├── system_stats
                                                     ├── http_ping
💬 Discord (요청 시)       📧 Email (요청 시)        ├── pm2_status
├── 채널 생성/삭제         ├── fetch_inbox           └── process_list
├── 역할 생성/주제 설정    ├── read_email
└── 킥/밴                  └── send_email
```

**보안:** 14가지 위험 패턴 차단 · 출력 4000자로 잘림 · 패치 검증 · 오류 재시도 힌트 · 120초 우아한 타임아웃

### 🧠 3계층 메모리

메모리 시스템은 [**R-Mem**](https://github.com/Adaimade/R-Mem)에 위임됩니다 — 벡터 회상, 사실 추출, 모순 해결, 엔티티 관계 그래프를 담당하는 별도의 Rust 크레이트입니다. RustClaw는 그 위에 혼합 스코프(mixed-mode scoping)를 추가한 얇은 래퍼입니다.

**혼합 스코프 회상** — 각 쿼리마다 세 가지 스코프를 병합:

| 범위 | 예시 | 공유 대상 |
|---|---|---|
| Local | `telegram:-100xxx` | 단일 그룹 |
| User | `user:12345` | 모든 채널의 한 사람 |
| Global | `global:system` | 모든 사람 |

### 📱 채널

| 채널 | 기능 |
|---|---|
| **Telegram** | 롱 폴링 · 스트리밍 편집 · ACL · 세션 히스토리 |
| **Discord** | @mention · 서버 관리 · `scan` / `fix issue #N` / `pr status` |
| **Gateway** | OpenClaw 호환 WebSocket, `:18789/ws`에 위치 |

### 🔌 MCP Client

```toml
[mcp]
servers = [
  { name = "fs", command = "npx @modelcontextprotocol/server-filesystem /tmp" },
]
```

### 🐙 GitHub · ⏰ Cron · 📧 Email

자동 리포지토리 스캔 · 이슈에서 자동 PR · 시스템 모니터링 알림 · 이메일 분류 — 모두 cron 스케줄링과 Discord 알림으로.

---

## 📊 Benchmark

### Berkeley Function Calling Leaderboard (BFCL)

**공식 [Gorilla BFCL](https://github.com/ShishirPatil/gorilla)** 벤치마크에서 테스트됨 — 업계 함수 호출 평가의 표준:

| 테스트 | qwen3-coder:30b | qwen2.5:32b | 속도 차이 |
|---|---|---|---|
| **BFCL simple_python** (400) | **99.75%** | 99.75% | 4.3× 빠름 |
| **BFCL multiple** (200) | **99.5%** | 99.5% | 4.3× 빠름 |
| **BFCL parallel** (200) | **100%** | 100% | 4.3× 빠름 |
| **BFCL parallel_multiple** (200) | **100%** | 100% | 4.3× 빠름 |
| **종합** (1,000) | **98.9%** | 99.8% | **4.3× 빠름** |

> 하드웨어: Mac Mini 2024, M4 Pro, 64 GB. qwen3-coder:30b는 MoE 아키텍처로 동일 정확도에서 4.3배 빠른 속도 달성.

### T-Eval (상하이 AI Lab)

**[T-Eval](https://github.com/open-compass/T-Eval)** 에서 테스트됨 — 상하이 AI Lab의 도구 사용 평가 스위트로 계획, 검색, 검토, 지시 따르기를 다룹니다:

| 테스트 | 점수 | 문항 | 속도 |
|---|---|---|---|
| **T-Eval retrieve** | **98%** (542/553) | 553 | 14.5초/문항 |
| **T-Eval plan** | **96%** (535/553) | 553 | 25.6초/문항 |
| **T-Eval review** | **96%** (472/487) | 487 | 3.5초/문항 |
| **T-Eval instruct** | **92%** (514/553) | 553 | 8.2초/문항 |

> 네 가지 핵심 카테고리, 총 2,146문항. 평균 **95.5%** — 도구 선택, 다단계 계획, 자가 검토 모두 강력함.

### 내부 Benchmark

500문항 도구 호출 벤치마크 (qwen2.5:32b, 로컬 Ollama):

| 버전 | 총점 | Timeout | 속도 |
|---|---|---|---|
| v3 baseline | 81% | 74 | 44초/문항 |
| v4 timeout fix | 85% | 3 | 36초/문항 |
| **v5 optimized** | **97%** | **0** | **38초/문항** |

| 카테고리 | v5 점수 |
|---|---|
| 핵심 작업 | 92% |
| 기본 도구 | 95% |
| 중급 작업 | **100%** |
| 고급 추론 | 98% |
| 환각 함정 | **100%** |
| 다단계 연쇄 | 99% |

> 벤치마크 문항은 [AI-Bench](https://github.com/Adaimade/AI-Bench)에 있습니다.

---

## 🏗️ 아키텍처

```
src/
├── main.rs              CLI dispatch + 시작
├── cli/mod.rs           clap subcommands
├── config.rs            TOML + env 설정
├── gateway/             WebSocket 서버 + 프로토콜 + handshake
├── agent/runner.rs      LLM streaming + agentic loop + 히스토리 압축
├── channels/            Telegram (teloxide) + Discord (serenity)
├── tools/               22개 도구: fs, shell, search, discord, email, system, github, mcp
├── session/             SessionStore(히스토리) + MemoryManager(R-Mem 래퍼)
└── cron/                예약 작업 (system, email, GitHub)
```

**27개 파일 · 5,296 라인 · 7.5 MB 바이너리 · 외부 서비스 제로**

---

## 🗺️ Roadmap

| 상태 | 기능 |
|---|---|
| ✅ | 도구 호출 (22개 도구 + agentic loop) |
| ✅ | 3계층 메모리 (벡터 + 그래프 + 하이브리드 범위) |
| ✅ | Telegram + Discord 채널 |
| ✅ | MCP client (투명 도구 라우팅) |
| ✅ | GitHub 통합 (스캔 + 자동 PR) |
| ✅ | 시스템 모니터링 + cron 알림 |
| ✅ | Email (IMAP + SMTP) |
| ✅ | SQLite 영속성 |
| ✅ | 크로스 플랫폼 설치 (macOS / Linux / Windows) |
| 🟡 | 다중 모델 라우팅 (수동 env/config 전환 가능; 자동 라우팅 계획 중) |
| 🔲 | Slack / LINE 채널 |
| 🔲 | 멀티 에이전트 라우팅 |
| 🔲 | Prometheus metrics |

커뮤니티 기여를 환영합니다 — 이슈나 PR을 열어주세요.

---

<div align="center">

**MIT License** · v0.5.0

[Ad Huang](https://github.com/Adaimade)이 [Claude Code](https://claude.ai)를 사용해 만들었습니다

*프레임워크는 여기 있습니다. 나머지는 커뮤니티에 맡깁니다.*

</div>
