<div align="center">

# RustClaw

### Framework de Agentes de IA — en Rust

**Un reemplazo ligero para [OpenClaw](https://github.com/nicepkg/OpenClaw).**<br>
**Un solo binario. 22 herramientas. Memoria de tres niveles. Telegram + Discord + MCP.**

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](../LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.85+-orange.svg)](https://www.rust-lang.org/)
[![Built with Claude Code](https://img.shields.io/badge/Built%20with-Claude%20Code-blueviolet)](https://claude.ai)

**Binario de 7.5 MB** · **14 MB de RAM** · **5,918 líneas** · **99.7% BFCL** · **95.5% T-Eval** · **0% de alucinaciones**

[Inicio rápido](#-quick-start) · [Características](#-features) · [Benchmark](#-benchmark) · [Arquitectura](#-architecture) · [Roadmap](#-roadmap)

🌐 [English](../README.md) · [繁體中文](README.zh-TW.md) · [简体中文](README.zh-CN.md) · [日本語](README.ja.md) · [한국어](README.ko.md) · [Português](README.pt.md)

</div>

---

## ¿Por qué RustClaw?

La idea nació de una observación simple: alguien reescribió OpenClaw en Go y redujo el uso de memoria de más de 1 GB a 35 MB. Impresionante. Pero nos preguntamos: ¿podríamos ir aún más lejos?

La mayoría de la gente no necesita 430.000 líneas de TypeScript. Necesitan un agente que hable con Telegram, lea sus archivos, ejecute su código y abra un PR en GitHub cuando algo se rompa. Nada más.

RustClaw es la versión 80/20 de OpenClaw: las funciones que realmente importan, en un solo `cargo build`.

<table>
<tr><td></td><td><strong>RustClaw</strong></td><td><strong>OpenClaw</strong></td></tr>
<tr><td>📦 Binario</td><td><strong>7.5 MB</strong> estático</td><td>requiere Node.js 24 + npm</td></tr>
<tr><td>💾 RAM en reposo</td><td><strong>14 MB</strong></td><td>1 GB+</td></tr>
<tr><td>⚡ Arranque</td><td><strong>&lt; 100 ms</strong></td><td>5–10 s</td></tr>
<tr><td>📝 Código</td><td><strong>5,918 líneas</strong></td><td>~430,000 líneas</td></tr>
<tr><td>🧠 Memoria</td><td>Tres niveles (vector + grafo + historial)</td><td>Sesión básica</td></tr>
<tr><td>🔧 Herramientas</td><td>22 integradas + MCP</td><td>Sistema de plugins</td></tr>
<tr><td>🤖 LLM</td><td>Anthropic, OpenAI, Ollama, Gemini</td><td>OpenAI</td></tr>
<tr><td>📱 Canales</td><td>Telegram, Discord, WebSocket</td><td>Interfaz web</td></tr>
</table>

> [!NOTE]
> RustClaw no pretende reemplazar a OpenClaw. Es la prueba de que el núcleo de lo que hace útil a un agente de IA no requiere un gigabyte de RAM. Requiere buena arquitectura, el lenguaje adecuado y la disposición a empezar de nuevo con restricciones más claras.

Construido íntegramente con [Claude Code](https://claude.ai/code) por [Ad Huang](https://github.com/Adaimade). Cero código escrito por humanos.

---

## 💡 Ventajas clave

**🪶 Funciona en cualquier parte** — Binario de 7.5 MB, 14 MB de RAM. Raspberry Pi, un VPS de 5 dólares o tu portátil. Sin Node.js, sin Python, sin Docker.

**🧠 Recuerda todo** — Memoria de tres niveles (vector + grafo + historial) con ámbito en modo mixto. Dile tu nombre al bot en Telegram y lo recordará en Discord. Hechos extraídos automáticamente, contradicciones resueltas automáticamente.

**🛡️ Seguro por diseño** — 14 patrones de comandos peligrosos bloqueados. Salida de herramientas truncada. Archivos de parche verificados antes de modificar. Recuperación automática con reintento ante errores. Timeout de 120 s con fallback elegante.

**🔧 Realmente hace cosas** — 97% de precisión en herramientas sobre un benchmark de 500 preguntas. 0% de tasa de alucinaciones. El bot lee tus archivos, ejecuta tus comandos, crea PRs — no se limita a describir lo que *haría*.

**🔌 Compatible con MCP** — Conecta cualquier servidor MCP. Las herramientas se descubren y enrutan de forma transparente. Tu LLM ve una única lista unificada de herramientas: locales y remotas, sin diferencia.

**📈 Con benchmarks demostrados** — Benchmark profesional de 500 preguntas que cubre operaciones diarias, programación, administración de sistemas y prompts adversarios. Mejora v3→v5: 81% → 97%. Cero timeouts.

**⚙️ Inspirado en Claude Code** — Ordenamiento de herramientas con el principio de entender primero, compresión de historial, carga de contexto de workspace, sugerencias de reintento ante errores. Los mismos patrones que hacen efectivo a Claude Code, aplicados a un agente de código abierto.

---

## 🚀 Inicio rápido

### Requisitos previos

| Requisito | Instalación |
|---|---|
| Rust 1.85+ | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` |
| Backend LLM | [Ollama](https://ollama.com), [OpenAI](https://platform.openai.com), [Anthropic](https://console.anthropic.com) o [Gemini](https://ai.google.dev) |

### Compilar y ejecutar

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

> **Seguridad:** RustClaw se enlaza a `0.0.0.0` por defecto para despliegues en la nube. Nunca pongas las API keys en el código: usa `~/.rustclaw/config.toml` (gitignored) o variables de entorno (`RUSTCLAW__AGENT__API_KEY`).

### Ejecutar

```bash
# Iniciar todo (gateway + canales + cron + memoria)
rustclaw gateway

# Llamada única al agente con acceso a herramientas
rustclaw agent "Lista todos los archivos .rs y cuenta las líneas totales de código"

# Operaciones de GitHub
rustclaw github scan
rustclaw github fix 123
```

---

## ✨ Características

### 🔧 Llamada a herramientas (Agentic Loop)

22 herramientas integradas con ejecución autónoma. Compatible con function calling de Anthropic y OpenAI. Máximo 10 iteraciones por petición.

**Carga de herramientas por capas** — primero entender, luego actuar, luego comprobar:

```
👁️ Entender                ⚡ Actuar                 🔍 Comprobar
├── read_file              ├── run_command           ├── process_check
├── list_dir               ├── write_file            ├── docker_status
└── search_code            └── patch_file            ├── system_stats
                                                     ├── http_ping
💬 Discord (bajo demanda)  📧 Email (bajo demanda)   ├── pm2_status
├── crear/eliminar canal   ├── fetch_inbox           └── process_list
├── create_role/set_topic  ├── read_email
└── kick/ban_member        └── send_email
```

**Seguridad:** 14 patrones peligrosos bloqueados · salida truncada a 4000c · verificación de parches · sugerencias de reintento ante errores · timeout elegante de 120 s

### 🧠 Memoria de tres niveles

Impulsada por la arquitectura de [R-Mem](https://github.com/Adaimade/R-Mem).

```
├─ 📝 Corto plazo ── historial de conversación (SQLite)
├─ 📦 Largo plazo ── extracción de hechos por LLM → deduplicación → ADD/UPDATE/DELETE/NONE
│    └── Mapeo de IDs enteros · detección de contradicciones · deduplicación semántica
└─ 🕸️ Grafo ─────── extracción de entidades + relaciones con borrado suave
```

**Recuperación en modo mixto** — tres ámbitos combinados:

| Ámbito | Ejemplo | Compartido entre |
|---|---|---|
| Local | `telegram:-100xxx` | Un solo grupo |
| Usuario | `user:12345` | Todos los canales de una persona |
| Global | `global:system` | Todos |

### 📱 Canales

| Canal | Características |
|---|---|
| **Telegram** | Long polling · edición en streaming · ACL · historial de sesión |
| **Discord** | @mención · gestión del servidor · `scan` / `fix issue #N` / `pr status` |
| **Gateway** | WebSocket compatible con OpenClaw en `:18789/ws` |

### 🔌 Cliente MCP

```toml
[mcp]
servers = [
  { name = "fs", command = "npx @modelcontextprotocol/server-filesystem /tmp" },
]
```

### 🐙 GitHub · ⏰ Cron · 📧 Email

Escaneo automático de repos · PRs automáticos desde issues · alertas de monitoreo del sistema · clasificación de email — todo programado vía cron, con notificaciones a Discord.

---

## 📊 Benchmark

### Berkeley Function Calling Leaderboard (BFCL)

Probado sobre el benchmark **oficial [Gorilla BFCL](https://github.com/ShishirPatil/gorilla)** — el estándar de la industria para evaluar function calling:

| Prueba | Puntuación | Preguntas | Velocidad |
|---|---|---|---|
| **BFCL simple_python** | **99.75%** (399/400) | 400 | 7.3s/p |
| **BFCL multiple** | **99.5%** (199/200) | 200 | 8.4s/p |
| **BFCL parallel** | **100%** (200/200) | 200 | 12.0s/p |
| **BFCL parallel_multiple** | **100%** (200/200) | 200 | 15.7s/p |

> 1.000 preguntas del benchmark oficial BFCL. Dos puntuaciones perfectas en function calling paralelo.

### T-Eval (Shanghai AI Lab)

Probado sobre **[T-Eval](https://github.com/open-compass/T-Eval)** — la suite de evaluación de uso de herramientas del Shanghai AI Lab, que cubre planificación, recuperación, revisión y seguimiento de instrucciones:

| Prueba | Puntuación | Preguntas | Velocidad |
|---|---|---|---|
| **T-Eval retrieve** | **98%** (542/553) | 553 | 14.5s/p |
| **T-Eval plan** | **96%** (535/553) | 553 | 25.6s/p |
| **T-Eval review** | **96%** (472/487) | 487 | 3.5s/p |
| **T-Eval instruct** | **92%** (514/553) | 553 | 8.2s/p |

> 2.146 preguntas en cuatro categorías centrales. Promedio del **95.5%** — fuerte en selección de herramientas, planificación multi-paso y autorrevisión.

### Benchmark interno

Benchmark de llamada a herramientas de 500 preguntas (qwen2.5:32b, Ollama local):

| Versión | Total | Timeout | Velocidad |
|---|---|---|---|
| v3 baseline | 81% | 74 | 44s/p |
| v4 timeout fix | 85% | 3 | 36s/p |
| **v5 optimized** | **97%** | **0** | **38s/p** |

| Categoría | Puntuación v5 |
|---|---|
| Operaciones básicas | 92% |
| Herramientas básicas | 95% |
| Tareas intermedias | **100%** |
| Razonamiento avanzado | 98% |
| Trampas de alucinación | **100%** |
| Cadenas multi-paso | 99% |

> Las preguntas del benchmark están disponibles en [AI-Bench](https://github.com/Adaimade/AI-Bench).

---

## 🏗️ Arquitectura

```
src/
├── main.rs              dispatch de CLI + arranque
├── cli/mod.rs           subcomandos clap
├── config.rs            configuración TOML + env
├── gateway/             servidor WebSocket + protocolo + handshake
├── agent/runner.rs      streaming LLM + agentic loop + compresión de historial
├── channels/            Telegram (teloxide) + Discord (serenity)
├── tools/               22 herramientas: fs, shell, search, discord, email, system, github, mcp
├── session/             MemoryManager + store SQLite + grafo + embedding + extracción
└── cron/                Tareas programadas (system, email, GitHub)
```

**30 archivos · 5,918 líneas · binario de 7.5 MB · cero servicios externos**

---

## 🗺️ Roadmap

| Estado | Característica |
|---|---|
| ✅ | Llamada a herramientas (22 herramientas + agentic loop) |
| ✅ | Memoria de tres niveles (vector + grafo + ámbito mixto) |
| ✅ | Canales Telegram + Discord |
| ✅ | Cliente MCP (enrutamiento transparente de herramientas) |
| ✅ | Integración con GitHub (escaneo + PR automático) |
| ✅ | Monitoreo del sistema + alertas por cron |
| ✅ | Email (IMAP + SMTP) |
| ✅ | Persistencia en SQLite |
| 🔲 | Dashboard con interfaz web |
| 🔲 | Canales Slack / LINE |
| 🔲 | RAG (búsqueda documental) |
| 🔲 | Enrutamiento multi-agente |
| 🔲 | Sistema de plugins WASM |
| 🔲 | Métricas Prometheus |

Contribuciones de la comunidad bienvenidas — abre un issue o PR.

---

<div align="center">

**Licencia MIT** · v0.4.0

Creado por [Ad Huang](https://github.com/Adaimade) con [Claude Code](https://claude.ai)

*El framework está aquí. El resto depende de la comunidad.*

</div>
