use std::fs;

use dbmd_app::{init, init_templates, render, InitRequest, InitTemplatesRequest, RenderRequest};
use rusqlite::Connection;

#[test]
fn initializes_an_unambiguous_sqlite_project_that_can_render() {
    let project = tempfile::tempdir().expect("temporary project should be created");
    Connection::open(project.path().join("app.db"))
        .expect("SQLite database should open")
        .execute_batch("CREATE TABLE users (id INTEGER PRIMARY KEY);")
        .expect("SQLite schema should execute");

    let report = init(InitRequest::new(project.path().join("dbmd.toml")))
        .expect("unambiguous SQLite project should initialize");
    let config = fs::read_to_string(&report.config_path).expect("config should exist");

    assert_eq!(report.detected_database.as_deref(), Some("app.db".as_ref()));
    assert!(config.contains("path = \"app.db\""));
    render(RenderRequest::new(&report.config_path)).expect("initialized config should render");
    assert!(project.path().join("DATABASE.md").exists());
}

#[test]
fn refuses_to_replace_an_existing_config() {
    let project = tempfile::tempdir().expect("temporary project should be created");
    let config_path = project.path().join("dbmd.toml");
    fs::write(&config_path, "user owned\n").expect("existing config should be written");

    let error =
        init(InitRequest::new(&config_path)).expect_err("existing config must be preserved");

    assert!(error.to_string().contains("already exists"));
    assert_eq!(
        fs::read_to_string(config_path).expect("config should remain"),
        "user owned\n"
    );
}

#[test]
fn writes_an_editable_example_when_sqlite_discovery_is_ambiguous() {
    let project = tempfile::tempdir().expect("temporary project should be created");
    for name in ["app.db", "test.db"] {
        Connection::open(project.path().join(name))
            .expect("SQLite database should open")
            .execute_batch("CREATE TABLE records (id INTEGER PRIMARY KEY);")
            .expect("SQLite schema should execute");
    }

    let report = init(InitRequest::new(project.path().join("dbmd.toml")))
        .expect("ambiguous project should receive an example config");
    let config = fs::read_to_string(report.config_path).expect("config should exist");

    assert!(report.detected_database.is_none());
    assert!(config.contains("path = \"dev.db\""));
    assert!(!config.contains("app.db"));
    assert!(!config.contains("test.db"));
}

#[test]
fn discovery_requires_a_regular_file_supported_extension_and_sqlite_header() {
    let project = tempfile::tempdir().expect("temporary project should be created");
    Connection::open(project.path().join("warehouse.SQLITE3"))
        .expect("valid SQLite database should open")
        .execute_batch("CREATE TABLE records (id INTEGER PRIMARY KEY);")
        .expect("valid SQLite database should persist a header");
    fs::write(project.path().join("fake.db"), "not sqlite\n")
        .expect("fake database candidate should be written");
    Connection::open(project.path().join("ignored.data"))
        .expect("unsupported-extension SQLite file should open");
    fs::create_dir(project.path().join("directory.sqlite"))
        .expect("directory candidate should be created");
    #[cfg(unix)]
    std::os::unix::fs::symlink(
        project.path().join("warehouse.SQLITE3"),
        project.path().join("linked.db"),
    )
    .expect("symlink candidate should be created");

    let report = init(InitRequest::new(project.path().join("dbmd.toml")))
        .expect("one real supported SQLite file should be discovered");

    assert_eq!(
        report.detected_database.as_deref(),
        Some("warehouse.SQLITE3".as_ref())
    );
}

#[test]
fn initializes_a_complete_project_owned_template_profile() {
    let project = tempfile::tempdir().expect("temporary project should be created");
    let root = project.path().join("templates/dbmd");

    let report = init_templates(InitTemplatesRequest::new(&root))
        .expect("template profile should initialize");

    assert_eq!(report.template_root, root);
    assert_eq!(
        report.files.len(),
        dbmd_render::embedded_template_files().len() + dbmd_backends::all_template_files().len()
    );
    let backend_templates = dbmd_backends::all_template_files();
    for file in dbmd_render::embedded_template_files()
        .iter()
        .chain(&backend_templates)
    {
        assert_eq!(
            fs::read_to_string(root.join("agent").join(file.relative_path))
                .expect("initialized template should exist"),
            file.contents
        );
    }
    dbmd_render::Renderer::from_template_root(&root, "agent", &dbmd_backends::all_template_files())
        .expect("initialized template root should compile");
}

#[test]
fn template_initialization_refuses_to_overlay_an_existing_directory() {
    let project = tempfile::tempdir().expect("temporary project should be created");
    let root = project.path().join("templates/dbmd");
    fs::create_dir_all(&root).expect("existing template root should be created");
    fs::write(root.join("user-owned.txt"), "preserve me\n").expect("marker should be written");

    let error = init_templates(InitTemplatesRequest::new(&root))
        .expect_err("existing template root must be preserved");

    assert!(error.to_string().contains("already exists"));
    assert_eq!(
        fs::read_to_string(root.join("user-owned.txt")).expect("marker should remain"),
        "preserve me\n"
    );
}
