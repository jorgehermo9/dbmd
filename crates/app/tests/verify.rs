mod support;

use std::fs;

use dbmd_app::{render, verify, ArtifactChangeKind, VerifyRequest};
use support::TestProject;

const CONFIG: &str = include_str!("fixtures/sqlite/full_schema/dbmd.toml");
const SCHEMA: &str = include_str!("fixtures/sqlite/full_schema/schema.sql");
const ANALYTICS_SCHEMA: &str = include_str!("fixtures/sqlite/full_schema/analytics.sql");
const DIRECTORY_CONFIG: &str = include_str!("fixtures/sqlite/directory/dbmd.toml");

#[test]
fn reports_an_unchanged_single_file_as_fresh() {
    let project = TestProject::from_fixture(CONFIG, SCHEMA, ANALYTICS_SCHEMA);
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
    let project = TestProject::from_fixture(CONFIG, SCHEMA, ANALYTICS_SCHEMA);
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
    let project = TestProject::from_fixture(DIRECTORY_CONFIG, SCHEMA, ANALYTICS_SCHEMA);
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
