# awesome-sqlite PR

**Repo:** https://github.com/planetopendata/awesome-sqlite
**Section:** Applications / AI & Machine Learning

---

**PR Title:** Add RustClaw — AI agent framework using SQLite for vector memory + knowledge graph

**Entry to add:**

```markdown
- [RustClaw](https://github.com/Adaimade/RustClaw) — Lightweight AI agent framework (7.5 MB binary) that uses SQLite as its sole storage backend: conversation history, vector memory with cosine similarity search, and a knowledge graph with soft-delete — all in rusqlite with zero external services.
```

**PR Description:**

RustClaw is a lightweight AI agent framework in Rust (7.5 MB, 14 MB RAM) that uses SQLite as its only storage backend for three distinct purposes:

1. **Conversation history** — standard session messages table with per-user isolation
2. **Vector memory** — embeddings stored as BLOBs, cosine similarity computed in application layer, with LLM-powered deduplication (ADD/UPDATE/DELETE/NONE decisions)
3. **Knowledge graph** — entity-relation tables with soft-delete (`valid` flag), multi-value relation detection, and mention counting

All three use a single `sessions.db` file via rusqlite (bundled SQLite). No Qdrant, no Neo4j, no Redis — just SQLite.

The project also integrates with [R-Mem](https://github.com/Adaimade/R-Mem) which takes the SQLite-only approach further with FTS5 pre-filtering for 19x faster vector search.

**Why it fits awesome-sqlite:**
- Demonstrates SQLite as a viable vector database for AI workloads (embedding storage + cosine similarity)
- Shows SQLite as a lightweight graph database (entity-relation with soft-delete)
- Single-file database for a complete AI agent (chat + memory + graph)
- rusqlite with bundled feature — truly zero external dependencies
