use std::process::Command;

#[test]
fn render_help_exposes_only_application_inputs() {
    let output = Command::new(env!("CARGO_BIN_EXE_dbmd"))
        .args(["render", "--help"])
        .output()
        .expect("dbmd help should execute");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("help should be UTF-8");
    assert!(stdout.contains("--config"));
    assert!(!stdout.contains("sqlite"));
    assert!(!stdout.contains("template"));
}
