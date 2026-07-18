use std::{fs, process::Command};

use rusqlite::Connection;

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

#[test]
fn verify_uses_nonzero_exit_and_prints_the_complete_diff_for_drift() {
    let project = tempfile::tempdir().expect("temporary CLI project should be created");
    let database = project.path().join("app.db");
    Connection::open(&database)
        .expect("SQLite fixture should open")
        .execute_batch("CREATE TABLE users (id INTEGER PRIMARY KEY, email TEXT NOT NULL);")
        .expect("SQLite fixture should execute");
    let init = Command::new(env!("CARGO_BIN_EXE_dbmd"))
        .current_dir(project.path())
        .arg("init")
        .output()
        .expect("init should execute");
    assert!(
        init.status.success(),
        "{}",
        String::from_utf8_lossy(&init.stderr)
    );
    assert!(String::from_utf8(init.stdout)
        .expect("init report should be UTF-8")
        .contains("Detected SQLite database: app.db"));

    let render = Command::new(env!("CARGO_BIN_EXE_dbmd"))
        .current_dir(project.path())
        .arg("render")
        .output()
        .expect("render should execute");
    assert!(render.status.success());
    fs::write(project.path().join("DATABASE.md"), "manual edit\n")
        .expect("canonical artifact should be edited");

    let verify = Command::new(env!("CARGO_BIN_EXE_dbmd"))
        .current_dir(project.path())
        .args(["verify", "--diff"])
        .output()
        .expect("verify should execute");

    assert!(!verify.status.success());
    let stderr = String::from_utf8(verify.stderr).expect("diagnostics should be UTF-8");
    assert!(stderr.contains("error: canonical artifact has drifted"));
    assert!(stderr.contains("modified  DATABASE.md"));
    assert!(stderr.contains("--- a/DATABASE.md"));
    assert!(stderr.contains("-manual edit"));
    assert_eq!(
        fs::read_to_string(project.path().join("DATABASE.md"))
            .expect("canonical artifact should remain"),
        "manual edit\n"
    );
}
