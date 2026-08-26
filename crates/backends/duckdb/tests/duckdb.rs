use std::str::FromStr;

use dbmd_backend_duckdb::{
    introspect, render_source, template_files, ConstraintKind, DuckDbSource, DuckDbSourceError,
    FunctionKind,
};
use dbmd_core::SourceId;
use dbmd_render::{OutputLayout, RenderContext, RenderOptions, RenderedArtifact, Renderer};
use duckdb::{Config as DuckDbConnectionConfig, Connection};

#[test]
fn source_configuration_rejects_empty_reserved_duplicate_and_nul_values() {
    let id = || SourceId::from_str("app").expect("test source ID should be valid");

    assert!(matches!(
        DuckDbSource::new(id(), ""),
        Err(DuckDbSourceError::EmptyPath)
    ));
    assert!(matches!(
        DuckDbSource::new(id(), "app.duckdb")
            .expect("base source should be valid")
            .with_attached_database("", "analytics.duckdb", true),
        Err(DuckDbSourceError::EmptyAttachmentName)
    ));
    assert!(matches!(
        DuckDbSource::new(id(), "app.duckdb")
            .expect("base source should be valid")
            .with_attached_database("system", "analytics.duckdb", true),
        Err(DuckDbSourceError::ReservedAttachmentName(name)) if name == "system"
    ));
    assert!(matches!(
        DuckDbSource::new(id(), "app.duckdb")
            .expect("base source should be valid")
            .with_attached_database("analytics", "analytics.duckdb", true)
            .expect("first attachment should be valid")
            .with_attached_database("analytics", "other.duckdb", true),
        Err(DuckDbSourceError::DuplicateAttachmentName(name)) if name == "analytics"
    ));
    assert!(matches!(
        DuckDbSource::new(id(), "app.duckdb")
            .expect("base source should be valid")
            .with_attached_database("analytics", "", true),
        Err(DuckDbSourceError::EmptyAttachmentPath(name)) if name == "analytics"
    ));
    assert!(matches!(
        DuckDbSource::new(id(), "app.duckdb")
            .expect("base source should be valid")
            .with_secret_directory(""),
        Err(DuckDbSourceError::EmptySecretDirectory)
    ));
    assert!(matches!(
        DuckDbSource::new(id(), "app.duckdb")
            .expect("base source should be valid")
            .with_extension_directory("bad\0directory"),
        Err(DuckDbSourceError::NulExtensionDirectory)
    ));
}

#[test]
fn missing_database_and_attachment_errors_are_source_scoped_without_directory_details() {
    let directory = tempfile::tempdir().expect("temporary directory should be created");
    let missing = directory.path().join("sentinel-missing-main.duckdb");
    let source = DuckDbSource::new(
        SourceId::from_str("warehouse").expect("test source ID should be valid"),
        &missing,
    )
    .expect("missing path is structurally valid");

    let error = introspect(&source).expect_err("read-only missing database should fail");
    assert!(error.to_string().contains("DuckDB source `warehouse`"));
    assert!(!error.to_string().contains("sentinel-missing-main"));

    let main = directory.path().join("main.duckdb");
    Connection::open(&main).expect("main fixture database should open");
    let source = DuckDbSource::new(
        SourceId::from_str("warehouse").expect("test source ID should be valid"),
        main,
    )
    .expect("main source should be valid")
    .with_attached_database(
        "analytics",
        directory.path().join("sentinel-missing-attachment.duckdb"),
        true,
    )
    .expect("attachment configuration should be structurally valid");

    let error = introspect(&source).expect_err("read-only missing attachment should fail");
    assert!(error
        .to_string()
        .contains("DuckDB database `analytics` for source `warehouse`"));
    assert!(!error.to_string().contains("sentinel-missing-attachment"));
}

#[test]
fn introspects_and_renders_the_duckdb_schema_surface_deterministically() {
    let directory = tempfile::tempdir().expect("temporary directory should be created");
    let path = directory.path().join("app.duckdb");
    let connection = Connection::open(&path).expect("DuckDB fixture should open");
    let version: String = connection
        .query_row("SELECT version()", [], |row| row.get(0))
        .expect("DuckDB version should be queryable");
    assert_eq!(version, "v1.5.4");
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
    assert!(
        first.catalog().types.iter().any(|value| {
            value.name == "account_pair"
                && value.definition == "STRUCT(account_id BIGINT, tenant_id BIGINT)"
        }),
        "types: {:?}",
        first.catalog().types
    );
    assert!(
        first.catalog().types.iter().any(|value| {
            value.name == "reference_value"
                && value.definition == "UNION(account_id BIGINT, external_id VARCHAR)"
        }),
        "types: {:?}",
        first.catalog().types
    );
    assert!(first
        .catalog()
        .types
        .iter()
        .any(|value| value.name == "positive_integer" && value.logical_type == "INTEGER"));
    assert!(first
        .catalog()
        .functions
        .iter()
        .any(|value| value.name == "normalize_email" && value.kind == FunctionKind::Macro));
    assert!(
        first
            .catalog()
            .functions
            .iter()
            .any(|value| value.name == "accounts_for_tenant"
                && value.kind == FunctionKind::TableMacro)
    );
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
        .any(|constraint| constraint.kind == ConstraintKind::ForeignKey
            && constraint.referenced_table.as_deref() == Some("tenants")
            && constraint.referenced_columns == ["tenant_id"]));
    assert!(accounts
        .constraints
        .iter()
        .any(|constraint| constraint.kind == ConstraintKind::Check));
    assert_eq!(accounts.indexes.len(), 1);
    assert_eq!(accounts.indexes[0].index_type, "ART");
    assert!(
        accounts.columns.iter().any(
            |column| column.name == "normalized_email" && column.generated_expression.is_some()
        ),
        "columns: {:?}",
        accounts.columns
    );
    let balance = accounts
        .columns
        .iter()
        .find(|column| column.name == "balance")
        .expect("balance column should be present");
    assert_eq!(balance.numeric_precision, Some(18));
    assert_eq!(balance.numeric_precision_radix, Some(10));
    assert_eq!(balance.numeric_scale, Some(2));
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
    let repeated = renderer
        .render(&context)
        .expect("repeat DuckDB presentation should render");
    assert_eq!(repeated.as_single_file(), Some(markdown.as_bytes()));
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

#[test]
fn introspects_persistent_secret_metadata_without_secret_material() {
    let directory = tempfile::tempdir().expect("temporary project should be created");
    let database_path = directory.path().join("secrets.duckdb");
    let secret_directory = directory.path().join("stored-secrets");
    let extension_directory = directory.path().join("extensions");
    std::fs::create_dir(&secret_directory).expect("secret directory should be created");
    std::fs::create_dir(&extension_directory).expect("extension directory should be created");
    let connection_config = DuckDbConnectionConfig::default()
        .with("extension_directory", extension_directory.to_string_lossy())
        .expect("extension directory should be valid");
    let connection = Connection::open_with_flags(&database_path, connection_config)
        .expect("DuckDB database should open");
    let escaped_directory = secret_directory.to_string_lossy().replace('\'', "''");
    connection
        .execute_batch(&format!(
            "SET secret_directory = '{escaped_directory}';
             CREATE PERSISTENT SECRET agent_storage (
                 TYPE s3,
                 KEY_ID 'dbmd-key-sentinel',
                 SECRET 'dbmd-secret-sentinel',
                 REGION 'eu-west-1',
                 SCOPE 's3://dbmd-fixture'
             );"
        ))
        .expect("persistent DuckDB secret fixture should be created");
    drop(connection);

    let source = DuckDbSource::new(
        SourceId::from_str("secrets").expect("test source ID should be valid"),
        &database_path,
    )
    .expect("DuckDB source should be valid")
    .with_secret_directory(&secret_directory)
    .expect("secret directory should be valid")
    .with_extension_directory(&extension_directory)
    .expect("extension directory should be valid");
    let snapshot = introspect(&source).expect("DuckDB secret metadata should be introspected");

    assert_eq!(snapshot.catalog().secrets.len(), 1);
    let secret = &snapshot.catalog().secrets[0];
    assert_eq!(secret.name, "agent_storage");
    assert_eq!(secret.secret_type, "s3");
    assert_eq!(secret.provider, "config");
    assert!(secret.persistent);
    assert_eq!(secret.scope, ["s3://dbmd-fixture"]);
    let serialized = serde_json::to_string(&snapshot).expect("catalog should serialize");
    assert!(!serialized.contains("dbmd-key-sentinel"));
    assert!(!serialized.contains("dbmd-secret-sentinel"));

    let context = RenderContext::new(vec![render_source(
        snapshot.id(),
        snapshot.display_name(),
        snapshot.catalog(),
        false,
    )]);
    let renderer = Renderer::embedded(template_files()).expect("DuckDB templates should compile");
    let RenderedArtifact::SingleFile(markdown) = renderer
        .render(&context)
        .expect("DuckDB secret metadata should render")
    else {
        panic!("default DuckDB rendering should produce one file");
    };
    let markdown = String::from_utf8(markdown).expect("Markdown should be UTF-8");
    assert!(markdown.contains("agent_storage"));
    assert!(markdown.contains("s3://dbmd-fixture"));
    assert!(!markdown.contains("dbmd-key-sentinel"));
    assert!(!markdown.contains("dbmd-secret-sentinel"));
    insta::assert_snapshot!("duckdb_secret_metadata", markdown);
    let repeated = renderer
        .render(&context)
        .expect("repeat DuckDB secret presentation should render");
    assert_eq!(repeated.as_single_file(), Some(markdown.as_bytes()));
}
