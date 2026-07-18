mod support;

use std::{collections::BTreeMap, fs};

use dbmd_app::{render, RenderRequest};
use support::TestProject;

const CONFIG: &str = include_str!("fixtures/sqlite/full_schema/dbmd.toml");
const SCHEMA: &str = include_str!("fixtures/sqlite/full_schema/schema.sql");
const ANALYTICS_SCHEMA: &str = include_str!("fixtures/sqlite/full_schema/analytics.sql");

#[test]
fn renders_the_complete_sqlite_schema_surface_deterministically() {
    let project = TestProject::from_fixture(CONFIG, SCHEMA, ANALYTICS_SCHEMA);

    let first_report = render(project.request()).expect("first render should succeed");
    let first = fs::read_to_string(project.output_path()).expect("artifact should exist");
    let second_report = render(project.request()).expect("second render should succeed");
    let second = fs::read_to_string(project.output_path()).expect("artifact should still exist");

    assert_eq!(first, second);
    assert_eq!(first_report, second_report);
    assert_eq!(first_report.bytes_written, first.len());
    insta::assert_snapshot!("full_sqlite_render", first);
}

#[test]
fn preserves_the_previous_artifact_when_introspection_fails() {
    let project = TestProject::from_fixture(CONFIG, SCHEMA, ANALYTICS_SCHEMA);
    fs::write(project.output_path(), "previous artifact\n")
        .expect("old artifact should be written");
    let request = RenderRequest::with_environment(
        project.path().join("dbmd.toml"),
        BTreeMap::from([
            (
                "DBMD_TEST_DATABASE".to_string(),
                project
                    .path()
                    .join("missing.db")
                    .to_string_lossy()
                    .into_owned(),
            ),
            (
                "DBMD_TEST_ANALYTICS_DATABASE".to_string(),
                project
                    .path()
                    .join("analytics.db")
                    .to_string_lossy()
                    .into_owned(),
            ),
        ]),
    );

    let error = render(request).expect_err("missing database should fail introspection");

    assert!(error.to_string().contains("failed to open SQLite source"));
    assert_eq!(
        fs::read_to_string(project.output_path()).expect("old artifact should remain"),
        "previous artifact\n"
    );
}
