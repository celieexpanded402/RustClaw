use serde::{Deserialize, Serialize};
use serde_json::Value;

// ── Inbound (client → server) ────────────────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum InboundFrame {
    #[serde(rename = "connect")]
    Connect(ConnectRequest),
    #[serde(rename = "auth")]
    Auth(AuthResponse),
    #[serde(rename = "req")]
    Req(ReqFrame),
}

#[derive(Debug, Deserialize)]
pub struct ConnectRequest {
    pub params: ConnectParams,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct ConnectParams {
    #[serde(default)]
    pub auth: Option<AuthParam>,
    #[serde(default)]
    pub role: Option<String>,
    #[serde(default)]
    pub min_protocol: Option<u32>,
    #[serde(default)]
    pub max_protocol: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct AuthParam {
    #[serde(default)]
    pub token: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct AuthResponse {
    pub nonce: String,
    pub token: String,
}

#[derive(Debug, Deserialize)]
pub struct ReqFrame {
    pub id: String,
    pub method: String,
    #[serde(default)]
    pub params: Value,
}

// ── Req params ───────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct AgentParams {
    pub input: String,
    #[serde(default)]
    pub workspace: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct SendParams {
    pub channel: String,
    pub to: String,
    pub text: String,
}

// ── Outbound (server → client) ───────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct OutboundFrame {
    pub r#type: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<Value>,
    // agent event fields
    #[serde(rename = "runId", skip_serializing_if = "Option::is_none")]
    pub run_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delta: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub done: Option<bool>,
}

impl OutboundFrame {
    pub fn connect_challenge(nonce: &str, ts: u64) -> Self {
        Self {
            r#type: "event",
            event: Some("connect.challenge"),
            payload: Some(serde_json::json!({ "nonce": nonce, "ts": ts })),
            ..Self::empty()
        }
    }

    pub fn hello_ok(device_token: &str) -> Self {
        Self {
            r#type: "res",
            payload: Some(serde_json::json!({
                "hello": "ok",
                "auth": { "deviceToken": device_token },
                "snapshot": {}
            })),
            ..Self::empty()
        }
    }

    pub fn agent_ack(req_id: &str, run_id: &str) -> Self {
        Self {
            r#type: "res",
            id: Some(req_id.to_string()),
            status: Some("accepted"),
            run_id: Some(run_id.to_string()),
            ..Self::empty()
        }
    }

    pub fn agent_event_delta(run_id: &str, delta: &str) -> Self {
        Self {
            r#type: "event",
            event: Some("agent"),
            run_id: Some(run_id.to_string()),
            delta: Some(delta.to_string()),
            done: Some(false),
            ..Self::empty()
        }
    }

    pub fn agent_event_done(run_id: &str) -> Self {
        Self {
            r#type: "event",
            event: Some("agent"),
            run_id: Some(run_id.to_string()),
            done: Some(true),
            ..Self::empty()
        }
    }

    pub fn res_ok(req_id: &str, payload: Value) -> Self {
        Self {
            r#type: "res",
            id: Some(req_id.to_string()),
            status: Some("ok"),
            payload: Some(payload),
            ..Self::empty()
        }
    }

    pub fn error(code: u32, msg: impl Into<String>) -> Self {
        Self {
            r#type: "error",
            code: Some(code),
            message: Some(msg.into()),
            ..Self::empty()
        }
    }

    pub fn error_with_id(req_id: &str, code: u32, msg: impl Into<String>) -> Self {
        Self {
            r#type: "error",
            id: Some(req_id.to_string()),
            code: Some(code),
            message: Some(msg.into()),
            ..Self::empty()
        }
    }

    fn empty() -> Self {
        Self {
            r#type: "error",
            id: None,
            event: None,
            status: None,
            code: None,
            message: None,
            payload: None,
            run_id: None,
            delta: None,
            done: None,
        }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("OutboundFrame serialization cannot fail")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn connect_challenge_has_nonce() {
        let frame = OutboundFrame::connect_challenge("abc123", 1000);
        let json = frame.to_json();
        assert!(json.contains("abc123"));
        assert!(json.contains("1000"));
        assert!(json.contains("connect.challenge"));
    }

    #[test]
    fn hello_ok_has_device_token() {
        let frame = OutboundFrame::hello_ok("tok_xyz");
        let json = frame.to_json();
        assert!(json.contains("tok_xyz"));
        assert!(json.contains("hello"));
    }

    #[test]
    fn agent_delta_not_done() {
        let frame = OutboundFrame::agent_event_delta("run1", "Hello");
        let json = frame.to_json();
        assert!(json.contains("Hello"));
        assert!(json.contains("\"done\":false"));
    }

    #[test]
    fn agent_done_is_true() {
        let frame = OutboundFrame::agent_event_done("run1");
        let json = frame.to_json();
        assert!(json.contains("\"done\":true"));
        assert!(!json.contains("delta"));
    }

    #[test]
    fn error_frame_has_code_and_message() {
        let frame = OutboundFrame::error(401, "Unauthorized");
        let json = frame.to_json();
        assert!(json.contains("401"));
        assert!(json.contains("Unauthorized"));
        assert!(json.contains("error"));
    }

    #[test]
    fn error_with_id_includes_request_id() {
        let frame = OutboundFrame::error_with_id("req_42", 500, "Internal error");
        let json = frame.to_json();
        assert!(json.contains("req_42"));
        assert!(json.contains("500"));
    }

    #[test]
    fn res_ok_has_payload() {
        let frame = OutboundFrame::res_ok("req_1", serde_json::json!({"result": "done"}));
        let json = frame.to_json();
        assert!(json.contains("req_1"));
        assert!(json.contains("done"));
        assert!(json.contains("\"status\":\"ok\""));
    }

    #[test]
    fn to_json_never_panics() {
        let frame = OutboundFrame::empty();
        let _ = frame.to_json();
    }
}
