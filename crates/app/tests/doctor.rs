use std::{collections::BTreeMap, fs};

use dbmd_app::{doctor, DiagnosticStage, DiagnosticStatus, DoctorRequest};
use rusqlite::Connection;

#[test]
fn local_doctor_does_not_connect_by_default() {
    let project = tempfile::tempdir().expect("temporary project should exist");
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
fn connection_doctor_reports_failure_after_independent_local_checks() {
    let project = tempfile::tempdir().expect("temporary project should exist");
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
    let project = tempfile::tempdir().expect("temporary project should exist");
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
    let project = tempfile::tempdir().expect("temporary project should exist");
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

#[test]
fn missing_source_environment_does_not_hide_independent_local_failures() {
    let project = tempfile::tempdir().expect("temporary project should exist");
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
fn request_debug_lists_environment_names_but_not_values() {
    let request = DoctorRequest::with_environment(
        "dbmd.toml",
        BTreeMap::from([("DATABASE_URL".to_string(), "sentinel-secret".to_string())]),
    );

    let debug = format!("{request:?}");

    assert!(debug.contains("DATABASE_URL"));
    assert!(!debug.contains("sentinel-secret"));
}
