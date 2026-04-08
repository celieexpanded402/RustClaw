use anyhow::{Context, Result};
use reqwest::Client;

use crate::config::AgentConfig;

const FACT_EXTRACTION_PROMPT: &str = r#"You are a Personal Information Organizer. Extract distinct facts from the conversation.

Types: personal preferences, important details (names, dates), plans, health, professional info.
Extract ONLY from user messages. Ignore greetings and generic statements.

Return a JSON array of strings. If no facts, return [].
Examples:
- "Hi." → []
- "My name is John. I work at Google." → ["Name is John", "Works at Google"]

Respond with ONLY the JSON array."#;

const DEDUP_PROMPT: &str = r#"You are a smart memory manager.

For each new fact, decide:
- ADD: genuinely new info (new integer ID)
- UPDATE: existing memory with richer version (keep same ID, new text)
- DELETE: new fact CONTRADICTS existing (remove old)
- NONE: already covered

Existing memories:
{existing}

New facts:
{new_facts}

Respond ONLY: {"memory": [{"id": "<int>", "text": "<content>", "event": "ADD|UPDATE|DELETE|NONE"}]}"#;

const ENTITY_PROMPT: &str = r#"Extract entities from the text. If "I"/"me"/"my" appears, use "{user_id}" as the entity.

Return JSON: [{"entity": "name", "entity_type": "person|place|organization|other"}]
Respond with ONLY the JSON array."#;

const RELATION_PROMPT: &str = r#"Extract relationships between entities. Replace "I"/"me" with "{user_id}".

Entities: {entities}
Text: {text}

Return JSON: [{"source": "...", "relation": "...", "destination": "..."}]
Use lowercase: lives_in, works_at, likes, knows, etc.
Respond with ONLY the JSON array."#;

// ── Fact extraction ──────────────────────────────────────────────────

pub async fn extract_facts(config: &AgentConfig, text: &str) -> Result<Vec<String>> {
    let response = llm_call(config, FACT_EXTRACTION_PROMPT, text).await?;
    parse_string_array(&response)
}

// ── Deduplication ────────────────────────────────────────────────────

#[derive(Debug)]
pub struct DeduplicatedFact {
    pub fact: String,
    pub action: FactAction,
    pub existing_id: Option<String>,
}

#[derive(Debug)]
pub enum FactAction {
    Add,
    Update,
    Delete,
    None,
}

pub async fn deduplicate(
    config: &AgentConfig,
    existing: &[(String, String)],
    new_facts: &[String],
) -> Result<Vec<DeduplicatedFact>> {
    if new_facts.is_empty() {
        return Ok(Vec::new());
    }

    if existing.is_empty() {
        return Ok(new_facts
            .iter()
            .map(|f| DeduplicatedFact {
                fact: f.clone(),
                action: FactAction::Add,
                existing_id: None,
            })
            .collect());
    }

    let existing_str = existing
        .iter()
        .map(|(id, text)| format!("[{id}] {text}"))
        .collect::<Vec<_>>()
        .join("\n");

    let new_str = new_facts
        .iter()
        .enumerate()
        .map(|(i, f)| format!("{}. {f}", i + 1))
        .collect::<Vec<_>>()
        .join("\n");

    let prompt = DEDUP_PROMPT
        .replace("{existing}", &existing_str)
        .replace("{new_facts}", &new_str);

    let response = llm_call(config, &prompt, "Deduplicate.").await?;

    let parsed: serde_json::Value = serde_json::from_str(&response)
        .or_else(|_| {
            if let Some(s) = response.find('{') {
                if let Some(e) = response.rfind('}') {
                    return serde_json::from_str(&response[s..=e]);
                }
            }
            Ok(serde_json::json!({"memory": []}))
        })
        .unwrap_or(serde_json::json!({"memory": []}));

    let arr = parsed
        .get("memory")
        .and_then(|m| m.as_array())
        .cloned()
        .unwrap_or_default();

    let mut results = Vec::new();
    for item in arr {
        let fact = item.get("text").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let event = item.get("event").and_then(|v| v.as_str()).unwrap_or("ADD").to_uppercase();
        let id = item.get("id").and_then(|v| v.as_str().map(String::from).or_else(|| v.as_u64().map(|n| n.to_string())));

        let action = match event.as_str() {
            "UPDATE" => FactAction::Update,
            "DELETE" => FactAction::Delete,
            "NONE" => FactAction::None,
            _ => FactAction::Add,
        };

        if !fact.is_empty() || matches!(action, FactAction::Delete) {
            results.push(DeduplicatedFact { fact, action, existing_id: id });
        }
    }

    if results.is_empty() && !new_facts.is_empty() {
        results = new_facts.iter().map(|f| DeduplicatedFact {
            fact: f.clone(), action: FactAction::Add, existing_id: None,
        }).collect();
    }

    Ok(results)
}

// ── Entity / Relation extraction ─────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Entity {
    pub name: String,
    pub entity_type: String,
}

#[derive(Debug, Clone)]
pub struct ExtractedRelation {
    pub source: String,
    pub relation: String,
    pub destination: String,
}

pub async fn extract_entities(config: &AgentConfig, text: &str, user_id: &str) -> Result<Vec<Entity>> {
    let prompt = ENTITY_PROMPT.replace("{user_id}", user_id);
    let response = llm_call(config, &prompt, text).await?;
    let parsed = parse_value_array(&response);

    Ok(parsed.into_iter().filter_map(|item| {
        let name = item.get("entity")?.as_str()?.to_lowercase();
        let etype = item.get("entity_type").and_then(|v| v.as_str()).unwrap_or("other").to_lowercase();
        let resolved = match name.as_str() {
            "i" | "me" | "my" | "myself" | "我" | "我的" => user_id.to_lowercase(),
            _ => name,
        };
        Some(Entity { name: resolved, entity_type: etype })
    }).collect())
}

pub async fn extract_relations(config: &AgentConfig, text: &str, entities: &[Entity]) -> Result<Vec<ExtractedRelation>> {
    let names: Vec<&str> = entities.iter().map(|e| e.name.as_str()).collect();
    let prompt = RELATION_PROMPT
        .replace("{entities}", &format!("{names:?}"))
        .replace("{text}", text);

    let response = llm_call(config, &prompt, "Extract relations.").await?;
    let parsed = parse_value_array(&response);

    Ok(parsed.into_iter().filter_map(|item| {
        let source = item.get("source")?.as_str()?.to_lowercase();
        let relation = item.get("relation")?.as_str()?.to_lowercase().replace(' ', "_");
        let destination = item.get("destination")?.as_str()?.to_lowercase();
        if source.is_empty() || destination.is_empty() { return None; }
        Some(ExtractedRelation { source, relation, destination })
    }).collect())
}

// ── Helpers ──────────────────────────────────────────────────────────

async fn llm_call(config: &AgentConfig, system: &str, user: &str) -> Result<String> {
    let client = Client::new();
    let base = if config.base_url.is_empty() {
        "http://127.0.0.1:11434"
    } else {
        config.base_url.trim_end_matches('/')
    };
    let url = format!("{base}/v1/chat/completions");
    let model = if config.model.is_empty() { "qwen2.5:32b" } else { &config.model };

    let body = serde_json::json!({
        "model": model,
        "messages": [{"role": "system", "content": system}, {"role": "user", "content": user}],
        "temperature": 0.1,
    });

    let mut req = client.post(&url).header("content-type", "application/json");
    if !config.api_key.is_empty() && config.api_key != "ollama" {
        req = req.header("authorization", format!("Bearer {}", config.api_key));
    }

    let resp = req.json(&body).send().await.context("LLM request failed")?;
    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        anyhow::bail!("LLM error {status}: {text}");
    }

    let data: serde_json::Value = resp.json().await?;
    Ok(data.get("choices").and_then(|c| c.get(0)).and_then(|c| c.get("message")).and_then(|m| m.get("content")).and_then(|c| c.as_str()).unwrap_or("").to_string())
}

fn parse_string_array(s: &str) -> Result<Vec<String>> {
    if let Ok(arr) = serde_json::from_str::<Vec<String>>(s) { return Ok(arr); }
    if let Some(start) = s.find('[') {
        if let Some(end) = s.rfind(']') {
            if let Ok(arr) = serde_json::from_str::<Vec<String>>(&s[start..=end]) { return Ok(arr); }
        }
    }
    if let Ok(obj) = serde_json::from_str::<serde_json::Value>(s) {
        if let Some(facts) = obj.get("facts").and_then(|f| f.as_array()) {
            return Ok(facts.iter().filter_map(|v| v.as_str().map(String::from)).collect());
        }
    }
    Ok(Vec::new())
}

fn parse_value_array(s: &str) -> Vec<serde_json::Value> {
    if let Ok(arr) = serde_json::from_str::<Vec<serde_json::Value>>(s) { return arr; }
    if let Some(start) = s.find('[') {
        if let Some(end) = s.rfind(']') {
            if let Ok(arr) = serde_json::from_str::<Vec<serde_json::Value>>(&s[start..=end]) { return arr; }
        }
    }
    Vec::new()
}
