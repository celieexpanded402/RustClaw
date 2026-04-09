use std::path::Path;
use std::sync::Arc;

use rusqlite::Connection;
use tokio::sync::Mutex;
use tracing::info;

use crate::agent::Message;

/// Session store for conversation history (short-term).
/// Long-term memory (vector + graph) delegated to rustmem crate.
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
    pub fn open(path: &str) -> anyhow::Result<Self> {
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
                 ON messages(session_id);",
        )?;

        info!(%path, "Session store opened");

        Ok(Self {
            db: Arc::new(Mutex::new(conn)),
        })
    }

    pub async fn get_or_create(&self, id: &str) {
        let db = self.db.lock().await;
        let _ = db.execute(
            "INSERT OR IGNORE INTO sessions (id) VALUES (?1)",
            [id],
        );
    }

    pub async fn create(&self) -> String {
        let id = uuid::Uuid::new_v4().to_string();
        self.get_or_create(&id).await;
        id
    }

    pub async fn get_history(&self, id: &str) -> Vec<Message> {
        let db = self.db.lock().await;
        let mut results = Vec::new();
        if let Ok(mut stmt) = db.prepare(
            "SELECT role, content FROM messages WHERE session_id = ?1 ORDER BY id ASC",
        ) {
            if let Ok(rows) = stmt.query_map([id], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            }) {
                for row in rows.flatten() {
                    results.push(Message { role: row.0, content: row.1 });
                }
            }
        }
        results
    }

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
