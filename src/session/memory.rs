use std::sync::Arc;

use anyhow::Result;
use tracing::{info, warn};

use crate::agent::Message;
use crate::session::store::SessionStore;

const GLOBAL_SCOPE: &str = "global:system";

/// Manages short-term history (SessionStore) + long-term memory (rustmem).
#[derive(Clone)]
pub struct MemoryManager {
    pub sessions: SessionStore,
    rmem: Arc<rustmem::MemoryManager>,
}

impl MemoryManager {
    pub async fn new(sessions: SessionStore, db_path: &str, llm_config: &crate::config::AgentConfig) -> Result<Self> {
        // Build rustmem config from our agent config
        let rmem_config = rustmem::config::AppConfig {
            llm: rustmem::config::LlmConfig {
                provider: llm_config.provider.clone(),
                api_key: llm_config.api_key.clone(),
                base_url: llm_config.base_url.clone(),
                model: llm_config.model.clone(),
            },
            embedding: rustmem::config::EmbeddingConfig {
                provider: "openai".to_string(),
                api_key: String::new(),
                base_url: llm_config.base_url.clone(),
                model: "nomic-embed-text".to_string(),
                dimensions: 768,
            },
            store: rustmem::config::StoreConfig {
                db_path: db_path.to_string(),
            },
            ..Default::default()
        };

        let rmem = Arc::new(rustmem::MemoryManager::new(&rmem_config).await?);
        info!("R-Mem memory manager initialized");

        Ok(Self { sessions, rmem })
    }

    // ── Short-term (conversation history) ────────────────────────────

    pub async fn get_or_create(&self, id: &str) {
        self.sessions.get_or_create(id).await;
    }

    pub async fn create(&self) -> String {
        self.sessions.create().await
    }

    pub async fn get_history(&self, id: &str) -> Vec<Message> {
        self.sessions.get_history(id).await
    }

    pub async fn push_message(&self, id: &str, msg: Message) {
        self.sessions.push_message(id, msg).await;
    }

    // ── Mixed-mode recall (local + user + global) ────────────────────

    /// Search for memories across three scopes, returns formatted context string.
    pub async fn recall(&self, session_id: &str, query: &str) -> String {
        let user_scope = extract_user_scope(session_id);
        let scopes = [session_id, &user_scope, GLOBAL_SCOPE];

        let mut facts = Vec::new();
        let mut seen = std::collections::HashSet::new();

        for scope in &scopes {
            if let Ok(results) = self.rmem.search(scope, query, 10).await {
                for r in results {
                    if seen.insert(r.text.clone()) {
                        facts.push(r.text);
                    }
                }
            }
        }

        if facts.is_empty() {
            return String::new();
        }

        let s = facts.iter().map(|f| format!("- {f}")).collect::<Vec<_>>().join("\n");
        format!("Known facts:\n{s}")
    }

    // ── Learn: extract + store to appropriate scopes ─────────────────

    /// Process a completed exchange. Non-blocking (spawns background task).
    pub async fn learn(&self, session_id: &str, user_text: &str, _assistant_text: &str) {
        let rmem = Arc::clone(&self.rmem);
        let user_scope = extract_user_scope(session_id);
        let local_scope = session_id.to_string();
        let text = user_text.to_string();

        tokio::spawn(async move {
            // Store to user scope (personal facts persist across channels)
            if let Err(e) = rmem.add(&user_scope, &text).await {
                warn!(%e, "R-Mem learn failed for user scope");
            } else {
                info!(scope = %user_scope, "R-Mem learned from conversation");
            }
        });
    }
}

/// Extract user scope from session_id.
fn extract_user_scope(session_id: &str) -> String {
    let parts: Vec<&str> = session_id.split(':').collect();
    match parts.len() {
        3 => format!("user:{}", parts[2]),  // discord:channel:user
        2 => format!("user:{}", parts[1]),  // telegram:user
        _ => format!("user:{session_id}"),  // gateway UUID
    }
}
