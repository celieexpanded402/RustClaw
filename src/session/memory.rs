use anyhow::Result;
use tracing::{info, warn};
use uuid::Uuid;

use crate::agent::Message;
use crate::config::AgentConfig;
use crate::session::embed;
use crate::session::extract::{self, FactAction};
use crate::session::graph::GraphStore;
use crate::session::store::SessionStore;

/// Manages both conversation history (short-term) and extracted memories (long-term + graph).
#[derive(Clone)]
pub struct MemoryManager {
    pub sessions: SessionStore,
    graph: GraphStore,
    agent_config: AgentConfig,
}

impl MemoryManager {
    pub fn new(sessions: SessionStore, graph: GraphStore, agent_config: AgentConfig) -> Self {
        Self { sessions, graph, agent_config }
    }

    // ── Short-term (conversation history) — delegate to SessionStore ─

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

    // ── Long-term memory: extract + store after conversation ─────────

    /// Process a completed exchange and extract long-term memories.
    /// Call this AFTER the assistant has responded.
    pub async fn learn(&self, session_id: &str, user_text: &str, assistant_text: &str) {
        let conversation = format!("User: {user_text}\nAssistant: {assistant_text}");

        // Extract facts (non-blocking — don't fail the response)
        tokio::spawn({
            let config = self.agent_config.clone();
            let sessions = self.sessions.clone();
            let graph = self.graph.clone();
            let sid = session_id.to_string();
            let conv = conversation.clone();
            let user_text = user_text.to_string();

            async move {
                if let Err(e) = learn_inner(&config, &sessions, &graph, &sid, &conv, &user_text).await {
                    warn!(%e, "Memory extraction failed (non-fatal)");
                }
            }
        });
    }

    // ── Long-term memory: search for relevant context ────────────────

    /// Search for memories relevant to the current query.
    /// Returns a formatted string to inject into the system prompt.
    pub async fn recall(&self, session_id: &str, query: &str) -> String {
        let mut context_parts = Vec::new();

        // Vector memory search
        match self.sessions.search_memories(session_id, query).await {
            Ok(memories) if !memories.is_empty() => {
                let mem_str = memories.iter()
                    .map(|m| format!("- {}", m.text))
                    .collect::<Vec<_>>()
                    .join("\n");
                context_parts.push(format!("Known facts about this user:\n{mem_str}"));
            }
            _ => {}
        }

        // Graph search
        match self.graph.search(session_id, query).await {
            Ok(relations) if !relations.is_empty() => {
                let rel_str = relations.iter()
                    .map(|r| format!("- {} {} {}", r.source, r.relation, r.destination))
                    .collect::<Vec<_>>()
                    .join("\n");
                context_parts.push(format!("Known relationships:\n{rel_str}"));
            }
            _ => {}
        }

        context_parts.join("\n\n")
    }
}

/// Internal: extract facts, deduplicate, store.
async fn learn_inner(
    config: &AgentConfig,
    sessions: &SessionStore,
    graph: &GraphStore,
    session_id: &str,
    conversation: &str,
    user_text: &str,
) -> Result<()> {
    // Step 1: Extract facts
    let facts = extract::extract_facts(config, conversation).await?;
    if facts.is_empty() {
        return Ok(());
    }
    info!(count = facts.len(), "Extracted facts from conversation");

    // Step 2: Embed each fact, search for similar existing memories
    let mut all_existing: Vec<(String, String)> = Vec::new();
    let mut seen_ids = std::collections::HashSet::new();

    for fact in &facts {
        if let Ok(emb) = embed::embed(config, fact).await {
            if let Ok(similar) = sessions.search_memories_by_vec(session_id, &emb, 5).await {
                for s in similar {
                    if seen_ids.insert(s.id.clone()) {
                        all_existing.push((s.id, s.text));
                    }
                }
            }
        }
    }

    // Step 3: Integer ID mapping
    let mut uuid_map: Vec<(String, String)> = Vec::new();
    let existing_for_llm: Vec<(String, String)> = all_existing.iter().enumerate()
        .map(|(i, (uuid, text))| {
            uuid_map.push((i.to_string(), uuid.clone()));
            (i.to_string(), text.clone())
        }).collect();

    // Step 4: LLM dedup decision
    let decisions = extract::deduplicate(config, &existing_for_llm, &facts).await?;

    // Step 5: Execute
    for d in decisions {
        match d.action {
            FactAction::Add => {
                let id = Uuid::new_v4().to_string();
                if let Ok(emb) = embed::embed(config, &d.fact).await {
                    let _ = sessions.add_memory(&id, session_id, &d.fact, &emb).await;
                    info!("Memory ADD: {}", d.fact);
                }
            }
            FactAction::Update => {
                if let Some(ref int_id) = d.existing_id {
                    let real_id = uuid_map.iter().find(|(k, _)| k == int_id).map(|(_, v)| v.clone()).unwrap_or_else(|| int_id.clone());
                    if let Ok(emb) = embed::embed(config, &d.fact).await {
                        let _ = sessions.update_memory(&real_id, &d.fact, &emb).await;
                        info!("Memory UPDATE: {}", d.fact);
                    }
                }
            }
            FactAction::Delete => {
                if let Some(ref int_id) = d.existing_id {
                    let real_id = uuid_map.iter().find(|(k, _)| k == int_id).map(|(_, v)| v.clone()).unwrap_or_else(|| int_id.clone());
                    let _ = sessions.delete_memory(&real_id).await;
                    info!("Memory DELETE: {}", d.fact);
                }
            }
            FactAction::None => {}
        }
    }

    // Step 6: Graph extraction
    if let Ok(entities) = extract::extract_entities(config, user_text, session_id).await {
        if !entities.is_empty() {
            if let Ok(relations) = extract::extract_relations(config, user_text, &entities).await {
                for rel in &relations {
                    let _ = graph.add_relation(session_id, &rel.source, &rel.relation, &rel.destination).await;
                }
                info!(entities = entities.len(), relations = relations.len(), "Graph updated");
            }
        }
    }

    Ok(())
}
