mod support;

use std::fs;

use dbmd_app::{render, verify, ArtifactChangeKind, VerifyRequest};
use std::collections::BTreeMap;
use support::TestProject;

const CONFIG: &str = include_str!("fixtures/sqlite/full_schema/dbmd.toml");
const SCHEMA: &str = include_str!("fixtures/sqlite/full_schema/schema.sql");
const ANALYTICS_SCHEMA: &str = include_str!("fixtures/sqlite/full_schema/analytics.sql");
const DIRECTORY_CONFIG: &str = include_str!("fixtures/sqlite/directory/dbmd.toml");

#[test]
fn request_debug_lists_environment_names_without_values() {
    let request = VerifyRequest::with_environment(
        "dbmd.toml",
        BTreeMap::from([("DATABASE_URL".to_string(), "sentinel-secret".to_string())]),
    );

    let debug = format!("{request:?}");

    assert!(debug.contains("DATABASE_URL"));
    assert!(!debug.contains("sentinel-secret"));
}

#[test]
fn reports_an_unchanged_single_file_as_fresh() {
    let project = TestProject::from_sqlite_fixture(CONFIG, SCHEMA, ANALYTICS_SCHEMA);
    render(project.request()).expect("canonical artifact should render");

    let report = verify(VerifyRequest::with_environment(
        project.path().join("dbmd.toml"),
        project.environment(),
    ))
    .expect("verification should complete");

    assert!(report.is_fresh());
    assert!(report.changes.is_empty());
    assert!(report.diff.is_none());
}

#[test]
fn reports_modified_bytes_without_changing_the_canonical_file() {
    let project = TestProject::from_sqlite_fixture(CONFIG, SCHEMA, ANALYTICS_SCHEMA);
    render(project.request()).expect("canonical artifact should render");
    fs::write(project.output_path(), "manually edited\n")
        .expect("canonical artifact should be edited");

    let report = verify(
        VerifyRequest::with_environment(project.path().join("dbmd.toml"), project.environment())
            .with_diff(true),
    )
    .expect("verification should distinguish drift from operational failure");

    assert!(!report.is_fresh());
    assert_eq!(report.changes.len(), 1);
    assert_eq!(report.changes[0].kind, ArtifactChangeKind::Modified);
    assert_eq!(report.changes[0].path, "DATABASE.md");
    assert!(report
        .diff
        .as_deref()
        .is_some_and(|diff| diff.contains("-manually edited")));
    assert_eq!(
        fs::read_to_string(project.output_path()).expect("artifact should remain readable"),
        "manually edited\n"
    );
}

#[test]
fn compares_complete_directory_file_sets() {
    let project = TestProject::from_sqlite_fixture(DIRECTORY_CONFIG, SCHEMA, ANALYTICS_SCHEMA);
    render(project.request()).expect("canonical directory should render");
    let output = project.path().join("database");
    fs::write(output.join("index.md"), "changed\n").expect("index should be changed");
    fs::remove_file(output.join("tables/main.accounts.md")).expect("table should be removed");
    fs::write(output.join("stale.md"), "stale\n").expect("stale file should be added");

    let report = verify(VerifyRequest::with_environment(
        project.path().join("dbmd.toml"),
        project.environment(),
    ))
    .expect("directory verification should complete");

    assert_eq!(
        report
            .changes
            .iter()
            .map(|change| (change.path.as_str(), change.kind))
            .collect::<Vec<_>>(),
        [
            ("index.md", ArtifactChangeKind::Modified),
            ("stale.md", ArtifactChangeKind::Deleted),
            ("tables/main.accounts.md", ArtifactChangeKind::Added),
        ]
    );
    assert_eq!(
        fs::read_to_string(output.join("index.md")).expect("edited index should remain"),
        "changed\n"
    );
}

#[test]
fn reports_missing_single_file_and_directory_outputs_as_added_drift() {
    let file_project = TestProject::from_sqlite_fixture(CONFIG, SCHEMA, ANALYTICS_SCHEMA);
    let file_report = verify(VerifyRequest::with_environment(
        file_project.path().join("dbmd.toml"),
        file_project.environment(),
    ))
    .expect("missing single-file output should be a drift result");

    assert_eq!(
        file_report
            .changes
            .iter()
            .map(|change| (change.path.as_str(), change.kind))
            .collect::<Vec<_>>(),
        [("DATABASE.md", ArtifactChangeKind::Added)]
    );
    assert!(!file_project.output_path().exists());

    let directory_project =
        TestProject::from_sqlite_fixture(DIRECTORY_CONFIG, SCHEMA, ANALYTICS_SCHEMA);
    let directory_report = verify(VerifyRequest::with_environment(
        directory_project.path().join("dbmd.toml"),
        directory_project.environment(),
    ))
    .expect("missing directory output should be a drift result");

    assert!(!directory_report.changes.is_empty());
    assert!(directory_report
        .changes
        .iter()
        .all(|change| change.kind == ArtifactChangeKind::Added));
    assert!(!directory_project.path().join("database").exists());
}

#[test]
fn reports_semantically_equivalent_markdown_edits_as_exact_byte_drift() {
    let project = TestProject::from_sqlite_fixture(CONFIG, SCHEMA, ANALYTICS_SCHEMA);
    render(project.request()).expect("canonical artifact should render");
    let mut edited =
        fs::read_to_string(project.output_path()).expect("artifact should be readable");
    edited.push('\n');
    fs::write(project.output_path(), &edited)
        .expect("equivalent whitespace edit should be written");

    let report = verify(VerifyRequest::with_environment(
        project.path().join("dbmd.toml"),
        project.environment(),
    ))
    .expect("byte drift should remain a comparison result");

    assert_eq!(report.changes.len(), 1);
    assert_eq!(report.changes[0].kind, ArtifactChangeKind::Modified);
    assert_eq!(
        fs::read_to_string(project.output_path()).expect("verify must not rewrite the artifact"),
        edited
    );
}

#[test]
fn returns_introspection_failures_as_operational_errors_instead_of_drift() {
    let project = TestProject::from_sqlite_fixture(CONFIG, SCHEMA, ANALYTICS_SCHEMA);
    let mut environment = project.environment();
    environment.insert(
        "DBMD_TEST_DATABASE".to_string(),
        project.path().join("missing.db").display().to_string(),
    );

    let error = verify(VerifyRequest::with_environment(
        project.path().join("dbmd.toml"),
        environment,
    ))
    .expect_err("failed introspection cannot produce a trustworthy drift report");

    assert!(error.to_string().contains("failed to open SQLite source"));
    assert!(!project.output_path().exists());
}

#[cfg(unix)]
#[test]
fn rejects_symlinks_inside_a_canonical_directory_as_unsafe_operational_state() {
    use std::os::unix::fs::symlink;

    let project = TestProject::from_sqlite_fixture(DIRECTORY_CONFIG, SCHEMA, ANALYTICS_SCHEMA);
    render(project.request()).expect("canonical directory should render");
    let outside = project.path().join("user-owned.md");
    fs::write(&outside, "private user content\n").expect("symlink target should be written");
    let link = project.path().join("database/linked.md");
    symlink(&outside, &link).expect("canonical directory symlink should be created");

    let error = verify(VerifyRequest::with_environment(
        project.path().join("dbmd.toml"),
        project.environment(),
    ))
    .expect_err("unsafe canonical directory entries cannot be treated as ordinary drift");

    assert!(error.to_string().contains("unsafe artifact output path"));
    assert_eq!(
        fs::read_to_string(outside).expect("symlink target should remain readable"),
        "private user content\n"
    );
}
