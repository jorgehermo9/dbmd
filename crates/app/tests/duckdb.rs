mod support;

use std::fs;

use dbmd_app::{render, RenderOutput, RenderRequest};
use duckdb::Connection;
use support::TestProject;

const SCHEMA: &str = include_str!("../../backends/duckdb/tests/fixtures/schema_surface.sql");

#[test]
fn renders_duckdb_through_configured_and_one_off_application_inputs() {
    let project = TestProject::new();
    let database_path = project.path().join("app.duckdb");
    Connection::open(&database_path)
        .expect("DuckDB fixture should open")
        .execute_batch(SCHEMA)
        .expect("DuckDB fixture should execute");
    let warehouse_path = project.path().join("warehouse.duckdb");
    Connection::open(&warehouse_path)
        .expect("attached DuckDB fixture should open")
        .execute_batch("CREATE TABLE facts (id BIGINT PRIMARY KEY);")
        .expect("attached DuckDB fixture should execute");
    fs::create_dir(project.path().join("stored-secrets"))
        .expect("secret directory should be created");
    fs::create_dir(project.path().join("extensions"))
        .expect("extension directory should be created");
    let config_path = project.path().join("dbmd.toml");
    fs::write(
        &config_path,
        r#"
[sources.analytics]
backend = "duckdb"
path = "app.duckdb"
display_name = "Analytical warehouse"
secret_directory = "stored-secrets"
extension_directory = "extensions"

[sources.analytics.attachments.warehouse]
path = "warehouse.duckdb"
read_only = true

[output]
path = "DATABASE.md"
"#,
    )
    .expect("DuckDB project config should be written");

    let first = render(RenderRequest::new(&config_path)).expect("configured render should succeed");
    let first_markdown = fs::read_to_string(
        first
            .output
            .path()
            .expect("configured render should write a file"),
    )
    .expect("configured Markdown should be readable");
    let second = render(RenderRequest::new(&config_path)).expect("repeat render should succeed");
    assert_eq!(first, second);
    assert!(first_markdown.contains("Analytical warehouse"));
    assert!(first_markdown.contains("app.analytics.accounts"));
    assert!(first_markdown.contains("warehouse.main.facts"));
    assert!(!first_markdown.contains("stored-secrets"));
    assert!(!first_markdown.contains("extensions/"));

    let one_off = render(RenderRequest::duckdb(&database_path).to_stdout())
        .expect("one-off DuckDB render should succeed");
    let RenderOutput::Stdout(one_off_markdown) = one_off.output else {
        panic!("one-off DuckDB render should return stdout");
    };
    let one_off_markdown =
        String::from_utf8(one_off_markdown).expect("one-off Markdown should be UTF-8");
    assert!(one_off_markdown.contains("app.analytics.accounts"));
    insta::assert_snapshot!("duckdb_app_render", first_markdown);
}
