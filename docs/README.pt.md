🌐 [English](../README.md) · [繁體中文](README.zh-TW.md) · [简体中文](README.zh-CN.md) · [日本語](README.ja.md) · [한국어](README.ko.md) · [Español](README.es.md)

# RustClaw

**Um substituto leve em Rust para o OpenClaw.** Binário único. Sem runtime. Capacidades completas de agent.

|                   | **RustClaw**          | **OpenClaw**              |
|-------------------|-----------------------|---------------------------|
| Binary / Runtime  | **6 MB** estático     | requer Node.js 24 + npm   |
| Memória idle (RSS)| **7.9 MB**            | 1 GB+                     |
| Tempo de início   | **< 100 ms**          | 5-10 s                    |
| Linhas de código  | **~4,000**            | ~430,000                  |
| Dependências      | Compiladas e incluídas| npm install...             |

---

## Por quê

O OpenClaw faz muita coisa. Demais, para a maioria dos casos. Se você só precisa de um agent LLM que se conecte ao Telegram, Discord e GitHub — com acesso a ferramentas e um plano de controle WebSocket — você não precisa de 430K linhas de TypeScript e 1GB de memória.

RustClaw é a versão 80/20: as funcionalidades que importam, em um único `cargo build`.

Construído inteiramente com [Claude Code](https://claude.ai/code). Zero código escrito por humanos.

---

## Início rápido

### Pré-requisitos

- Toolchain Rust (`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`)
- Backend LLM: [Ollama](https://ollama.com) (local) ou [Anthropic API key](https://console.anthropic.com)

### Compilar

```bash
git clone https://github.com/Adaimade/RustClaw.git && cd RustClaw
cargo build --release
# Binary: target/release/rustclaw (6 MB)
```

### Configurar

```bash
mkdir -p ~/.rustclaw
cp config.example.toml ~/.rustclaw/config.toml
```

Configuração mínima para Ollama:

```toml
[agent]
provider = "openai"
api_key = "ollama"
base_url = "http://127.0.0.1:11434"
model = "qwen2.5:32b"
system_prompt = "You are a coding assistant with tool access."
```

Para Anthropic:

```toml
[agent]
provider = "anthropic"
api_key = "sk-ant-..."
model = "claude-sonnet-4-20250514"
```

### Executar

```bash
# Iniciar tudo (gateway + channels + cron)
rustclaw gateway

# Ou conversar diretamente com o agent
rustclaw agent "Liste todos os arquivos .rs em src/"
```

---

## Funcionalidades

### Gateway (Plano de controle WebSocket)

Protocolo WebSocket compatível com OpenClaw em `ws://127.0.0.1:18789/ws`.

### Channels

#### Telegram

Long polling via teloxide. Respostas streaming com edição progressiva de mensagens.

#### Discord

Bot baseado em Serenity. Responde a @mentions e DMs. Ferramentas de gerenciamento de servidor integradas.

### Tool Calling (Loop Agentic)

O agent tem acesso a ferramentas e as usa de forma autônoma. Suporta formatos de function calling do Anthropic e OpenAI. Máximo de 10 iterações de tool por requisição.

**Ferramentas integradas:**

| Tool | Descrição |
|---|---|
| `read_file` | Ler conteúdo de arquivos (auto-trunca >100KB) |
| `write_file` | Escrever/criar arquivos (cria diretórios automaticamente) |
| `patch_file` | Buscar e substituir em arquivos |
| `list_dir` | Árvore de diretório (profundidade limitada) |
| `run_command` | Execução de shell (limitado ao workspace, com timeout) |
| `search_code` | Busca de código tipo grep (Rust puro) |
| `discord_create_channel` | Criar canal text/voice/category |
| `discord_delete_channel` | Excluir um canal |
| `discord_create_role` | Criar um cargo com cor |
| `discord_set_channel_topic` | Definir tópico do canal |
| `discord_kick_member` | Expulsar um membro |
| `discord_ban_member` | Banir um membro |

### Integração com GitHub

Escaneia repos, auto-gera PRs a partir de issues usando análise LLM.

```bash
rustclaw github scan          # escanear todos os repos configurados
rustclaw github fix 123       # auto-PR para issue #123
```

### Cron

```toml
[cron]
github_scan = "0 0 9 * * *"  # diário às 09:00
```

---

## Comandos do Discord

Ao mencionar o bot com @:

```
@RustClaw scan                    # Relatório de escaneamento de repos GitHub
@RustClaw fix issue #42           # Auto-gerar PR para issue 42
@RustClaw pr status               # Listar PRs criados pelo bot
@RustClaw leia src/main.rs e resuma
@RustClaw crie um canal chamado announcements
@RustClaw execute cargo test e me diga o que falhou
```

---

## CLI

```
rustclaw [OPTIONS] <COMMAND>

Commands:
  gateway              Iniciar gateway + channels habilitados + cron
  agent <MESSAGE>      Enviar mensagem ao agent (com acesso a ferramentas)
  health               Health check HTTP
  status               Check de status WebSocket
  github scan          Escanear repos configurados
  github fix <N>       Auto-PR para issue N

Options:
  -c, --config <PATH>  Caminho do arquivo de configuração
  -h, --help           Mostrar ajuda
  -V, --version        Mostrar versão
```

---

## Roadmap

- [ ] **MCP client** — Suporte a Model Context Protocol para tool servers externos
- [ ] **Web UI** — Dashboard leve no navegador
- [ ] **Slack channel** — Integração com Slack bot
- [ ] **LINE channel** — LINE Messaging API
- [ ] **Sessões persistentes** — Armazenamento SQLite
- [ ] **Multi-agent** — Diferentes modelos/prompts por canal
- [ ] **Sistema de plugins** — Carregamento dinâmico de ferramentas via WASM ou shared libs
- [ ] **Metrics** — Endpoint Prometheus `/metrics`

Contribuições da comunidade são bem-vindas. Abra uma issue ou PR.

---

## License

MIT
