use std::str::FromStr;

use dbmd_backend_duckdb::{introspect, render_source, template_files, DuckDbSource};
use dbmd_core::SourceId;
use dbmd_render::{OutputLayout, RenderContext, RenderOptions, RenderedArtifact, Renderer};
use duckdb::Connection;

#[test]
fn introspects_and_renders_the_duckdb_schema_surface_deterministically() {
    let directory = tempfile::tempdir().expect("temporary directory should be created");
    let path = directory.path().join("app.duckdb");
    let connection = Connection::open(&path).expect("DuckDB fixture should open");
    connection
        .execute_batch(include_str!("fixtures/schema_surface.sql"))
        .expect("DuckDB fixture DDL should execute");
    drop(connection);
    let warehouse_path = directory.path().join("warehouse.duckdb");
    Connection::open(&warehouse_path)
        .expect("attached DuckDB fixture should open")
        .execute_batch("CREATE TABLE facts (id BIGINT PRIMARY KEY, amount DECIMAL(18, 2));")
        .expect("attached DuckDB fixture DDL should execute");
    let source = DuckDbSource::new(
        SourceId::from_str("analytics").expect("test source ID should be valid"),
        &path,
    )
    .expect("source path should be valid")
    .with_attached_database("warehouse", &warehouse_path, true)
    .expect("attached database should be valid");

    let first = introspect(&source).expect("DuckDB introspection should succeed");
    let second = introspect(&source).expect("repeat introspection should succeed");
    assert_eq!(first, second);
    assert_eq!(first.catalog().tables.len(), 3);
    assert_eq!(first.catalog().views.len(), 1);
    assert_eq!(first.catalog().sequences.len(), 1);
    assert!(first
        .catalog()
        .databases
        .iter()
        .any(|database| { database.name == "warehouse" && database.readonly }));
    assert!(first
        .catalog()
        .tables
        .iter()
        .any(|table| { table.database == "warehouse" && table.name == "facts" }));
    assert!(first
        .catalog()
        .types
        .iter()
        .any(|value| value.name == "account_status" && value.labels == ["active", "disabled"]));
    assert!(first
        .catalog()
        .functions
        .iter()
        .any(|value| value.name == "normalize_email" && value.kind == "macro"));
    assert!(first
        .catalog()
        .functions
        .iter()
        .any(|value| value.name == "accounts_for_tenant" && value.kind == "table_macro"));
    let accounts = first
        .catalog()
        .tables
        .iter()
        .find(|table| table.name == "accounts")
        .expect("accounts table should be present");
    assert!(accounts.constraints.len() >= 5);
    assert!(accounts
        .constraints
        .iter()
        .any(|constraint| constraint.kind == "FOREIGN KEY"));
    assert!(accounts
        .constraints
        .iter()
        .any(|constraint| constraint.kind == "CHECK"));
    assert_eq!(accounts.indexes.len(), 1);
    assert!(
        accounts.columns.iter().any(
            |column| column.name == "normalized_email" && column.generated_expression.is_some()
        ),
        "columns: {:?}",
        accounts.columns
    );
    let mut stable_catalog = first.catalog().clone();
    for database in &mut stable_catalog.databases {
        database.path = database
            .path
            .as_ref()
            .map(|_| "[DATABASE PATH]".to_string());
    }
    insta::assert_yaml_snapshot!("duckdb_schema_surface", stable_catalog);

    let context = RenderContext::new(vec![render_source(
        first.id(),
        first.display_name(),
        first.catalog(),
        false,
    )]);
    let renderer = Renderer::embedded(template_files()).expect("DuckDB templates should compile");
    let RenderedArtifact::SingleFile(markdown) = renderer
        .render(&context)
        .expect("DuckDB catalog should render")
    else {
        panic!("default DuckDB rendering should produce one file");
    };
    let markdown = String::from_utf8(markdown).expect("Markdown should be UTF-8");
    assert!(markdown.contains("analytics.accounts"));
    assert!(markdown.contains("normalize_email"));
    insta::assert_snapshot!("duckdb_markdown", markdown);
    let RenderedArtifact::Directory(files) = renderer
        .render_with_options(
            &context,
            RenderOptions {
                layout: OutputLayout::Directory,
                ..RenderOptions::default()
            },
        )
        .expect("DuckDB directory profile should render")
    else {
        panic!("directory options should produce a directory artifact");
    };
    let index_path = "index.md"
        .parse()
        .expect("static artifact path should parse");
    let index = String::from_utf8(files[&index_path].clone()).expect("index should be UTF-8");
    assert!(index.contains("tables/app%2Eanalytics.accounts.md"));
    assert!(files
        .keys()
        .any(|path| path.as_str() == "tables/app%2Eanalytics.accounts.md"));
}
