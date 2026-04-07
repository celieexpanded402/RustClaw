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
use crate::session::store::SessionStore;

use super::connection;

/// Shared state available to every WebSocket connection.
pub struct AppState {
    pub config: AppConfig,
    pub agent: AgentRunner,
    pub sessions: SessionStore,
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

/// Start the gateway server with a pre-existing session store (shared with channels).
pub async fn run_with_sessions(config: AppConfig, sessions: SessionStore) -> anyhow::Result<()> {
    let listen = config.gateway.listen_addr();
    let agent = AgentRunner::new(config.agent.clone());

    let state = Arc::new(AppState {
        config,
        agent,
        sessions,
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
