🌐 [English](../README.md) · [繁體中文](README.zh-TW.md) · [简体中文](README.zh-CN.md) · [日本語](README.ja.md) · [Español](README.es.md) · [Português](README.pt.md)

# RustClaw

**OpenClaw의 경량 Rust 대체품.** 단일 바이너리. 런타임 불필요. 완전한 agent 기능.

|                   | **RustClaw**       | **OpenClaw**              |
|-------------------|--------------------|---------------------------|
| Binary / Runtime  | **6 MB** 정적 링크  | Node.js 24 + npm 필요      |
| 유휴 메모리 (RSS)  | **7.9 MB**         | 1 GB+                     |
| 시작 시간          | **< 100 ms**       | 5-10초                    |
| 코드 라인 수       | **~4,000**         | ~430,000                  |
| 의존성 관리        | 컴파일 시 포함      | npm install...             |

---

## 왜 만들었나

OpenClaw은 기능이 많지만 대부분의 경우 과합니다. Telegram, Discord, GitHub와 연동하는 LLM agent만 필요하다면 — tool 호출과 WebSocket 제어 플레인까지 포함해서 — 43만 줄의 TypeScript와 1GB 메모리는 필요 없습니다.

RustClaw은 80/20 법칙의 산물입니다: 정말 중요한 기능만, 하나의 `cargo build`에 담았습니다.

[Claude Code](https://claude.ai/code)로 완전히 구축. 사람이 작성한 코드는 없습니다.

---

## 빠른 시작

### 사전 요구사항

- Rust 툴체인 (`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`)
- LLM 백엔드: [Ollama](https://ollama.com) (로컬) 또는 [Anthropic API key](https://console.anthropic.com)

### 빌드

```bash
git clone https://github.com/Adaimade/RustClaw.git && cd RustClaw
cargo build --release
# Binary: target/release/rustclaw (6 MB)
```

### 설정

```bash
mkdir -p ~/.rustclaw
cp config.example.toml ~/.rustclaw/config.toml
```

Ollama 최소 설정:

```toml
[agent]
provider = "openai"
api_key = "ollama"
base_url = "http://127.0.0.1:11434"
model = "qwen2.5:32b"
system_prompt = "You are a coding assistant with tool access."
```

Anthropic 설정:

```toml
[agent]
provider = "anthropic"
api_key = "sk-ant-..."
model = "claude-sonnet-4-20250514"
```

### 실행

```bash
# 모든 서비스 시작 (gateway + channels + cron)
rustclaw gateway

# 또는 agent와 직접 대화
rustclaw agent "src/ 디렉토리의 모든 .rs 파일을 나열해줘"
```

---

## 기능

### Gateway (WebSocket 제어 플레인)

OpenClaw 호환 WebSocket 프로토콜 (`ws://127.0.0.1:18789/ws`).

### Channels

#### Telegram

teloxide를 통한 long polling. 메시지 편집을 통한 streaming 응답 시뮬레이션.

#### Discord

Serenity 기반 bot. @mention과 DM에 응답. 서버 관리 도구 내장.

### Tool Calling (Agentic Loop)

Agent가 도구를 자율적으로 사용합니다. Anthropic과 OpenAI function calling 형식 모두 지원. 요청당 최대 10라운드 tool 반복.

**내장 도구:**

| Tool | 설명 |
|---|---|
| `read_file` | 파일 읽기 (100KB 초과 시 자동 절단) |
| `write_file` | 파일 쓰기/생성 (디렉토리 자동 생성) |
| `patch_file` | 파일 내 찾기 및 바꾸기 |
| `list_dir` | 디렉토리 트리 (깊이 제한) |
| `run_command` | Shell 실행 (workspace 내 제한, 타임아웃) |
| `search_code` | grep 스타일 코드 검색 (순수 Rust) |
| `discord_create_channel` | text/voice/category 채널 생성 |
| `discord_delete_channel` | 채널 삭제 |
| `discord_create_role` | 역할 생성 (색상 포함) |
| `discord_set_channel_topic` | 채널 주제 설정 |
| `discord_kick_member` | 멤버 추방 |
| `discord_ban_member` | 멤버 차단 |

### GitHub 통합

저장소 스캔, LLM 분석을 사용하여 issue에서 PR 자동 생성.

```bash
rustclaw github scan          # 설정된 모든 저장소 스캔
rustclaw github fix 123       # issue #123의 auto-PR 생성
```

### Cron 스케줄

```toml
[cron]
github_scan = "0 0 9 * * *"  # 매일 09:00
```

---

## Discord 명령어

bot을 @mention하면 사용 가능:

```
@RustClaw scan                    # GitHub 저장소 스캔 리포트
@RustClaw fix issue #42           # issue 42의 PR 자동 생성
@RustClaw pr status               # bot이 만든 PR 목록
@RustClaw src/main.rs 읽고 요약해줘
@RustClaw announcements 채널 만들어줘
@RustClaw cargo test 실행하고 실패한 곳 알려줘
```

---

## CLI

```
rustclaw [OPTIONS] <COMMAND>

Commands:
  gateway              gateway + 활성화된 channels + cron 시작
  agent <MESSAGE>      agent에 메시지 전송 (tool 접근 포함)
  health               HTTP 헬스 체크
  status               WebSocket 상태 확인
  github scan          설정된 저장소 스캔
  github fix <N>       issue N의 PR 자동 생성

Options:
  -c, --config <PATH>  설정 파일 경로
  -h, --help           도움말 표시
  -V, --version        버전 표시
```

---

## Roadmap

- [ ] **MCP client** — 외부 tool 서버용 Model Context Protocol 지원
- [ ] **Web UI** — 세션과 로그를 위한 경량 브라우저 대시보드
- [ ] **Slack channel** — Slack bot 통합
- [ ] **LINE channel** — LINE Messaging API
- [ ] **영속적 세션** — SQLite 백엔드
- [ ] **멀티 agent** — 채널별 다른 모델/프롬프트
- [ ] **Plugin 시스템** — WASM 또는 shared lib를 통한 동적 도구 로딩
- [ ] **Metrics** — Prometheus `/metrics` 엔드포인트

커뮤니티 기여를 환영합니다. issue 또는 PR을 보내주세요.

---

## License

MIT
