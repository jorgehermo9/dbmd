#[path = "support/sqlite.rs"]
mod support;

use std::str::FromStr;

use dbmd_backend_sqlite::{
    introspect, ConstraintKind, SqliteSource, SqliteSourceError, TriggerTiming,
};
use dbmd_core::SourceId;
use dbmd_relational::ForeignKeyAction;
use support::TestDatabase;

fn source_id() -> SourceId {
    SourceId::from_str("app").expect("test source ID should be valid")
}

#[test]
fn attachment_configuration_rejects_an_empty_namespace() {
    assert!(matches!(
        SqliteSource::new(source_id(), "app.db").with_attached_database("", "analytics.db"),
        Err(SqliteSourceError::EmptyNamespace)
    ));
}

#[test]
fn attachment_configuration_rejects_the_main_namespace_case_insensitively() {
    assert!(matches!(
        SqliteSource::new(source_id(), "app.db").with_attached_database("MAIN", "analytics.db"),
        Err(SqliteSourceError::ReservedNamespace(name)) if name == "MAIN"
    ));
}

#[test]
fn attachment_configuration_rejects_the_temp_namespace_case_insensitively() {
    assert!(matches!(
        SqliteSource::new(source_id(), "app.db").with_attached_database("TeMp", "analytics.db"),
        Err(SqliteSourceError::ReservedNamespace(name)) if name == "TeMp"
    ));
}

#[test]
fn attachment_configuration_rejects_duplicate_namespaces_case_insensitively() {
    assert!(matches!(
        SqliteSource::new(source_id(), "app.db")
            .with_attached_database("analytics", "analytics.db")
            .expect("first attachment should be valid")
            .with_attached_database("ANALYTICS", "other.db"),
        Err(SqliteSourceError::DuplicateNamespace(name)) if name == "ANALYTICS"
    ));
}

#[test]
fn attachment_configuration_rejects_a_nul_namespace() {
    assert!(matches!(
        SqliteSource::new(source_id(), "app.db")
            .with_attached_database("bad\0namespace", "analytics.db"),
        Err(SqliteSourceError::NamespaceContainsNul)
    ));
}

#[test]
fn missing_database_and_attachment_errors_are_source_scoped_and_path_free() {
    let directory = tempfile::tempdir().expect("temporary directory should be created");
    let source = SqliteSource::new(
        SourceId::from_str("app").expect("test source ID should be valid"),
        directory.path().join("sentinel-missing-main.db"),
    );

    let error = introspect(&source).expect_err("missing read-only database should fail");
    assert!(error.to_string().contains("SQLite source `app`"));
    assert!(!error.to_string().contains("sentinel-missing-main"));

    let main = TestDatabase::from_sql("CREATE TABLE items (id INTEGER PRIMARY KEY);");
    let source = SqliteSource::new(
        SourceId::from_str("app").expect("test source ID should be valid"),
        main.path(),
    )
    .with_attached_database(
        "analytics",
        directory.path().join("sentinel-missing-attachment.db"),
    )
    .expect("attachment configuration should be structurally valid");

    let error = introspect(&source).expect_err("missing attachment should fail");
    assert!(error
        .to_string()
        .contains("SQLite namespace `analytics` for source `app`"));
    assert!(!error.to_string().contains("sentinel-missing-attachment"));
}

#[test]
fn introspects_an_ordinary_table_into_a_source_snapshot() {
    let database = TestDatabase::from_sql(include_str!("fixtures/ordinary_table/schema.sql"));
    let source = SqliteSource::new(
        SourceId::from_str("app").expect("test source ID should be valid"),
        database.path(),
    );

    let snapshot = introspect(&source).expect("ordinary SQLite table should be introspected");

    insta::assert_yaml_snapshot!("ordinary_table", snapshot);
}

#[test]
fn introspects_generated_columns_and_table_storage_modes() {
    let database = TestDatabase::from_sql(include_str!("fixtures/table_features/schema.sql"));
    let source = SqliteSource::new(
        SourceId::from_str("app").expect("test source ID should be valid"),
        database.path(),
    );

    let snapshot = introspect(&source).expect("SQLite table features should be introspected");

    insta::assert_yaml_snapshot!("table_features", snapshot);
}

#[test]
fn introspects_composite_foreign_keys_and_referential_actions() {
    let database = TestDatabase::from_sql(include_str!("fixtures/relationships/schema.sql"));
    let source = SqliteSource::new(
        SourceId::from_str("app").expect("test source ID should be valid"),
        database.path(),
    );

    let snapshot = introspect(&source).expect("SQLite relationships should be introspected");
    let tracks = snapshot
        .catalog()
        .tables
        .iter()
        .find(|table| table.name == "tracks")
        .expect("tracks table should be present");
    let foreign_key = tracks
        .constraints
        .iter()
        .find(|constraint| constraint.kind == ConstraintKind::ForeignKey)
        .expect("tracks should contain its composite foreign key");
    let reference = foreign_key
        .references
        .as_ref()
        .expect("foreign key should contain its target");

    assert_eq!(foreign_key.columns, ["album_artist", "album_title"]);
    assert_eq!(reference.table, "albums");
    assert_eq!(reference.columns, ["artist", "title"]);
    assert_eq!(reference.on_update, ForeignKeyAction::Cascade);
    assert_eq!(reference.on_delete, ForeignKeyAction::Restrict);
    insta::assert_yaml_snapshot!("relationships", snapshot);
}

#[test]
fn introspects_explicit_and_constraint_backed_indexes() {
    let database = TestDatabase::from_sql(include_str!("fixtures/indexes/schema.sql"));
    let source = SqliteSource::new(
        SourceId::from_str("app").expect("test source ID should be valid"),
        database.path(),
    );

    let snapshot = introspect(&source).expect("SQLite indexes should be introspected");
    let table = snapshot
        .catalog()
        .tables
        .first()
        .expect("catalog_items table should be present");

    assert_eq!(table.indexes.len(), 3);
    assert!(table
        .constraints
        .iter()
        .any(|constraint| constraint.kind == ConstraintKind::Unique));
    insta::assert_yaml_snapshot!("indexes", snapshot);
}

#[test]
fn introspects_semantics_preserved_only_in_table_definitions() {
    let database = TestDatabase::from_sql(include_str!("fixtures/table_definition/schema.sql"));
    let source = SqliteSource::new(
        SourceId::from_str("app").expect("test source ID should be valid"),
        database.path(),
    );

    let snapshot = introspect(&source).expect("SQLite table definitions should be introspected");
    let accounts = snapshot
        .catalog()
        .tables
        .iter()
        .find(|table| table.name == "accounts")
        .expect("accounts table should be present");

    assert_eq!(
        accounts
            .constraints
            .iter()
            .filter(|constraint| constraint.kind == ConstraintKind::Check)
            .count(),
        2
    );
    assert!(accounts
        .constraints
        .iter()
        .any(|constraint| constraint.name.as_deref() == Some("accounts_pk")));
    insta::assert_yaml_snapshot!("table_definition", snapshot);
}

#[test]
fn introspects_views_triggers_and_virtual_table_families() {
    let database = TestDatabase::from_sql(include_str!("fixtures/schema_objects/schema.sql"));
    let source = SqliteSource::new(
        SourceId::from_str("app").expect("test source ID should be valid"),
        database.path(),
    );

    let snapshot = introspect(&source).expect("SQLite schema objects should be introspected");

    assert_eq!(snapshot.catalog().views.len(), 2);
    let derived_view = snapshot
        .catalog()
        .views
        .iter()
        .find(|view| view.name == "document_titles")
        .expect("view with catalog-derived columns should be present");
    assert_eq!(
        derived_view
            .columns
            .iter()
            .map(|column| (column.name.as_str(), column.nullable))
            .collect::<Vec<_>>(),
        [("id", None), ("title", None), ("body_length", None)]
    );
    assert_eq!(snapshot.catalog().triggers.len(), 3);
    assert!(snapshot
        .catalog()
        .tables
        .iter()
        .any(|table| table.name == "document_search"));
    insta::assert_yaml_snapshot!("schema_objects", snapshot);
}

#[test]
fn introspects_configured_attached_database_namespaces_in_order() {
    let main = TestDatabase::from_sql(include_str!("fixtures/namespaces/main.sql"));
    let analytics = TestDatabase::from_sql(include_str!("fixtures/namespaces/analytics.sql"));
    let archive = TestDatabase::from_sql(include_str!("fixtures/namespaces/archive.sql"));
    let source = SqliteSource::new(
        SourceId::from_str("app").expect("test source ID should be valid"),
        main.path(),
    )
    .with_attached_database("analytics", analytics.path())
    .expect("analytics should be a valid attached namespace")
    .with_attached_database("archive", archive.path())
    .expect("archive should be a valid attached namespace");

    let snapshot = introspect(&source).expect("attached SQLite databases should be introspected");

    assert_eq!(
        snapshot
            .catalog()
            .tables
            .iter()
            .map(|table| table.namespace.as_str())
            .collect::<Vec<_>>(),
        ["main", "analytics", "archive"]
    );
    insta::assert_yaml_snapshot!("namespaces", snapshot);
}

#[test]
fn introspects_the_persisted_result_of_alter_create_as_and_drop() {
    let database = TestDatabase::from_sql(include_str!("fixtures/schema_evolution/schema.sql"));
    let source = SqliteSource::new(
        SourceId::from_str("app").expect("test source ID should be valid"),
        database.path(),
    );

    let snapshot = introspect(&source).expect("evolved SQLite schema should be introspected");

    assert_eq!(
        snapshot
            .catalog()
            .tables
            .iter()
            .map(|table| table.name.as_str())
            .collect::<Vec<_>>(),
        ["imported_records", "records"]
    );
    assert!(snapshot.catalog().views.is_empty());
    assert!(snapshot.catalog().triggers.is_empty());
    insta::assert_yaml_snapshot!("schema_evolution", snapshot);
}

#[test]
fn introspects_quoted_grammar_edges_without_hiding_sqlite_prefixed_user_objects() {
    let database = TestDatabase::from_sql(include_str!("fixtures/grammar_edges/schema.sql"));
    let source = SqliteSource::new(
        SourceId::from_str("app").expect("test source ID should be valid"),
        database.path(),
    );

    let snapshot = introspect(&source).expect("SQLite grammar edges should be introspected");
    let child = snapshot
        .catalog()
        .tables
        .iter()
        .find(|table| table.name == "child refs")
        .expect("quoted child table should be present");
    let foreign_keys = child
        .constraints
        .iter()
        .filter(|constraint| constraint.kind == ConstraintKind::ForeignKey)
        .collect::<Vec<_>>();
    assert_eq!(foreign_keys.len(), 2);
    assert_eq!(
        foreign_keys[0]
            .references
            .as_ref()
            .expect("first foreign key should have a target")
            .columns,
        ["code a"]
    );
    assert_eq!(
        foreign_keys[1]
            .references
            .as_ref()
            .expect("second foreign key should have a target")
            .columns,
        ["code b"]
    );

    let ascending = snapshot
        .catalog()
        .tables
        .iter()
        .find(|table| table.name == "sqliteXascending")
        .expect("non-reserved sqliteX-prefixed table should be visible");
    assert_eq!(ascending.columns[0].nullable, Some(false));
    let descending = snapshot
        .catalog()
        .tables
        .iter()
        .find(|table| table.name == "sqliteXdescending")
        .expect("descending primary-key table should be visible");
    assert_eq!(descending.columns[0].nullable, Some(true));
    assert!(snapshot
        .catalog()
        .views
        .iter()
        .any(|view| view.name == "sqliteXview"));
    let trigger = snapshot
        .catalog()
        .triggers
        .iter()
        .find(|trigger| trigger.name == "sqliteXdefault_timing")
        .expect("non-reserved sqliteX-prefixed trigger should be visible");
    assert_eq!(trigger.timing, TriggerTiming::Before);

    insta::assert_yaml_snapshot!("grammar_edges", snapshot);
}
