use std::path::Path;
use std::sync::Arc;

use rusqlite::Connection;
use tokio::sync::Mutex;
use tracing::info;

use crate::agent::Message;

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
                 ON messages(session_id);",
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
