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
