<div align="center">

# RustClaw

### Framework de Agentes de IA em Rust

**Uma alternativa enxuta ao [OpenClaw](https://github.com/nicepkg/OpenClaw).**<br>
**Binário único. 22 ferramentas. Memória em três camadas. Telegram + Discord + MCP.**

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](../LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.85+-orange.svg)](https://www.rust-lang.org/)
[![Built with Claude Code](https://img.shields.io/badge/Built%20with-Claude%20Code-blueviolet)](https://claude.ai)

**Binário de 7.5 MB** · **14 MB de RAM** · **5.918 linhas** · **99.7% BFCL** · **95.5% T-Eval** · **0% de alucinação**

[Início Rápido](#-início-rápido) · [Recursos](#-recursos) · [Benchmark](#-benchmark) · [Arquitetura](#-arquitetura) · [Roadmap](#-roadmap)

🌐 [English](../README.md) · [繁體中文](README.zh-TW.md) · [简体中文](README.zh-CN.md) · [日本語](README.ja.md) · [한국어](README.ko.md) · [Español](README.es.md)

</div>

---

## Por que RustClaw?

A ideia começou com uma observação simples: alguém reescreveu o OpenClaw em Go e reduziu o uso de memória de mais de 1 GB para 35 MB. Foi impressionante. Mas nos perguntamos — seria possível ir ainda mais longe?

A maioria das pessoas não precisa de 430.000 linhas de TypeScript. O que elas precisam é de um agente que conversa no Telegram, lê seus arquivos, executa seu código e abre um PR no GitHub quando algo quebra. Só isso.

RustClaw é a versão 80/20 do OpenClaw — os recursos que realmente importam, em um único `cargo build`.

<table>
<tr><td></td><td><strong>RustClaw</strong></td><td><strong>OpenClaw</strong></td></tr>
<tr><td>📦 Binário</td><td><strong>7.5 MB</strong> estático</td><td>requer Node.js 24 + npm</td></tr>
<tr><td>💾 RAM ocioso</td><td><strong>14 MB</strong></td><td>1 GB+</td></tr>
<tr><td>⚡ Inicialização</td><td><strong>&lt; 100 ms</strong></td><td>5–10 s</td></tr>
<tr><td>📝 Código</td><td><strong>5.918 linhas</strong></td><td>~430.000 linhas</td></tr>
<tr><td>🧠 Memória</td><td>Três camadas (vetor + grafo + histórico)</td><td>Sessão básica</td></tr>
<tr><td>🔧 Ferramentas</td><td>22 integradas + MCP</td><td>Sistema de plugins</td></tr>
<tr><td>🤖 LLM</td><td>Anthropic, OpenAI, Ollama, Gemini</td><td>OpenAI</td></tr>
<tr><td>📱 Canais</td><td>Telegram, Discord, WebSocket</td><td>Interface Web</td></tr>
</table>

> [!NOTE]
> RustClaw não tenta substituir o OpenClaw. Ele é a prova de que o núcleo do que torna um agente de IA útil não precisa de um gigabyte de RAM. Ele precisa de uma boa arquitetura, da linguagem certa e da disposição de recomeçar com restrições mais claras.

Construído inteiramente com [Claude Code](https://claude.ai/code) por [Ad Huang](https://github.com/Adaimade). Zero código escrito por humanos.

---

## 💡 Principais Vantagens

**🪶 Roda em qualquer lugar** — Binário de 7.5 MB, 14 MB de RAM. Raspberry Pi, VPS de 5 dólares ou seu notebook. Sem Node.js, sem Python, sem Docker.

**🧠 Lembra de tudo** — Memória em três camadas (vetor + grafo + histórico) com escopo em modo misto. Diga seu nome ao bot no Telegram e ele vai lembrar no Discord. Fatos extraídos automaticamente, contradições resolvidas automaticamente.

**🛡️ Seguro por design** — 14 padrões de comandos perigosos bloqueados. Saída de ferramentas truncada. Arquivos de patch verificados antes da modificação. Retry de erro com auto-recuperação. Timeout de 120 s com fallback gracioso.

**🔧 Realmente faz as coisas** — 97% de precisão de ferramentas em benchmark de 500 questões. 0% de taxa de alucinação. O bot lê seus arquivos, executa seus comandos, cria PRs — ele não apenas descreve o que *faria*.

**🔌 Compatível com MCP** — Conecte qualquer servidor MCP. Ferramentas descobertas automaticamente e roteadas de forma transparente. Seu LLM vê uma lista unificada de ferramentas — local e remoto, sem diferença.

**📈 Testado e comprovado** — Benchmark profissional de 500 questões cobrindo operações diárias, programação, administração de sistemas e prompts adversariais. Melhoria v3→v5: 81% → 97%. Zero timeouts.

**⚙️ Inspirado no Claude Code** — Ordenação de ferramentas pelo princípio "entender primeiro", compressão de histórico, carregamento de contexto do workspace, dicas de retry em erros. Os mesmos padrões que tornam o Claude Code eficaz, aplicados a um agente open source.

---

## 🚀 Início Rápido

### Pré-requisitos

| Requisito | Instalação |
|---|---|
| Rust 1.85+ | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` |
| Backend LLM | [Ollama](https://ollama.com), [OpenAI](https://platform.openai.com), [Anthropic](https://console.anthropic.com) ou [Gemini](https://ai.google.dev) |

### Compilar e Executar

```bash
git clone https://github.com/Adaimade/RustClaw.git && cd RustClaw
cargo build --release
# → target/release/rustclaw (7.5 MB)
```

### Configurar

```bash
mkdir -p ~/.rustclaw
cp config.example.toml ~/.rustclaw/config.toml
```

<table>
<tr>
<td><strong>Ollama (local)</strong></td>
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

> **Segurança:** O RustClaw faz bind em `0.0.0.0` por padrão para facilitar o deploy em nuvem. Nunca coloque chaves de API no código — use `~/.rustclaw/config.toml` (no gitignore) ou variáveis de ambiente (`RUSTCLAW__AGENT__API_KEY`).

### Executar

```bash
# Inicia tudo (gateway + canais + cron + memória)
rustclaw gateway

# Chamada única ao agente com acesso a ferramentas
rustclaw agent "List all .rs files and count total lines of code"

# Operações do GitHub
rustclaw github scan
rustclaw github fix 123
```

---

## ✨ Recursos

### 🔧 Chamada de Ferramentas (Loop Agêntico)

22 ferramentas integradas com execução autônoma. Suporta function calling da Anthropic e da OpenAI. Máximo de 10 iterações por requisição.

**Carregamento de ferramentas em camadas** — entender primeiro, depois agir, depois verificar:

```
👁️ Entender                ⚡ Agir                   🔍 Verificar
├── read_file              ├── run_command           ├── process_check
├── list_dir               ├── write_file            ├── docker_status
└── search_code            └── patch_file            ├── system_stats
                                                     ├── http_ping
💬 Discord (sob demanda)   📧 Email (sob demanda)    ├── pm2_status
├── criar/excluir canal    ├── fetch_inbox           └── process_list
├── create_role/set_topic  ├── read_email
└── expulsar/banir membro  └── send_email
```

**Segurança:** 14 padrões perigosos bloqueados · saída truncada em 4000c · verificação de patch · dicas de retry de erro · timeout gracioso de 120s

### 🧠 Memória em Três Camadas

Impulsionada pela arquitetura [R-Mem](https://github.com/Adaimade/R-Mem).

```
├─ 📝 Curto prazo ── histórico de conversas (SQLite)
├─ 📦 Longo prazo ── extração de fatos por LLM → dedup → ADD/UPDATE/DELETE/NONE
│    └── Mapeamento por ID inteiro · detecção de contradição · dedup semântico
└─ 🕸️ Grafo ─────── extração de entidades + relações com soft-delete
```

**Recuperação em modo misto** — três escopos mesclados:

| Escopo | Exemplo | Compartilhado em |
|---|---|---|
| Local | `telegram:-100xxx` | Um único grupo |
| User | `user:12345` | Todos os canais para uma pessoa |
| Global | `global:system` | Todos |

### 📱 Canais

| Canal | Recursos |
|---|---|
| **Telegram** | Long polling · edição em streaming · ACL · histórico de sessão |
| **Discord** | @mention · gerenciamento de servidor · `scan` / `fix issue #N` / `pr status` |
| **Gateway** | WebSocket compatível com OpenClaw em `:18789/ws` |

### 🔌 Cliente MCP

```toml
[mcp]
servers = [
  { name = "fs", command = "npx @modelcontextprotocol/server-filesystem /tmp" },
]
```

### 🐙 GitHub · ⏰ Cron · 📧 Email

Auto-scan de repositórios · auto-PR a partir de issues · alertas de monitoramento de sistema · classificação de e-mails — tudo agendado via cron, com notificações para o Discord.

---

## 📊 Benchmark

### Berkeley Function Calling Leaderboard (BFCL)

Testado no benchmark **oficial [Gorilla BFCL](https://github.com/ShishirPatil/gorilla)** — o padrão da indústria para avaliação de function calling:

| Teste | Pontuação | Questões | Velocidade |
|---|---|---|---|
| **BFCL simple_python** | **99.75%** (399/400) | 400 | 7.3s/q |
| **BFCL multiple** | **99.5%** (199/200) | 200 | 8.4s/q |
| **BFCL parallel** | **100%** (200/200) | 200 | 12.0s/q |
| **BFCL parallel_multiple** | **100%** (200/200) | 200 | 15.7s/q |

> 1.000 questões no benchmark oficial BFCL. Duas pontuações perfeitas em function calling paralelo.

### T-Eval (Shanghai AI Lab)

Testado no **[T-Eval](https://github.com/open-compass/T-Eval)** — suíte de avaliação de uso de ferramentas do Shanghai AI Lab, cobrindo planejamento, recuperação, revisão e seguimento de instruções:

| Teste | Pontuação | Questões | Velocidade |
|---|---|---|---|
| **T-Eval retrieve** | **98%** (542/553) | 553 | 14.5s/q |
| **T-Eval plan** | **96%** (535/553) | 553 | 25.6s/q |
| **T-Eval review** | **96%** (472/487) | 487 | 3.5s/q |
| **T-Eval instruct** | **92%** (514/553) | 553 | 8.2s/q |

> 2.146 questões em quatro categorias centrais. Média de **95.5%** — forte em seleção de ferramentas, planejamento multi-etapa e auto-revisão.

### Benchmark Interno

Benchmark de chamada de ferramentas com 500 questões (qwen2.5:32b, Ollama local):

| Versão | Total | Timeout | Velocidade |
|---|---|---|---|
| v3 baseline | 81% | 74 | 44s/q |
| v4 timeout fix | 85% | 3 | 36s/q |
| **v5 optimized** | **97%** | **0** | **38s/q** |

| Categoria | Pontuação v5 |
|---|---|
| Operações básicas | 92% |
| Ferramentas básicas | 95% |
| Tarefas de médio porte | **100%** |
| Raciocínio avançado | 98% |
| Armadilhas de alucinação | **100%** |
| Cadeias multi-etapa | 99% |

> Questões do benchmark disponíveis em [AI-Bench](https://github.com/Adaimade/AI-Bench).

---

## 🏗️ Arquitetura

```
src/
├── main.rs              Dispatch de CLI + inicialização
├── cli/mod.rs           Subcomandos clap
├── config.rs            Configuração TOML + env
├── gateway/             Servidor WebSocket + protocolo + handshake
├── agent/runner.rs      Streaming de LLM + loop agêntico + compressão de histórico
├── channels/            Telegram (teloxide) + Discord (serenity)
├── tools/               22 ferramentas: fs, shell, search, discord, email, system, github, mcp
├── session/             MemoryManager + store SQLite + grafo + embedding + extração
└── cron/                Tarefas agendadas (sistema, email, GitHub)
```

**30 arquivos · 5.918 linhas · binário de 7.5 MB · Zero serviços externos**

---

## 🗺️ Roadmap

| Status | Recurso |
|---|---|
| ✅ | Chamada de ferramentas (22 ferramentas + loop agêntico) |
| ✅ | Memória em três camadas (vetor + grafo + escopo misto) |
| ✅ | Canais Telegram + Discord |
| ✅ | Cliente MCP (roteamento transparente de ferramentas) |
| ✅ | Integração com GitHub (scan + auto-PR) |
| ✅ | Monitoramento de sistema + alertas via cron |
| ✅ | Email (IMAP + SMTP) |
| ✅ | Persistência em SQLite |
| 🔲 | Dashboard de UI Web |
| 🔲 | Canais Slack / LINE |
| 🔲 | RAG (busca em documentos) |
| 🔲 | Roteamento multi-agente |
| 🔲 | Sistema de plugins WASM |
| 🔲 | Métricas Prometheus |

Contribuições da comunidade são bem-vindas — abra uma issue ou PR.

---

<div align="center">

**MIT License** · v0.4.0

Criado por [Ad Huang](https://github.com/Adaimade) com [Claude Code](https://claude.ai)

*O framework está aqui. O resto é com a comunidade.*

</div>
