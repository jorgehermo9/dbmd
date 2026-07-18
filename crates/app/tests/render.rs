mod support;

use std::{collections::BTreeMap, fs};

use dbmd_app::{render, RenderRequest};
use support::TestProject;

const CONFIG: &str = include_str!("fixtures/sqlite/full_schema/dbmd.toml");
const SCHEMA: &str = include_str!("fixtures/sqlite/full_schema/schema.sql");
const ANALYTICS_SCHEMA: &str = include_str!("fixtures/sqlite/full_schema/analytics.sql");
const MULTI_SOURCE_CONFIG: &str = include_str!("fixtures/sqlite/multi_source/dbmd.toml");
const DIRECTORY_CONFIG: &str = include_str!("fixtures/sqlite/directory/dbmd.toml");

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

#[test]
fn renders_selected_sources_in_configured_order_with_source_sections() {
    let project = TestProject::from_fixture(MULTI_SOURCE_CONFIG, SCHEMA, ANALYTICS_SCHEMA);

    let report = render(project.request()).expect("multiple SQLite sources should render");
    let markdown = fs::read_to_string(project.output_path()).expect("artifact should exist");

    assert_eq!(
        report
            .sources
            .iter()
            .map(dbmd_core::SourceId::as_str)
            .collect::<Vec<_>>(),
        ["analytics", "app"]
    );
    insta::assert_snapshot!("multiple_sqlite_sources", markdown);
}

#[test]
fn atomically_renders_a_directory_artifact_without_stale_files() {
    let project = TestProject::from_fixture(DIRECTORY_CONFIG, SCHEMA, ANALYTICS_SCHEMA);
    let output = project.path().join("database");
    fs::create_dir_all(output.join("tables")).expect("old artifact tree should be created");
    fs::write(output.join("tables/stale.md"), "stale\n").expect("stale artifact should be created");

    let report = render(project.request()).expect("directory artifact should render");
    let index = fs::read_to_string(output.join("index.md")).expect("index should exist");
    let table = fs::read_to_string(output.join("tables/main.accounts.md"))
        .expect("table artifact should exist");

    assert_eq!(report.output_path, output);
    assert!(!report.output_path.join("tables/stale.md").exists());
    insta::assert_snapshot!("directory_index", index);
    insta::assert_snapshot!("directory_table", table);
}
