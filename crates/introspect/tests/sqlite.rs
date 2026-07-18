#[path = "support/sqlite.rs"]
mod support;

use std::str::FromStr;

use dbmd_core::{ConstraintKind, ForeignKeyAction, SourceId};
use dbmd_introspect::sqlite::{introspect, SqliteSource};
use support::TestDatabase;

#[test]
fn introspects_an_ordinary_table_into_a_source_snapshot() {
    let database =
        TestDatabase::from_sql(include_str!("fixtures/sqlite/ordinary_table/schema.sql"));
    let source = SqliteSource::new(
        SourceId::from_str("app").expect("test source ID should be valid"),
        database.path(),
    );

    let snapshot = introspect(&source).expect("ordinary SQLite table should be introspected");

    insta::assert_yaml_snapshot!("ordinary_table", snapshot);
}

#[test]
fn introspects_generated_columns_and_table_storage_modes() {
    let database =
        TestDatabase::from_sql(include_str!("fixtures/sqlite/table_features/schema.sql"));
    let source = SqliteSource::new(
        SourceId::from_str("app").expect("test source ID should be valid"),
        database.path(),
    );

    let snapshot = introspect(&source).expect("SQLite table features should be introspected");

    insta::assert_yaml_snapshot!("table_features", snapshot);
}

#[test]
fn introspects_composite_foreign_keys_and_referential_actions() {
    let database = TestDatabase::from_sql(include_str!("fixtures/sqlite/relationships/schema.sql"));
    let source = SqliteSource::new(
        SourceId::from_str("app").expect("test source ID should be valid"),
        database.path(),
    );

    let snapshot = introspect(&source).expect("SQLite relationships should be introspected");
    let tracks = snapshot
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
    let database = TestDatabase::from_sql(include_str!("fixtures/sqlite/indexes/schema.sql"));
    let source = SqliteSource::new(
        SourceId::from_str("app").expect("test source ID should be valid"),
        database.path(),
    );

    let snapshot = introspect(&source).expect("SQLite indexes should be introspected");
    let table = snapshot
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
    let database =
        TestDatabase::from_sql(include_str!("fixtures/sqlite/table_definition/schema.sql"));
    let source = SqliteSource::new(
        SourceId::from_str("app").expect("test source ID should be valid"),
        database.path(),
    );

    let snapshot = introspect(&source).expect("SQLite table definitions should be introspected");
    let accounts = snapshot
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
    let database =
        TestDatabase::from_sql(include_str!("fixtures/sqlite/schema_objects/schema.sql"));
    let source = SqliteSource::new(
        SourceId::from_str("app").expect("test source ID should be valid"),
        database.path(),
    );

    let snapshot = introspect(&source).expect("SQLite schema objects should be introspected");

    assert_eq!(snapshot.views.len(), 2);
    let derived_view = snapshot
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
    assert_eq!(snapshot.triggers.len(), 3);
    assert!(snapshot
        .tables
        .iter()
        .any(|table| table.name == "document_search"));
    insta::assert_yaml_snapshot!("schema_objects", snapshot);
}

#[test]
fn introspects_configured_attached_database_namespaces_in_order() {
    let main = TestDatabase::from_sql(include_str!("fixtures/sqlite/namespaces/main.sql"));
    let analytics =
        TestDatabase::from_sql(include_str!("fixtures/sqlite/namespaces/analytics.sql"));
    let archive = TestDatabase::from_sql(include_str!("fixtures/sqlite/namespaces/archive.sql"));
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
    let database =
        TestDatabase::from_sql(include_str!("fixtures/sqlite/schema_evolution/schema.sql"));
    let source = SqliteSource::new(
        SourceId::from_str("app").expect("test source ID should be valid"),
        database.path(),
    );

    let snapshot = introspect(&source).expect("evolved SQLite schema should be introspected");

    assert_eq!(
        snapshot
            .tables
            .iter()
            .map(|table| table.name.as_str())
            .collect::<Vec<_>>(),
        ["imported_records", "records"]
    );
    assert!(snapshot.views.is_empty());
    assert!(snapshot.triggers.is_empty());
    insta::assert_yaml_snapshot!("schema_evolution", snapshot);
}
