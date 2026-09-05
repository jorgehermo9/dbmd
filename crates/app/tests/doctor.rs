mod support;

use std::{collections::BTreeMap, fs};

use dbmd_app::{doctor, DiagnosticStage, DiagnosticStatus, DoctorRequest};
use rusqlite::Connection;
use support::TestProject;

#[test]
fn local_doctor_does_not_connect_by_default() {
    let project = TestProject::new();
    let config = project.path().join("dbmd.toml");
    fs::write(
        &config,
        r#"
[sources.app]
backend = "sqlite"
path = "missing.db"

[output]
path = "DATABASE.md"
"#,
    )
    .expect("config should be written");

    let report = doctor(DoctorRequest::new(config));

    assert!(report.is_ready());
    assert!(report.diagnostics.iter().any(|diagnostic| {
        diagnostic.stage == DiagnosticStage::Connection
            && diagnostic.status == DiagnosticStatus::Skipped
    }));
}

#[test]
fn successful_connection_doctor_reports_every_stage_in_deterministic_order() {
    let project = TestProject::new();
    Connection::open(project.path().join("app.db"))
        .expect("fixture database should open")
        .execute_batch("CREATE TABLE users (id INTEGER PRIMARY KEY);")
        .expect("fixture should execute");
    let config = project.path().join("dbmd.toml");
    fs::write(
        &config,
        r#"
[sources.app]
backend = "sqlite"
path = "app.db"

[output]
path = "DATABASE.md"
"#,
    )
    .expect("config should be written");

    let report = doctor(DoctorRequest::new(config).with_connections());

    assert!(report.is_ready());
    assert_eq!(
        report
            .diagnostics
            .iter()
            .map(|diagnostic| (diagnostic.stage, diagnostic.status))
            .collect::<Vec<_>>(),
        [
            (DiagnosticStage::Configuration, DiagnosticStatus::Passed),
            (DiagnosticStage::Environment, DiagnosticStatus::Passed),
            (DiagnosticStage::Output, DiagnosticStatus::Passed),
            (DiagnosticStage::Templates, DiagnosticStatus::Passed),
            (DiagnosticStage::Connection, DiagnosticStatus::Passed),
        ]
    );
    assert_eq!(
        report.diagnostics[4]
            .source
            .as_ref()
            .map(dbmd_core::SourceId::as_str),
        Some("app")
    );
}

#[test]
fn connection_doctor_reports_failure_after_independent_local_checks() {
    let project = TestProject::new();
    let config = project.path().join("dbmd.toml");
    fs::write(
        &config,
        r#"
[sources.app]
backend = "sqlite"
path = "missing.db"

[output]
path = "DATABASE.md"
"#,
    )
    .expect("config should be written");

    let report = doctor(DoctorRequest::new(config).with_connections());

    assert!(!report.is_ready());
    assert!(report.diagnostics.iter().any(|diagnostic| {
        diagnostic.stage == DiagnosticStage::Output && diagnostic.status == DiagnosticStatus::Passed
    }));
    assert!(report.diagnostics.iter().any(|diagnostic| {
        diagnostic.stage == DiagnosticStage::Templates
            && diagnostic.status == DiagnosticStatus::Passed
    }));
    assert!(report.diagnostics.iter().any(|diagnostic| {
        diagnostic.stage == DiagnosticStage::Connection
            && diagnostic.status == DiagnosticStatus::Failed
            && diagnostic
                .source
                .as_ref()
                .is_some_and(|source| source.as_str() == "app")
    }));
}

#[test]
fn all_sources_checks_sources_outside_the_canonical_selection() {
    let project = TestProject::new();
    Connection::open(project.path().join("selected.db"))
        .expect("selected database should open")
        .execute_batch("CREATE TABLE users (id INTEGER PRIMARY KEY);")
        .expect("fixture should execute");
    let config = project.path().join("dbmd.toml");
    fs::write(
        &config,
        r#"
[sources.selected]
backend = "sqlite"
path = "selected.db"

[sources.unselected]
backend = "sqlite"
path = "missing.db"

[output]
path = "DATABASE.md"
sources = ["selected"]
"#,
    )
    .expect("config should be written");

    let canonical = doctor(DoctorRequest::new(&config).with_connections());
    let all = doctor(
        DoctorRequest::new(&config)
            .with_connections()
            .with_all_sources(),
    );

    assert!(canonical.is_ready());
    assert!(!all.is_ready());
    assert!(all.diagnostics.iter().any(|diagnostic| {
        diagnostic.status == DiagnosticStatus::Failed
            && diagnostic
                .source
                .as_ref()
                .is_some_and(|source| source.as_str() == "unselected")
    }));
}

#[cfg(unix)]
#[test]
fn rejects_an_existing_non_regular_single_file_destination() {
    let project = TestProject::new();
    let config = project.path().join("dbmd.toml");
    fs::write(
        &config,
        r#"
[sources.app]
backend = "sqlite"
path = "app.db"

[output]
path = "/dev/null"
"#,
    )
    .expect("config should be written");

    let report = doctor(DoctorRequest::new(config));

    assert!(!report.is_ready());
    assert!(report.diagnostics.iter().any(|diagnostic| {
        diagnostic.stage == DiagnosticStage::Output && diagnostic.status == DiagnosticStatus::Failed
    }));
}

#[cfg(unix)]
#[test]
fn reports_an_unwritable_existing_output_parent_as_a_local_failure() {
    use std::os::unix::fs::PermissionsExt;

    let project = TestProject::new();
    let config = project.path().join("dbmd.toml");
    let output_parent = project.path().join("locked");
    fs::create_dir(&output_parent).expect("output parent should be created");
    fs::set_permissions(&output_parent, fs::Permissions::from_mode(0o500))
        .expect("output parent should become unwritable");
    fs::write(
        &config,
        r#"
[sources.app]
backend = "sqlite"
path = "app.db"

[output]
path = "locked/DATABASE.md"
"#,
    )
    .expect("config should be written");

    let report = doctor(DoctorRequest::new(config));
    fs::set_permissions(&output_parent, fs::Permissions::from_mode(0o700))
        .expect("fixture permissions should be restored for cleanup");

    assert!(!report.is_ready());
    assert!(report.diagnostics.iter().any(|diagnostic| {
        diagnostic.stage == DiagnosticStage::Output && diagnostic.status == DiagnosticStatus::Failed
    }));
}

#[test]
fn missing_source_environment_does_not_hide_independent_local_failures() {
    let project = TestProject::new();
    let config = project.path().join("dbmd.toml");
    fs::write(
        &config,
        r#"
[sources.app]
backend = "postgres"
url = "${DATABASE_URL}"

[output]
path = "."

[templates]
dir = "missing-templates"
"#,
    )
    .expect("config should be written");

    let report = doctor(DoctorRequest::with_environment(config, BTreeMap::new()));

    assert!(!report.is_ready());
    for stage in [
        DiagnosticStage::Environment,
        DiagnosticStage::Output,
        DiagnosticStage::Templates,
    ] {
        assert!(report.diagnostics.iter().any(|diagnostic| {
            diagnostic.stage == stage && diagnostic.status == DiagnosticStatus::Failed
        }));
    }
}

#[test]
fn invalid_custom_template_syntax_fails_template_preflight_without_connecting() {
    let project = TestProject::new();
    let config = project.path().join("dbmd.toml");
    fs::write(
        &config,
        r#"
[sources.app]
backend = "sqlite"
path = "missing.db"

[output]
path = "DATABASE.md"

[templates]
dir = "templates/dbmd"
"#,
    )
    .expect("config should be written");
    let root = project.path().join("templates/dbmd/agent");
    for file in dbmd_render::embedded_template_files()
        .iter()
        .chain(&dbmd_backends::all_template_files())
    {
        let path = root.join(file.relative_path);
        fs::create_dir_all(path.parent().expect("template should have a parent"))
            .expect("template directory should be created");
        fs::write(&path, file.contents).expect("template should be written");
    }
    fs::write(root.join("single_file/database.md.j2"), "{% if context %}")
        .expect("invalid template should be written");

    let report = doctor(DoctorRequest::new(config));

    assert!(!report.is_ready());
    assert!(report.diagnostics.iter().any(|diagnostic| {
        diagnostic.stage == DiagnosticStage::Templates
            && diagnostic.status == DiagnosticStatus::Failed
            && diagnostic.message.contains("failed strict preflight")
    }));
    assert!(report.diagnostics.iter().any(|diagnostic| {
        diagnostic.stage == DiagnosticStage::Connection
            && diagnostic.status == DiagnosticStatus::Skipped
    }));
}

#[test]
fn missing_source_environment_skips_only_the_affected_connection_check() {
    let project = TestProject::new();
    let config = project.path().join("dbmd.toml");
    fs::write(
        &config,
        r#"
[sources.app]
backend = "postgres"
url = "${DATABASE_URL}"

[output]
path = "DATABASE.md"
"#,
    )
    .expect("config should be written");

    let report =
        doctor(DoctorRequest::with_environment(config, BTreeMap::new()).with_connections());
    let connection = report
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.stage == DiagnosticStage::Connection)
        .expect("connection diagnostic should be present");

    assert_eq!(connection.status, DiagnosticStatus::Skipped);
    assert_eq!(
        connection.source.as_ref().map(dbmd_core::SourceId::as_str),
        Some("app")
    );
    assert!(connection.message.contains("DATABASE_URL"));
    assert!(!connection.message.contains("__DBMD_MISSING_ENVIRONMENT__"));
}

#[test]
fn request_debug_lists_environment_names_but_not_values() {
    let request = DoctorRequest::with_environment(
        "dbmd.toml",
        BTreeMap::from([("DATABASE_URL".to_string(), "sentinel-secret".to_string())]),
    );

    let debug = format!("{request:?}");

    assert!(debug.contains("DATABASE_URL"));
    assert!(!debug.contains("sentinel-secret"));
}
