use std::{fs, process::Command};

use rusqlite::Connection;

#[test]
fn render_help_exposes_configured_and_one_off_application_inputs() {
    let output = Command::new(env!("CARGO_BIN_EXE_dbmd"))
        .args(["render", "--help"])
        .output()
        .expect("dbmd help should execute");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("help should be UTF-8");
    assert!(stdout.contains("--config"));
    assert!(stdout.contains("--source"));
    assert!(stdout.contains("--backend"));
    assert!(stdout.contains("--path"));
    assert!(stdout.contains("--output"));
    assert!(stdout.contains("--stdout"));
    assert!(stdout.contains("--template-root"));
}

#[test]
fn configless_sqlite_render_writes_the_requested_output() {
    let project = tempfile::tempdir().expect("temporary CLI project should be created");
    Connection::open(project.path().join("app.db"))
        .expect("SQLite fixture should open")
        .execute_batch("CREATE TABLE users (id INTEGER PRIMARY KEY);")
        .expect("SQLite fixture should execute");

    let render = Command::new(env!("CARGO_BIN_EXE_dbmd"))
        .current_dir(project.path())
        .args([
            "render",
            "--backend",
            "sqlite",
            "--path",
            "app.db",
            "--output",
            "ONE_OFF.md",
        ])
        .output()
        .expect("render should execute");

    assert!(
        render.status.success(),
        "{}",
        String::from_utf8_lossy(&render.stderr)
    );
    assert!(fs::read_to_string(project.path().join("ONE_OFF.md"))
        .expect("one-off output should exist")
        .contains("`main.users`"));
    assert!(!project.path().join("dbmd.toml").exists());
}

#[test]
fn configured_stdout_prints_only_markdown_and_does_not_write_canonical_output() {
    let project = tempfile::tempdir().expect("temporary CLI project should be created");
    Connection::open(project.path().join("app.db"))
        .expect("SQLite fixture should open")
        .execute_batch("CREATE TABLE users (id INTEGER PRIMARY KEY);")
        .expect("SQLite fixture should execute");
    fs::write(
        project.path().join("dbmd.toml"),
        r#"
[sources.app]
backend = "sqlite"
path = "app.db"

[output]
path = "DATABASE.md"
"#,
    )
    .expect("config should be written");

    let render = Command::new(env!("CARGO_BIN_EXE_dbmd"))
        .current_dir(project.path())
        .args(["render", "--stdout"])
        .output()
        .expect("render should execute");

    assert!(
        render.status.success(),
        "{}",
        String::from_utf8_lossy(&render.stderr)
    );
    let stdout = String::from_utf8(render.stdout).expect("Markdown should be UTF-8");
    assert!(stdout.contains("`main.users`"));
    assert!(!stdout.contains("Rendered 1 source"));
    assert!(!project.path().join("DATABASE.md").exists());
}

#[test]
fn render_template_root_flag_replaces_the_configured_template_source() {
    let project = tempfile::tempdir().expect("temporary CLI project should be created");
    Connection::open(project.path().join("app.db"))
        .expect("SQLite fixture should open")
        .execute_batch("CREATE TABLE users (id INTEGER PRIMARY KEY);")
        .expect("SQLite fixture should execute");
    fs::write(
        project.path().join("dbmd.toml"),
        r#"
[sources.app]
backend = "sqlite"
path = "app.db"

[output]
path = "DATABASE.md"
"#,
    )
    .expect("config should be written");
    let root = project.path().join("custom-templates");
    for file in dbmd_render::embedded_template_files() {
        let path = root.join("agent").join(file.relative_path);
        fs::create_dir_all(path.parent().expect("template should have a parent"))
            .expect("template directories should be created");
        let contents = if file.template_name == "database.md.j2" {
            "# CLI custom `{{ context.sources[0].id }}`\n"
        } else {
            file.contents
        };
        fs::write(path, contents).expect("template should be written");
    }

    let render = Command::new(env!("CARGO_BIN_EXE_dbmd"))
        .current_dir(project.path())
        .args(["render", "--template-root", "custom-templates"])
        .output()
        .expect("render should execute");

    assert!(
        render.status.success(),
        "{}",
        String::from_utf8_lossy(&render.stderr)
    );
    assert_eq!(
        fs::read_to_string(project.path().join("DATABASE.md"))
            .expect("custom artifact should exist"),
        "# CLI custom `app`"
    );
}

#[test]
fn init_templates_creates_a_complete_compilable_profile() {
    let project = tempfile::tempdir().expect("temporary CLI project should be created");

    let init = Command::new(env!("CARGO_BIN_EXE_dbmd"))
        .current_dir(project.path())
        .arg("init-templates")
        .output()
        .expect("init-templates should execute");

    assert!(
        init.status.success(),
        "{}",
        String::from_utf8_lossy(&init.stderr)
    );
    let root = project.path().join("templates/dbmd");
    dbmd_render::Renderer::from_template_root(&root, "agent")
        .expect("CLI-initialized template root should compile");
    assert!(String::from_utf8(init.stdout)
        .expect("report should be UTF-8")
        .contains("8 template files"));
}

#[test]
fn init_ci_creates_a_protected_github_actions_workflow() {
    let project = tempfile::tempdir().expect("temporary CLI project should be created");

    let init = Command::new(env!("CARGO_BIN_EXE_dbmd"))
        .current_dir(project.path())
        .args(["init", "ci"])
        .output()
        .expect("init ci should execute");

    assert!(
        init.status.success(),
        "{}",
        String::from_utf8_lossy(&init.stderr)
    );
    let path = project.path().join(".github/workflows/dbmd.yml");
    assert!(fs::read_to_string(&path)
        .expect("workflow should exist")
        .contains("run: dbmd verify"));

    let second = Command::new(env!("CARGO_BIN_EXE_dbmd"))
        .current_dir(project.path())
        .args(["init", "ci"])
        .output()
        .expect("second init ci should execute");
    assert!(!second.status.success());
    assert!(String::from_utf8(second.stderr)
        .expect("diagnostic should be UTF-8")
        .contains("explicit overwrite is required"));
}

#[test]
fn render_source_flags_replace_config_selection_and_preserve_flag_order() {
    let project = tempfile::tempdir().expect("temporary CLI project should be created");
    for (database, table) in [("app.db", "app_table"), ("analytics.db", "analytics_table")] {
        Connection::open(project.path().join(database))
            .expect("SQLite fixture should open")
            .execute_batch(&format!("CREATE TABLE {table} (id INTEGER PRIMARY KEY);"))
            .expect("SQLite fixture should execute");
    }
    fs::write(
        project.path().join("dbmd.toml"),
        r#"
[sources.app]
backend = "sqlite"
path = "app.db"

[sources.analytics]
backend = "sqlite"
path = "analytics.db"

[output]
path = "DATABASE.md"
sources = ["analytics", "app"]
"#,
    )
    .expect("config should be written");

    let render = Command::new(env!("CARGO_BIN_EXE_dbmd"))
        .current_dir(project.path())
        .args(["render", "--source", "app", "--source", "analytics"])
        .output()
        .expect("render should execute");

    assert!(
        render.status.success(),
        "{}",
        String::from_utf8_lossy(&render.stderr)
    );
    let markdown = fs::read_to_string(project.path().join("DATABASE.md"))
        .expect("rendered artifact should exist");
    let app = markdown
        .find("## Source: `app`")
        .expect("app source should render");
    let analytics = markdown
        .find("## Source: `analytics`")
        .expect("analytics source should render");
    assert!(
        app < analytics,
        "source sections should preserve CLI flag order"
    );
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
