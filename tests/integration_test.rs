#[test]
fn test_main() {
    let output = std::process::Command::new("target/debug/main")
        .output()
        .unwrap();
    assert!(output.status.success());
}
