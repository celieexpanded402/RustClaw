use std::path::Path;
use std::sync::Arc;

use rusqlite::Connection;
use tokio::sync::Mutex;
use tracing::info;

use crate::agent::Message;

#[derive(Debug, Clone)]
pub struct MemoryResult {
    pub id: String,
    pub text: String,
    pub score: f32,
}

/// Session store backed by SQLite. Thread-safe via Mutex.
#[derive(Clone)]
pub struct SessionStore {
    db: Arc<Mutex<Connection>>,
}

impl std::fmt::Debug for SessionStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SessionStore").finish()
    }
}

impl SessionStore {
    /// Open (or create) a SQLite database at the given path.
    pub fn open(path: &str) -> anyhow::Result<Self> {
        // Ensure parent directory exists
        if let Some(parent) = Path::new(path).parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(path)?;

        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS sessions (
                 id TEXT PRIMARY KEY
             );
             CREATE TABLE IF NOT EXISTS messages (
                 id INTEGER PRIMARY KEY AUTOINCREMENT,
                 session_id TEXT NOT NULL,
                 role TEXT NOT NULL,
                 content TEXT NOT NULL,
                 created_at TEXT DEFAULT (datetime('now')),
                 FOREIGN KEY (session_id) REFERENCES sessions(id)
             );
             CREATE INDEX IF NOT EXISTS idx_messages_session
                 ON messages(session_id);

             CREATE TABLE IF NOT EXISTS memories (
                 id TEXT PRIMARY KEY,
                 user_id TEXT NOT NULL,
                 text TEXT NOT NULL,
                 embedding BLOB,
                 created_at TEXT DEFAULT (datetime('now')),
                 updated_at TEXT DEFAULT (datetime('now'))
             );
             CREATE INDEX IF NOT EXISTS idx_memories_user ON memories(user_id);

             CREATE TABLE IF NOT EXISTS memory_history (
                 id INTEGER PRIMARY KEY AUTOINCREMENT,
                 memory_id TEXT NOT NULL,
                 action TEXT NOT NULL,
                 old_text TEXT,
                 new_text TEXT,
                 created_at TEXT DEFAULT (datetime('now'))
             );",
        )?;

        info!(%path, "Session store opened");

        Ok(Self {
            db: Arc::new(Mutex::new(conn)),
        })
    }

    /// In-memory store (for tests or when persistence is disabled).
    pub fn in_memory() -> anyhow::Result<Self> {
        Self::open(":memory:")
    }

    /// Ensure a session exists.
    pub async fn get_or_create(&self, id: &str) {
        let db = self.db.lock().await;
        let _ = db.execute(
            "INSERT OR IGNORE INTO sessions (id) VALUES (?1)",
            [id],
        );
    }

    /// Create a new session with a random UUID, return the ID.
    pub async fn create(&self) -> String {
        let id = uuid::Uuid::new_v4().to_string();
        self.get_or_create(&id).await;
        id
    }

    /// Get all messages for a session.
    pub async fn get_history(&self, id: &str) -> Vec<Message> {
        let db = self.db.lock().await;
        let mut stmt = match db.prepare(
            "SELECT role, content FROM messages WHERE session_id = ?1 ORDER BY id ASC",
        ) {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };

        let rows = match stmt.query_map([id], |row| {
            Ok(Message {
                role: row.get(0)?,
                content: row.get(1)?,
            })
        }) {
            Ok(rows) => rows.filter_map(|r| r.ok()).collect(),
            Err(_) => Vec::new(),
        };

        rows
    }

    // ── Long-term memory (vector) ────────────────────────────────────

    pub async fn add_memory(&self, id: &str, user_id: &str, text: &str, embedding: &[f32]) -> anyhow::Result<()> {
        let db = self.db.lock().await;
        let blob = embedding_to_blob(embedding);
        db.execute(
            "INSERT INTO memories (id, user_id, text, embedding) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![id, user_id, text, blob],
        )?;
        db.execute(
            "INSERT INTO memory_history (memory_id, action, new_text) VALUES (?1, 'ADD', ?2)",
            rusqlite::params![id, text],
        )?;
        Ok(())
    }

    pub async fn update_memory(&self, id: &str, text: &str, embedding: &[f32]) -> anyhow::Result<()> {
        let db = self.db.lock().await;
        let blob = embedding_to_blob(embedding);
        let old: Option<String> = db.query_row("SELECT text FROM memories WHERE id = ?1", [id], |r| r.get(0)).ok();
        db.execute(
            "UPDATE memories SET text = ?1, embedding = ?2, updated_at = datetime('now') WHERE id = ?3",
            rusqlite::params![text, blob, id],
        )?;
        db.execute(
            "INSERT INTO memory_history (memory_id, action, old_text, new_text) VALUES (?1, 'UPDATE', ?2, ?3)",
            rusqlite::params![id, old, text],
        )?;
        Ok(())
    }

    pub async fn delete_memory(&self, id: &str) -> anyhow::Result<()> {
        let db = self.db.lock().await;
        let old: Option<String> = db.query_row("SELECT text FROM memories WHERE id = ?1", [id], |r| r.get(0)).ok();
        db.execute("DELETE FROM memories WHERE id = ?1", [id])?;
        db.execute(
            "INSERT INTO memory_history (memory_id, action, old_text) VALUES (?1, 'DELETE', ?2)",
            rusqlite::params![id, old],
        )?;
        Ok(())
    }

    /// Search memories by text query (needs embedding).
    pub async fn search_memories(&self, user_id: &str, query: &str) -> anyhow::Result<Vec<MemoryResult>> {
        let emb = crate::session::embed::embed(
            &crate::config::AgentConfig::default(),
            query,
        ).await?;
        self.search_memories_by_vec(user_id, &emb, 10).await
    }

    /// Search memories by pre-computed embedding vector.
    pub async fn search_memories_by_vec(&self, user_id: &str, query_emb: &[f32], limit: usize) -> anyhow::Result<Vec<MemoryResult>> {
        let db = self.db.lock().await;
        let mut stmt = db.prepare("SELECT id, text, embedding FROM memories WHERE user_id = ?1")?;

        let mut results: Vec<MemoryResult> = stmt
            .query_map([user_id], |row| {
                let id: String = row.get(0)?;
                let text: String = row.get(1)?;
                let blob: Vec<u8> = row.get(2)?;
                Ok((id, text, blob))
            })?
            .filter_map(|r| r.ok())
            .map(|(id, text, blob)| {
                let emb = blob_to_embedding(&blob);
                let score = crate::session::embed::cosine_similarity(query_emb, &emb);
                MemoryResult { id, text, score }
            })
            .filter(|r| r.score > 0.3) // minimum threshold
            .collect();

        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(limit);
        Ok(results)
    }

    // ── Short-term (conversation history) ────────────────────────────

    /// Append a message to a session (auto-creates session if needed).
    pub async fn push_message(&self, id: &str, msg: Message) {
        let db = self.db.lock().await;
        let _ = db.execute(
            "INSERT OR IGNORE INTO sessions (id) VALUES (?1)",
            [id],
        );
        let _ = db.execute(
            "INSERT INTO messages (session_id, role, content) VALUES (?1, ?2, ?3)",
            rusqlite::params![id, msg.role, msg.content],
        );
    }
}

fn embedding_to_blob(embedding: &[f32]) -> Vec<u8> {
    embedding.iter().flat_map(|f| f.to_le_bytes()).collect()
}

fn blob_to_embedding(blob: &[u8]) -> Vec<f32> {
    blob.chunks_exact(4)
        .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect()
}
