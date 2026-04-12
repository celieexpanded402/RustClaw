mod fs_tools {
    #[test]
    fn read_file_returns_content() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.txt");
        std::fs::write(&path, "hello world").unwrap();
        let result = rustclaw::tools::fs::read_file(path.to_str().unwrap());
        assert_eq!(result.unwrap(), "hello world");
    }

    #[test]
    fn read_file_missing_returns_error() {
        let result = rustclaw::tools::fs::read_file("/nonexistent/file.txt");
        assert!(result.is_err());
    }

    #[test]
    fn write_file_creates_with_parents() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("sub/dir/file.txt");
        rustclaw::tools::fs::write_file(path.to_str().unwrap(), "content").unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "content");
    }

    #[test]
    fn patch_file_replaces_first_occurrence() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.txt");
        std::fs::write(&path, "foo bar foo").unwrap();
        rustclaw::tools::fs::patch_file(path.to_str().unwrap(), "foo", "baz").unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "baz bar foo");
    }

    #[test]
    fn patch_file_pattern_not_found_errors() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.txt");
        std::fs::write(&path, "hello").unwrap();
        let result = rustclaw::tools::fs::patch_file(path.to_str().unwrap(), "missing", "new");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("pattern not found"));
    }

    #[test]
    fn list_dir_shows_files() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("a.txt"), "").unwrap();
        std::fs::write(dir.path().join("b.rs"), "").unwrap();
        let result = rustclaw::tools::fs::list_dir(dir.path().to_str().unwrap(), 1).unwrap();
        assert!(result.contains("a.txt"));
        assert!(result.contains("b.rs"));
    }

    #[test]
    fn list_dir_skips_hidden() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(dir.path().join(".hidden")).unwrap();
        std::fs::write(dir.path().join("visible.txt"), "").unwrap();
        let result = rustclaw::tools::fs::list_dir(dir.path().to_str().unwrap(), 1).unwrap();
        assert!(!result.contains(".hidden"));
        assert!(result.contains("visible.txt"));
    }

    #[test]
    fn list_dir_depth_limited() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("a/b/c/d")).unwrap();
        std::fs::write(dir.path().join("a/b/c/d/deep.txt"), "").unwrap();
        let result = rustclaw::tools::fs::list_dir(dir.path().to_str().unwrap(), 2).unwrap();
        assert!(result.contains("a/"));
        assert!(result.contains("b/"));
        assert!(!result.contains("deep.txt"));
    }
}

mod exec_tools {
    #[tokio::test]
    async fn run_command_basic() {
        let result = rustclaw::tools::exec::run_command("echo hello", "/tmp", 5, "/tmp").await.unwrap();
        assert_eq!(result.exit_code, 0);
        assert!(result.stdout.contains("hello"));
    }

    #[tokio::test]
    async fn run_command_outside_workspace_blocked() {
        let result = rustclaw::tools::exec::run_command("echo test", "/", 5, "/tmp").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("outside workspace"));
    }

    #[tokio::test]
    async fn run_command_timeout() {
        let result = rustclaw::tools::exec::run_command("sleep 10", "/tmp", 1, "/tmp").await.unwrap();
        assert_eq!(result.exit_code, -1);
        assert!(result.stderr.contains("timed out"));
    }
}

mod agent_loop {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    use axum::{extract::State, routing::post, Json, Router};
    use serde_json::json;

    struct MockState {
        call_count: AtomicUsize,
    }

    async fn mock_openai_chat(
        State(state): State<Arc<MockState>>,
        Json(_body): Json<serde_json::Value>,
    ) -> Json<serde_json::Value> {
        let n = state.call_count.fetch_add(1, Ordering::SeqCst);

        if n == 0 {
            // First call: return a tool_call for list_dir
            Json(json!({
                "choices": [{
                    "message": {
                        "content": null,
                        "tool_calls": [{
                            "id": "call_1",
                            "type": "function",
                            "function": {
                                "name": "list_dir",
                                "arguments": "{\"path\":\"/tmp\",\"depth\":1}"
                            }
                        }]
                    },
                    "finish_reason": "tool_calls"
                }]
            }))
        } else {
            // Second call: return final text
            Json(json!({
                "choices": [{
                    "message": {
                        "content": "Here are the files in /tmp.",
                        "tool_calls": []
                    },
                    "finish_reason": "stop"
                }]
            }))
        }
    }

    async fn start_mock_server() -> (String, tokio::task::JoinHandle<()>) {
        let state = Arc::new(MockState {
            call_count: AtomicUsize::new(0),
        });

        let app = Router::new()
            .route("/v1/chat/completions", post(mock_openai_chat))
            .with_state(state);

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let handle = tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        (format!("http://127.0.0.1:{}", addr.port()), handle)
    }

    #[tokio::test]
    async fn agentic_loop_dispatches_tool_and_returns_text() {
        let (base_url, _handle) = start_mock_server().await;

        let config = rustclaw::config::AgentConfig {
            provider: "openai".into(),
            api_key: "test".into(),
            base_url,
            model: "mock".into(),
            system_prompt: String::new(),
        };

        let tc = rustclaw::agent::ToolContext {
            config: rustclaw::config::ToolsConfig {
                enabled: true,
                workspace_dir: "/tmp".into(),
                allow_exec: false,
                exec_timeout_secs: 5,
            },
            discord_http: None,
            email_config: None,
            mcp: None,
        };

        let runner = rustclaw::agent::AgentRunner::new(config);
        let mut tokens = Vec::new();
        let result = runner
            .run_agentic("list files in /tmp", &[], &tc, |t| tokens.push(t))
            .await
            .unwrap();

        assert!(result.contains("Here are the files"));
        assert!(!tokens.is_empty());
    }

    #[tokio::test]
    async fn agentic_loop_text_only_no_tool_call() {
        let state = Arc::new(MockState {
            call_count: AtomicUsize::new(1), // skip to "return text" mode
        });

        let app = Router::new()
            .route("/v1/chat/completions", post(mock_openai_chat))
            .with_state(state);

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });

        let config = rustclaw::config::AgentConfig {
            provider: "openai".into(),
            api_key: "test".into(),
            base_url: format!("http://127.0.0.1:{}", addr.port()),
            model: "mock".into(),
            system_prompt: String::new(),
        };

        let tc = rustclaw::agent::ToolContext {
            config: rustclaw::config::ToolsConfig::default(),
            discord_http: None,
            email_config: None,
            mcp: None,
        };

        let runner = rustclaw::agent::AgentRunner::new(config);
        let result = runner
            .run_agentic("hello", &[], &tc, |_| {})
            .await
            .unwrap();

        assert_eq!(result, "Here are the files in /tmp.");
    }
}

mod config {
    #[test]
    fn config_loads_defaults() {
        let cfg = rustclaw::config::AppConfig::default();
        assert_eq!(cfg.gateway.port, 18789);
        assert_eq!(cfg.agent.provider, "anthropic");
        assert!(!cfg.channels.telegram.enabled);
        assert!(!cfg.channels.discord.enabled);
    }

    #[test]
    fn gateway_listen_addr() {
        let cfg = rustclaw::config::GatewayConfig {
            port: 9999,
            bind: "10.0.0.1".into(),
            token: None,
        };
        assert_eq!(cfg.listen_addr(), "10.0.0.1:9999");
    }
}
