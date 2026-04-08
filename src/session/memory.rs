use anyhow::Result;
use tracing::{info, warn};
use uuid::Uuid;

use crate::agent::Message;
use crate::config::AgentConfig;
use crate::session::embed;
use crate::session::extract::{self, FactAction};
use crate::session::graph::GraphStore;
use crate::session::store::SessionStore;

/// Three memory scopes:
/// - Local:  per session (telegram:12345, discord:ch:user)
/// - User:   per real person across channels (user:{user_id})
/// - Global: shared by everyone (global:system)
const GLOBAL_SCOPE: &str = "global:system";

/// Manages three-tier memory: short-term history + long-term vector + graph.
/// Supports mixed-mode recall: local + user + global scopes merged.
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

    // ── Mixed-mode recall ────────────────────────────────────────────

    /// Search for memories across three scopes:
    /// 1. Local (this session_id) — group/channel specific
    /// 2. User (extracted from session_id) — personal across channels
    /// 3. Global — shared system knowledge
    pub async fn recall(&self, session_id: &str, query: &str) -> String {
        let user_scope = extract_user_scope(session_id);
        let scopes = [session_id, &user_scope, GLOBAL_SCOPE];

        let mut facts = Vec::new();
        let mut relations = Vec::new();
        let mut seen = std::collections::HashSet::new();

        for scope in &scopes {
            // Vector memory search
            if let Ok(memories) = self.sessions.search_memories(scope, query).await {
                for m in memories {
                    if seen.insert(m.text.clone()) {
                        facts.push(m.text);
                    }
                }
            }

            // Graph search
            if let Ok(rels) = self.graph.search(scope, query).await {
                for r in rels {
                    let triple = format!("{} {} {}", r.source, r.relation, r.destination);
                    if seen.insert(triple.clone()) {
                        relations.push(triple);
                    }
                }
            }
        }

        let mut parts = Vec::new();
        if !facts.is_empty() {
            let s = facts.iter().map(|f| format!("- {f}")).collect::<Vec<_>>().join("\n");
            parts.push(format!("Known facts:\n{s}"));
        }
        if !relations.is_empty() {
            let s = relations.iter().map(|r| format!("- {r}")).collect::<Vec<_>>().join("\n");
            parts.push(format!("Known relationships:\n{s}"));
        }

        parts.join("\n\n")
    }

    // ── Learn: extract + store to multiple scopes ────────────────────

    /// Process a completed exchange. Stores memories to:
    /// - Local scope (session_id) — context-specific facts
    /// - User scope (user:{id}) — personal facts about the user
    /// - Global scope — system-level knowledge (if detected)
    /// - Graph — entity relations to user scope
    pub async fn learn(&self, session_id: &str, user_text: &str, assistant_text: &str) {
        let conversation = format!("User: {user_text}\nAssistant: {assistant_text}");

        tokio::spawn({
            let config = self.agent_config.clone();
            let sessions = self.sessions.clone();
            let graph = self.graph.clone();
            let sid = session_id.to_string();
            let conv = conversation;
            let utxt = user_text.to_string();

            async move {
                if let Err(e) = learn_inner(&config, &sessions, &graph, &sid, &conv, &utxt).await {
                    warn!(%e, "Memory extraction failed (non-fatal)");
                }
            }
        });
    }
}

/// Extract user scope from session_id.
/// "telegram:12345" → "user:12345"
/// "discord:ch123:user456" → "user:user456"
/// "some-uuid" → "user:some-uuid"
fn extract_user_scope(session_id: &str) -> String {
    let parts: Vec<&str> = session_id.split(':').collect();
    match parts.len() {
        3 => format!("user:{}", parts[2]),  // discord:channel:user
        2 => format!("user:{}", parts[1]),  // telegram:user
        _ => format!("user:{session_id}"),  // gateway UUID
    }
}

/// Internal: extract facts, deduplicate, store to appropriate scopes.
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

    let user_scope = extract_user_scope(session_id);

    // Step 2: For each fact, decide scope + dedup against user scope
    let mut all_existing: Vec<(String, String)> = Vec::new();
    let mut seen_ids = std::collections::HashSet::new();

    // Search both local and user scope for dedup
    for fact in &facts {
        if let Ok(emb) = embed::embed(config, fact).await {
            for scope in [session_id, user_scope.as_str()] {
                if let Ok(similar) = sessions.search_memories_by_vec(scope, &emb, 5).await {
                    for s in similar {
                        if seen_ids.insert(s.id.clone()) {
                            all_existing.push((s.id, s.text));
                        }
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

    // Step 4: LLM dedup
    let decisions = extract::deduplicate(config, &existing_for_llm, &facts).await?;

    // Step 5: Execute — store personal facts to user scope, context facts to local
    for d in decisions {
        let target_scope = if is_personal_fact(&d.fact) {
            &user_scope
        } else {
            session_id
        };

        match d.action {
            FactAction::Add => {
                let id = Uuid::new_v4().to_string();
                if let Ok(emb) = embed::embed(config, &d.fact).await {
                    let _ = sessions.add_memory(&id, target_scope, &d.fact, &emb).await;
                    info!(scope = %target_scope, "Memory ADD: {}", d.fact);
                }
            }
            FactAction::Update => {
                if let Some(ref int_id) = d.existing_id {
                    let real_id = uuid_map.iter()
                        .find(|(k, _)| k == int_id)
                        .map(|(_, v)| v.clone())
                        .unwrap_or_else(|| int_id.clone());
                    if let Ok(emb) = embed::embed(config, &d.fact).await {
                        let _ = sessions.update_memory(&real_id, &d.fact, &emb).await;
                        info!("Memory UPDATE: {}", d.fact);
                    }
                }
            }
            FactAction::Delete => {
                if let Some(ref int_id) = d.existing_id {
                    let real_id = uuid_map.iter()
                        .find(|(k, _)| k == int_id)
                        .map(|(_, v)| v.clone())
                        .unwrap_or_else(|| int_id.clone());
                    let _ = sessions.delete_memory(&real_id).await;
                    info!("Memory DELETE: {}", d.fact);
                }
            }
            FactAction::None => {}
        }
    }

    // Step 6: Graph — always store to user scope (personal knowledge graph)
    if let Ok(entities) = extract::extract_entities(config, user_text, &user_scope).await {
        if !entities.is_empty() {
            if let Ok(relations) = extract::extract_relations(config, user_text, &entities).await {
                for rel in &relations {
                    let _ = graph.add_relation(&user_scope, &rel.source, &rel.relation, &rel.destination).await;
                }
                info!(entities = entities.len(), relations = relations.len(), "Graph updated (user scope)");
            }
        }
    }

    Ok(())
}

/// Heuristic: is this fact about the user personally (→ user scope)
/// or about the current context (→ local scope)?
fn is_personal_fact(fact: &str) -> bool {
    let lower = fact.to_lowercase();
    // Personal indicators
    let personal = [
        "name is", "name:", "叫", "名字",
        "lives in", "住", "lives at",
        "works at", "工作", "job", "職業",
        "likes", "loves", "hates", "prefers", "喜歡", "討厭", "偏好",
        "born", "birthday", "age", "年齡", "生日",
        "married", "wife", "husband", "family", "家",
        "speaks", "language", "語言",
        "email", "phone", "信箱", "電話",
        "allergic", "diet", "health", "過敏",
    ];
    personal.iter().any(|p| lower.contains(p))
}
