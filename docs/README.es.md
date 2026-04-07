🌐 [English](../README.md) · [繁體中文](README.zh-TW.md) · [简体中文](README.zh-CN.md) · [日本語](README.ja.md) · [한국어](README.ko.md) · [Português](README.pt.md)

# RustClaw

**Un reemplazo ligero en Rust para OpenClaw.** Un solo binario. Sin runtime. Capacidades completas de agent.

|                   | **RustClaw**          | **OpenClaw**              |
|-------------------|-----------------------|---------------------------|
| Binary / Runtime  | **6 MB** estático     | requiere Node.js 24 + npm |
| Memoria idle (RSS)| **7.9 MB**            | 1 GB+                     |
| Tiempo de inicio  | **< 100 ms**          | 5-10 s                    |
| Líneas de código  | **~4,000**            | ~430,000                  |
| Dependencias      | Compiladas e incluidas| npm install...             |

---

## Por qué

OpenClaw hace mucho. Demasiado, para la mayoría de los casos. Si solo necesitas un agent LLM que se conecte con Telegram, Discord y GitHub — con acceso a herramientas y un plano de control WebSocket — no necesitas 430K líneas de TypeScript ni 1GB de memoria.

RustClaw es la versión 80/20: las funciones que importan, en un solo `cargo build`.

Construido completamente con [Claude Code](https://claude.ai/code). Cero código escrito por humanos.

---

## Inicio rápido

### Prerrequisitos

- Toolchain de Rust (`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`)
- Backend LLM: [Ollama](https://ollama.com) (local) o [Anthropic API key](https://console.anthropic.com)

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

Configuración mínima para Ollama:

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

### Ejecutar

```bash
# Iniciar todo (gateway + channels + cron)
rustclaw gateway

# O hablar directamente con el agent
rustclaw agent "Lista todos los archivos .rs en src/"
```

---

## Funcionalidades

### Gateway (Plano de control WebSocket)

Protocolo WebSocket compatible con OpenClaw en `ws://127.0.0.1:18789/ws`.

### Channels

#### Telegram

Long polling vía teloxide. Respuestas streaming con edición progresiva de mensajes.

#### Discord

Bot basado en Serenity. Responde a @mentions y DMs. Herramientas de gestión de servidor integradas.

### Tool Calling (Bucle Agentic)

El agent tiene acceso a herramientas y las usa de forma autónoma. Soporta formatos de function calling de Anthropic y OpenAI. Máximo 10 iteraciones de tool por solicitud.

**Herramientas integradas:**

| Tool | Descripción |
|---|---|
| `read_file` | Leer contenido de archivos (auto-trunca >100KB) |
| `write_file` | Escribir/crear archivos (crea directorios automáticamente) |
| `patch_file` | Buscar y reemplazar en archivos |
| `list_dir` | Árbol de directorio (profundidad limitada) |
| `run_command` | Ejecución de shell (limitado al workspace, con timeout) |
| `search_code` | Búsqueda de código tipo grep (Rust puro) |
| `discord_create_channel` | Crear canal text/voice/category |
| `discord_delete_channel` | Eliminar un canal |
| `discord_create_role` | Crear un rol con color |
| `discord_set_channel_topic` | Establecer tema del canal |
| `discord_kick_member` | Expulsar a un miembro |
| `discord_ban_member` | Banear a un miembro |

### Integración con GitHub

Escanea repos, auto-genera PRs desde issues usando análisis LLM.

```bash
rustclaw github scan          # escanear todos los repos configurados
rustclaw github fix 123       # auto-PR para issue #123
```

### Cron

```toml
[cron]
github_scan = "0 0 9 * * *"  # diario a las 09:00
```

---

## Comandos de Discord

Al hacer @mention al bot:

```
@RustClaw scan                    # Reporte de escaneo de repos GitHub
@RustClaw fix issue #42           # Auto-generar PR para issue 42
@RustClaw pr status               # Listar PRs creados por el bot
@RustClaw lee src/main.rs y resúmelo
@RustClaw crea un canal llamado announcements
@RustClaw ejecuta cargo test y dime qué falla
```

---

## CLI

```
rustclaw [OPTIONS] <COMMAND>

Commands:
  gateway              Iniciar gateway + channels habilitados + cron
  agent <MESSAGE>      Enviar mensaje al agent (con acceso a herramientas)
  health               Health check HTTP
  status               Check de estado WebSocket
  github scan          Escanear repos configurados
  github fix <N>       Auto-PR para issue N

Options:
  -c, --config <PATH>  Ruta al archivo de configuración
  -h, --help           Mostrar ayuda
  -V, --version        Mostrar versión
```

---

## Roadmap

- [ ] **MCP client** — Soporte Model Context Protocol para tool servers externos
- [ ] **Web UI** — Dashboard ligero en navegador
- [ ] **Slack channel** — Integración con Slack bot
- [ ] **LINE channel** — LINE Messaging API
- [ ] **Sesiones persistentes** — Almacenamiento SQLite
- [ ] **Multi-agent** — Diferentes modelos/prompts por canal
- [ ] **Sistema de plugins** — Carga dinámica de herramientas vía WASM o shared libs
- [ ] **Metrics** — Endpoint Prometheus `/metrics`

Contribuciones de la comunidad son bienvenidas. Abre un issue o PR.

---

## License

MIT
