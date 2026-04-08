use anyhow::{Context, Result};
use rusqlite::params;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone, serde::Serialize)]
pub struct Relation {
    pub source: String,
    pub relation: String,
    pub destination: String,
    pub mentions: i64,
}

/// SQLite graph store with soft-delete.
#[derive(Clone)]
pub struct GraphStore {
    db: Arc<Mutex<rusqlite::Connection>>,
}

impl GraphStore {
    pub fn open(path: &str) -> Result<Self> {
        let conn = rusqlite::Connection::open(path).context("Failed to open graph DB")?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS entities (
                 id INTEGER PRIMARY KEY AUTOINCREMENT,
                 user_id TEXT NOT NULL,
                 name TEXT NOT NULL,
                 mentions INTEGER DEFAULT 1,
                 UNIQUE(user_id, name)
             );
             CREATE TABLE IF NOT EXISTS relations (
                 id INTEGER PRIMARY KEY AUTOINCREMENT,
                 user_id TEXT NOT NULL,
                 source TEXT NOT NULL,
                 relation TEXT NOT NULL,
                 destination TEXT NOT NULL,
                 mentions INTEGER DEFAULT 1,
                 valid INTEGER DEFAULT 1,
                 updated_at TEXT DEFAULT (datetime('now')),
                 UNIQUE(user_id, source, relation, destination)
             );
             CREATE INDEX IF NOT EXISTS idx_rel_user ON relations(user_id);",
        )?;
        Ok(Self { db: Arc::new(Mutex::new(conn)) })
    }

    pub async fn add_relation(&self, user_id: &str, source: &str, relation: &str, destination: &str) -> Result<()> {
        let db = self.db.lock().await;

        // Upsert entities
        for name in [source, destination] {
            db.execute(
                "INSERT INTO entities (user_id, name) VALUES (?1, ?2)
                 ON CONFLICT(user_id, name) DO UPDATE SET mentions = mentions + 1",
                params![user_id, name],
            )?;
        }

        // Soft-delete conflicting single-value relations
        if !is_multi_value(relation) {
            db.execute(
                "UPDATE relations SET valid = 0 WHERE user_id = ?1 AND source = ?2 AND relation = ?3 AND destination != ?4 AND valid = 1",
                params![user_id, source, relation, destination],
            )?;
        }

        // Upsert relation
        db.execute(
            "INSERT INTO relations (user_id, source, relation, destination)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(user_id, source, relation, destination)
             DO UPDATE SET mentions = mentions + 1, valid = 1, updated_at = datetime('now')",
            params![user_id, source, relation, destination],
        )?;

        Ok(())
    }

    pub async fn search(&self, user_id: &str, query: &str) -> Result<Vec<Relation>> {
        let db = self.db.lock().await;
        let words: Vec<String> = query.split_whitespace()
            .map(|w| w.to_lowercase().replace(['，', '。', '？', '！', ',', '.'], ""))
            .filter(|w| w.len() > 1)
            .collect();

        if words.is_empty() { return Ok(Vec::new()); }

        let mut conditions = Vec::new();
        let mut qparams: Vec<String> = vec![user_id.to_string()];

        for word in &words {
            let idx = qparams.len();
            qparams.push(format!("%{word}%"));
            conditions.push(format!("(LOWER(source) LIKE ?{} OR LOWER(destination) LIKE ?{})", idx + 1, idx + 1));
        }

        let sql = format!(
            "SELECT source, relation, destination, mentions FROM relations WHERE user_id = ?1 AND valid = 1 AND ({}) ORDER BY mentions DESC LIMIT 10",
            conditions.join(" OR ")
        );

        let mut stmt = db.prepare(&sql)?;
        let refs: Vec<&dyn rusqlite::types::ToSql> = qparams.iter().map(|s| s as &dyn rusqlite::types::ToSql).collect();

        let rows = stmt.query_map(refs.as_slice(), |row| {
            Ok(Relation { source: row.get(0)?, relation: row.get(1)?, destination: row.get(2)?, mentions: row.get(3)? })
        })?.filter_map(|r| r.ok()).collect();

        Ok(rows)
    }

    pub async fn reset(&self, user_id: &str) -> Result<()> {
        let db = self.db.lock().await;
        db.execute("DELETE FROM relations WHERE user_id = ?1", [user_id])?;
        db.execute("DELETE FROM entities WHERE user_id = ?1", [user_id])?;
        Ok(())
    }
}

fn is_multi_value(relation: &str) -> bool {
    let lower = relation.to_lowercase();
    ["likes", "loves", "enjoys", "uses", "knows", "has", "plays", "watches", "reads", "speaks"]
        .iter().any(|m| lower.contains(m))
}
