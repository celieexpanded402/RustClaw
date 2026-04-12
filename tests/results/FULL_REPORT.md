# RustClaw 完整評測報告

**最後更新：** 2026-04-12
**平台：** Mac Mini 2024, Apple M4 Pro, 64 GB 統一記憶體, macOS arm64
**預設 LLM：** Ollama qwen3-coder:30b (本地, MoE) — 2026-04-12 起
**舊預設：** Ollama qwen2.5:32b (本地, dense) — 2026-04-10 前
**版本：** RustClaw v0.2.0

---

## 一、系統指標

| 指標 | 值 | 對比 OpenClaw |
|---|---|---|
| **Binary** | 7.5 MB | requires Node.js 24 + npm |
| **Idle RSS** | 14 MB | 1 GB+ |
| **啟動時間** | < 100 ms | 5-10 秒 |
| **程式碼** | ~5,296 行 | ~430,000 行 |
| **HTTP 延遲** | ~23 ms (/health) | — |
| **Agent 延遲** | ~1.5-3 秒 (warm, qwen3-coder) | — |
| **直接依賴** | 24 crates (Cargo 33 total) | — |
| **DB** | SQLite (sessions.db) | — |

---

## 二、BFCL 1000 題 Tool Calling Benchmark（業界標準）

### 當前預設：qwen3-coder:30b (MoE, 2026-04-12)

| 分類 | 題數 | 通過 | 正確率 | 速度 |
|---|---|---|---|---|
| simple_python | 400 | 400 | **100%** | 1.5s/q |
| multiple | 200 | 194 | **97%** | 2.4s/q |
| parallel | 200 | 199 | **99.5%** | 2.9s/q |
| parallel_multiple | 200 | 196 | **98%** | 3.4s/q |
| **Overall** | **1000** | **989** | **98.9%** | **2.6s/q** |

### 舊預設：qwen2.5:32b (dense, 2026-04-10)

| 分類 | 題數 | 通過 | 正確率 | 速度 |
|---|---|---|---|---|
| simple_python | 400 | 399 | **99.75%** | 7.3s/q |
| multiple | 200 | 199 | **99.5%** | 8.4s/q |
| parallel | 200 | 200 | **100%** | 12.0s/q |
| parallel_multiple | 200 | 200 | **100%** | 15.7s/q |
| **Overall** | **1000** | **998** | **99.7%** | **10.8s/q** |

### 對比結論

| 指標 | qwen2.5:32b | qwen3-coder:30b |
|---|---|---|
| 準確度 | 99.7% | 98.9% (-0.8%) |
| 速度 | 10.8s/q | 2.6s/q (**4.3× 快**) |
| 大小 | 19 GB | 18 GB |
| 架構 | Dense | MoE |

決策：接受 -0.8% 準確度換取 4.3× 加速,於 2026-04-12 更換預設。

### 內部 500 題 AI-Bench（qwen2.5:32b, 2026-04-09）

| 版本 | 分數 |
|---|---|
| v5 optimized | **97%** (485/500) |
| v4 timeout-fix | 85% |
| v3 baseline | 81% |

（AI-Bench 尚未在 qwen3-coder:30b 上重跑）

---

## 二-B、中文寫作測試（2026-04-12）

6 主題（金融/經濟/政治/地緣/加密貨幣/生活）× 3 模型改寫比較：

| 模型 | 總分(/60) | 速度 | 繁體純淨度 | 強項 |
|---|---|---|---|---|
| **gemma4:26b** | **55** | 3s/題 | 100% | 新聞改寫、數字精準 |
| gemma4:E4b | 48 | 21s/題 | 100% | 散文戲劇化 |
| qwen3:8b | 43 | 11s/題 | ❌ 繁簡混用 | — |

結論：gemma4:26b 為中文寫作首選。

---

## 三、模型對比總覽

| 模型 | 大小 | Tool Calling | BFCL 1000Q | 中文寫作 | 角色 |
|---|---|---|---|---|---|
| **qwen3-coder:30b** | 18 GB | ✅ (MoE) | **98.9%** | 合格(需 prompt 約束繁體) | ⭐ 預設 |
| qwen2.5:32b | 19 GB | ✅ (Dense) | 99.7% | 良好 | 高精準後備 |
| gpt-oss:20b | 13 GB | ✅ | 未測(smoke passed) | 未測 | 輕量後備 |
| **gemma4:26b** | 17 GB | ❌ (0%) | — | **最佳** | 中文寫作 |
| gemma4:E4b | 9.6 GB | ❌ (0%) | — | 良好 | 寫作備選 |
| nomic-embed-text | 274 MB | — | — | — | Embedding |

### 已測試並淘汰的模型

| 模型 | 原因 |
|---|---|
| qwen2.5:7b / 14b | text-mode JSON, tool calling 失效 |
| qwen3:8b | 繁簡混用 + 英文 code-switch |
| hermes3:8b | 幻覺答案,比放棄還差 |
| gemma4:31b | 推論 > 120s, 超過 RustClaw timeout |

**結論：8b 級模型在 Ollama + RustClaw 堆疊全部失敗。Tool calling 門檻 ≥ 13 GB。**

---

## 四、已實現功能清單

### 核心
- [x] Gateway WebSocket 控制平面（OpenClaw 相容）
- [x] CLI：gateway / agent / health / status / github
- [x] Config：TOML + env overlay + 裸 API key

### Channels
- [x] Telegram（長輪詢 + streaming edit）
- [x] Discord（@mention + 伺服器管理 + bot 指令）
- [x] WebSocket Gateway

### LLM
- [x] Anthropic API
- [x] OpenAI compatible（Ollama / Gemini / vLLM）
- [x] Tool calling agentic loop（最多 10 輪）
- [x] 雙 provider function calling（Anthropic + OpenAI 格式）

### Tools（22 個）
- [x] read_file, write_file, patch_file, list_dir
- [x] run_command（沙箱 + timeout）
- [x] search_code（純 Rust）
- [x] discord_create_channel, delete_channel, create_role, set_topic, kick, ban
- [x] process_list, process_check, docker_status, docker_inspect, pm2_status
- [x] http_ping, system_stats
- [x] fetch_inbox, read_email, send_email

### 記憶系統（R-Mem 整合）
- [x] 短期：對話歷史（SQLite）
- [x] 長期：LLM 事實抽取 → 向量去重 → ADD/UPDATE/DELETE/NONE
- [x] Graph：實體 + 關係抽取，軟刪除
- [x] 混合 scope：local + user + global
- [x] 整數 ID 映射（防 UUID 幻覺）
- [x] 自我指代解析（I/me → user_id）
- [x] 持久化（SQLite, sessions.db）

### 整合
- [x] GitHub：掃描 issues/PRs + 自動 PR
- [x] Cron：system_check / email_scan / github_scan
- [x] MCP client：stdio transport, tool 自動發現 + 路由
- [x] Email：IMAP 讀信 + SMTP 發信

### 部署
- [x] Dockerfile（多階段編譯）
- [x] Zeabur template
- [x] .dockerignore

---

## 五、需要優化 / 強化的部分

### 🔴 優先（影響核心體驗）

| 項目 | 問題 | 建議方案 |
|---|---|---|
| **System prompt 強化** | 低階操作只有 68%，模型有時不呼叫 tool | 增加更多 few-shot 範例，覆蓋英文指令；針對 B 類題型加範例 |
| **Tool calling 可靠性** | 74 個 timeout（15%），模型思考太久 | 加 LLM timeout 設定（單次呼叫 30s），失敗時 fallback 到純文字回覆 |
| **回應最小化問題** | 91 題回答太短被誤判 | tool 執行後強制附帶結果摘要（不只回一個 "done"） |
| **英文指令偏弱** | B 類英文題大量失敗 | system prompt 加英文 few-shot 範例 |
| **recall() 未接入對話** | memory.recall() 寫好了但 channel 沒呼叫 | 在 Telegram/Discord handler 裡加 recall() 注入 |

### 🟡 中優先（提升穩定性）

| 項目 | 問題 | 建議方案 |
|---|---|---|
| **LLM fallback** | Ollama 掛了整個 agent 失敗 | 加 fallback provider（本地掛了切雲端）|
| **Tool 執行安全** | run_command 的 workspace 限制可被繞過 | 加 chroot 或 seccomp 沙箱 |
| **Embedding 模型** | nomic-embed-text 精度一般 | 支援可配置的 embedding model |
| **向量搜尋效能** | 暴力 cosine（O(n) per query）| 加 HNSW 索引或用 SQLite FTS5 |
| **Cron 通知** | 通知只能發 Discord | 加 Telegram 通知支援 |
| **Graph 搜尋** | 純關鍵字匹配 | 加 embedding-based 圖搜尋 |

### 🟢 低優先（錦上添花）

| 項目 | 問題 | 建議方案 |
|---|---|---|
| **Web UI** | 沒有視覺化面板 | 輕量 HTML dashboard（sessions, memory, graph）|
| **多 agent** | 所有 channel 用同一個 model | 路由：不同 channel 用不同 model/prompt |
| **Rate limiting** | 沒有請求限流 | 加 tower rate limit middleware |
| **Metrics** | 沒有監控指標 | Prometheus /metrics endpoint |
| **單元測試** | 只有 integration test | 加 unit test for extract, embed, graph |
| **Plugin 系統** | 加新 tool 要改 executor.rs | WASM plugin 或 dynamic dispatch |
| **RAG** | 沒有文件知識庫 | 文件切片 + 向量搜尋 |
| **語音** | 不支援語音 | Whisper STT + TTS |

---

## 六、下一步建議（按優先級）

### Phase 1：立即可做（提升到 90%+）
1. **強化 system prompt** — 加更多 few-shot（特別是英文 + 低階操作）
2. **接入 recall()** — 讓每次對話都帶記憶上下文
3. **加 LLM timeout** — 單次呼叫 30s，超時 graceful fallback

### Phase 2：穩定性（1-2 週）
4. **LLM fallback** — 本地掛了自動切 Gemini/Anthropic
5. **向量搜尋加速** — HNSW 或分桶
6. **Cron 多通知** — Telegram + Discord

### Phase 3：功能擴展（1 個月）
7. **Web UI** — sessions dashboard
8. **RAG** — 文件知識庫
9. **多 agent 路由** — channel → model mapping
10. **Plugin 系統** — 外部 tool 動態載入

---

## 七、結論

RustClaw 用 ~5,296 行 Rust 實現了 OpenClaw ~430,000 行 TypeScript 的核心功能。

- **7.5 MB binary，14 MB RAM** — 可以跑在任何地方
- **98.9% BFCL tool calling（qwen3-coder:30b）** — 本地推論，2.6s/題
- **99.7% BFCL（qwen2.5:32b）** — 高精準後備
- **三層記憶系統（R-Mem）** — 短期 + 長期向量 + Graph，混合 scope
- **22 個內建工具 + MCP** — 檔案、Shell、搜尋、Discord、Email、GitHub
- **繁體中文約束** — System prompt LANGUAGE 區塊確保 Qwen3 輸出 100% 繁體

2026-04-12 模型切換決策：qwen3-coder:30b (MoE) 以 -0.8% 準確度換取 4.3× 加速，成為新預設。qwen2.5:32b 保留做高精準後備。gemma4:26b 負責中文寫作。
