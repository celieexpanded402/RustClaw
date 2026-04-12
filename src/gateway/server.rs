use std::sync::Arc;

use axum::{
    extract::{State, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
    Router,
};
use tracing::info;

use crate::agent::AgentRunner;
use crate::config::AppConfig;
use crate::session::memory::MemoryManager;

use super::connection;

/// Shared state available to every WebSocket connection.
pub struct AppState {
    pub config: AppConfig,
    pub agent: AgentRunner,
    pub memory: MemoryManager,
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| connection::handle(socket, state))
}

async fn health() -> &'static str {
    "ok"
}

pub async fn run_with_memory(config: AppConfig, memory: MemoryManager) -> anyhow::Result<()> {
    let listen = config.gateway.listen_addr();

    match &config.gateway.token {
        Some(t) if !t.is_empty() => info!("Gateway authentication enabled"),
        _ => tracing::warn!("Gateway has NO authentication token — any client can connect. Set [gateway] token in config for production use."),
    }

    let agent = AgentRunner::new(config.agent.clone());

    let state = Arc::new(AppState {
        config,
        agent,
        memory,
    });

    let app = Router::new()
        .route("/ws", get(ws_handler))
        .route("/health", get(health))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&listen).await?;
    info!("Listening on {listen}");

    axum::serve(listener, app).await?;

    Ok(())
}
