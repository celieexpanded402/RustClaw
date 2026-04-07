use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::agent::Message;

/// A single session's state.
#[derive(Debug, Clone)]
pub struct Session {
    pub id: String,
    pub history: Vec<Message>,
}

/// In-memory session store.
#[derive(Debug, Clone, Default)]
pub struct SessionStore {
    sessions: Arc<RwLock<HashMap<String, Session>>>,
}

impl SessionStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn create(&self) -> String {
        let id = Uuid::new_v4().to_string();
        let session = Session {
            id: id.clone(),
            history: Vec::new(),
        };
        self.sessions.write().await.insert(id.clone(), session);
        id
    }

    /// Get or create a session with a specific ID (e.g. "telegram:12345").
    pub async fn get_or_create(&self, id: &str) -> Session {
        let mut sessions = self.sessions.write().await;
        sessions
            .entry(id.to_string())
            .or_insert_with(|| Session {
                id: id.to_string(),
                history: Vec::new(),
            })
            .clone()
    }

    pub async fn get_history(&self, id: &str) -> Vec<Message> {
        self.sessions
            .read()
            .await
            .get(id)
            .map(|s| s.history.clone())
            .unwrap_or_default()
    }

    pub async fn push_message(&self, id: &str, msg: Message) {
        let mut sessions = self.sessions.write().await;
        let session = sessions
            .entry(id.to_string())
            .or_insert_with(|| Session {
                id: id.to_string(),
                history: Vec::new(),
            });
        session.history.push(msg);
    }
}
