use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::extract::ws::{Message, WebSocket};
use tokio_stream::StreamExt;
use tracing::{info, warn};
use uuid::Uuid;

use crate::agent;

use super::protocol::*;
use super::server::AppState;

/// Handle a single WebSocket connection through its full lifecycle.
pub async fn handle(mut socket: WebSocket, state: Arc<AppState>) {
    let peer = "client";
    info!(%peer, "WebSocket connection opened");

    // ── Phase 1: Expect connect request ──────────────────────────────
    let connect_req = match read_frame(&mut socket).await {
        Some(InboundFrame::Connect(req)) => req,
        Some(_) => {
            send(&mut socket, OutboundFrame::error(4001, "expected connect frame")).await;
            return;
        }
        None => return,
    };

    // ── Phase 2: Send challenge ──────────────────────────────────────
    let nonce = Uuid::new_v4().to_string();
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    send(&mut socket, OutboundFrame::connect_challenge(&nonce, ts)).await;

    // ── Phase 3: Expect auth response ────────────────────────────────
    let _auth = match read_frame(&mut socket).await {
        Some(InboundFrame::Auth(auth)) => auth,
        Some(_) => {
            send(&mut socket, OutboundFrame::error(4002, "expected auth frame")).await;
            return;
        }
        None => return,
    };

    // Validate token if configured
    if let Some(ref expected) = state.config.gateway.token {
        let client_token = connect_req
            .params
            .auth
            .as_ref()
            .and_then(|a| a.token.as_deref())
            .unwrap_or("");
        if client_token != expected {
            send(&mut socket, OutboundFrame::error(4003, "authentication failed")).await;
            return;
        }
    }

    // ── Phase 4: Send hello-ok ───────────────────────────────────────
    let device_token = Uuid::new_v4().to_string();
    send(&mut socket, OutboundFrame::hello_ok(&device_token)).await;
    info!(%peer, "Handshake complete");

    // Create a session for this connection
    let session_id = state.memory.create().await;

    // ── Phase 5: Request / response loop ─────────────────────────────
    while let Some(frame) = read_frame(&mut socket).await {
        match frame {
            InboundFrame::Req(req) => {
                handle_req(&mut socket, &state, &session_id, req).await;
            }
            other => {
                warn!(?other, "Unexpected frame after handshake");
                send(
                    &mut socket,
                    OutboundFrame::error(4010, "unexpected frame type"),
                )
                .await;
            }
        }
    }

    info!(%peer, "WebSocket connection closed");
}

/// Route a request frame by method.
async fn handle_req(
    socket: &mut WebSocket,
    state: &AppState,
    session_id: &str,
    req: ReqFrame,
) {
    match req.method.as_str() {
        "agent" => handle_agent(socket, state, session_id, req).await,
        "send" => handle_send(socket, req).await,
        "health" => handle_health(socket, req).await,
        _ => {
            send(
                socket,
                OutboundFrame::error_with_id(&req.id, 4040, format!("unknown method: {}", req.method)),
            )
            .await;
        }
    }
}

/// Handle method="agent": accept, stream tokens, done.
async fn handle_agent(
    socket: &mut WebSocket,
    state: &AppState,
    session_id: &str,
    req: ReqFrame,
) {
    let params: AgentParams = match serde_json::from_value(req.params) {
        Ok(p) => p,
        Err(e) => {
            send(
                socket,
                OutboundFrame::error_with_id(&req.id, 4000, format!("bad agent params: {e}")),
            )
            .await;
            return;
        }
    };

    let run_id = Uuid::new_v4().to_string();

    // ACK
    send(socket, OutboundFrame::agent_ack(&req.id, &run_id)).await;

    // Get session history + recall long-term memories
    let history = state.memory.get_history(session_id).await;
    let recalled = state.memory.recall(session_id, &params.input).await;
    let input_with_memory = if recalled.is_empty() {
        params.input.clone()
    } else {
        format!("[Memory]\n{recalled}\n\n{}", params.input)
    };
    let mut rx = state.agent.chat_stream(&input_with_memory, &history).await;

    // Record user message
    state
        .memory
        .push_message(
            session_id,
            agent::Message {
                role: "user".to_string(),
                content: params.input.clone(),
            },
        )
        .await;

    let mut full_response = String::new();
    while let Some(token) = rx.recv().await {
        full_response.push_str(&token);
        send(socket, OutboundFrame::agent_event_delta(&run_id, &token)).await;
    }

    // Record assistant response
    state
        .memory
        .push_message(
            session_id,
            agent::Message {
                role: "assistant".to_string(),
                content: full_response.clone(),
            },
        )
        .await;

    // Extract long-term memories from this exchange
    state.memory.learn(session_id, &params.input, &full_response).await;

    // Done
    send(socket, OutboundFrame::agent_event_done(&run_id)).await;
}

/// Handle method="send": stub, just acknowledge.
async fn handle_send(socket: &mut WebSocket, req: ReqFrame) {
    let params: SendParams = match serde_json::from_value(req.params) {
        Ok(p) => p,
        Err(e) => {
            send(
                socket,
                OutboundFrame::error_with_id(&req.id, 4000, format!("bad send params: {e}")),
            )
            .await;
            return;
        }
    };

    info!(
        channel = %params.channel,
        to = %params.to,
        "Send request (stub)"
    );

    send(
        socket,
        OutboundFrame::res_ok(&req.id, serde_json::json!({ "sent": true })),
    )
    .await;
}

/// Handle method="health": respond immediately.
async fn handle_health(socket: &mut WebSocket, req: ReqFrame) {
    send(
        socket,
        OutboundFrame::res_ok(&req.id, serde_json::json!({ "status": "ok" })),
    )
    .await;
}

// ── Helpers ──────────────────────────────────────────────────────────

/// Read and parse a single inbound JSON frame, returning None on close/error.
async fn read_frame(socket: &mut WebSocket) -> Option<InboundFrame> {
    loop {
        match socket.next().await? {
            Ok(Message::Text(text)) => {
                match serde_json::from_str::<InboundFrame>(&text) {
                    Ok(frame) => return Some(frame),
                    Err(e) => {
                        warn!(%e, "Malformed JSON frame");
                        let _ = socket
                            .send(Message::Text(
                                OutboundFrame::error(4000, format!("malformed frame: {e}"))
                                    .to_json(),
                            ))
                            .await;
                    }
                }
            }
            Ok(Message::Close(_)) => return None,
            Ok(Message::Ping(_)) | Ok(Message::Pong(_)) => continue,
            Ok(_) => continue,
            Err(e) => {
                warn!(%e, "WebSocket read error");
                return None;
            }
        }
    }
}

/// Serialize and send an outbound frame.
async fn send(socket: &mut WebSocket, frame: OutboundFrame) {
    let json = frame.to_json();
    if let Err(e) = socket.send(Message::Text(json)).await {
        warn!(%e, "WebSocket send error");
    }
}
