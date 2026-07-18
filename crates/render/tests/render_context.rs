use std::str::FromStr;

use dbmd_core::{
    Backend, Column, ColumnBackend, DatabaseContext, SourceId, SourceSnapshot, SqliteColumn,
    SqliteColumnKind, SqliteTable, SqliteTableKind, Table, TableBackend,
};
use dbmd_render::{
    ArtifactPath, OutputLayout, RenderContext, RenderOptions, RenderedArtifact, Renderer,
    SourceLayout,
};

fn source(id: &str, display_name: Option<&str>, table_name: &str) -> SourceSnapshot {
    let mut source = SourceSnapshot::new(
        SourceId::from_str(id).expect("test source ID should be valid"),
        Backend::Sqlite,
    );
    source.display_name = display_name.map(str::to_string);
    source.tables.push(Table {
        namespace: "main".to_string(),
        name: table_name.to_string(),
        comment: None,
        columns: vec![Column {
            name: "id".to_string(),
            data_type: "INTEGER".to_string(),
            nullable: Some(false),
            default: None,
            comment: None,
            backend: ColumnBackend::Sqlite(SqliteColumn {
                kind: SqliteColumnKind::Normal,
                collation: "BINARY".to_string(),
                generated_expression: None,
            }),
        }],
        constraints: Vec::new(),
        indexes: Vec::new(),
        backend: TableBackend::Sqlite(SqliteTable {
            without_rowid: false,
            strict: false,
            definition: Some(format!("CREATE TABLE {table_name} (id INTEGER)")),
            kind: SqliteTableKind::Ordinary,
        }),
    });
    source
}

#[test]
fn builds_a_dedicated_context_for_ordered_sources() {
    let database = DatabaseContext::new(vec![
        source("analytics", Some("Analytics"), "events"),
        source("app", None, "users"),
    ])
    .expect("distinct sources should form a database context");

    let context = RenderContext::from(&database);

    insta::assert_yaml_snapshot!("ordered_sources", context);
}

#[test]
fn renders_multiple_sources_as_one_document() {
    let database = DatabaseContext::new(vec![
        source("analytics", Some("Analytics"), "events"),
        source("app", None, "users"),
    ])
    .expect("distinct sources should form a database context");

    let artifact = Renderer::embedded()
        .expect("embedded templates should compile")
        .render(&database)
        .expect("database context should render");
    let RenderedArtifact::SingleFile(markdown) = artifact else {
        panic!("default renderer should produce one file");
    };
    let markdown = String::from_utf8(markdown).expect("rendered Markdown should be UTF-8");

    insta::assert_snapshot!("multiple_sources", markdown);
}

#[test]
fn renders_directory_objects_with_validated_relative_paths() {
    let database = DatabaseContext::new(vec![
        source("analytics", Some("Analytics"), "events"),
        source("app", None, "users"),
    ])
    .expect("distinct sources should form a database context");
    let options = RenderOptions {
        layout: OutputLayout::Directory,
        source_layout: SourceLayout::Auto,
    };

    let artifact = Renderer::embedded()
        .expect("embedded templates should compile")
        .render_with_options(&database, options)
        .expect("database context should render");
    let RenderedArtifact::Directory(files) = artifact else {
        panic!("directory options should produce a directory artifact");
    };

    assert_eq!(
        files.keys().map(ArtifactPath::as_str).collect::<Vec<_>>(),
        [
            "analytics/index.md",
            "analytics/tables/main.events.md",
            "app/index.md",
            "app/tables/main.users.md",
            "index.md",
        ]
    );
    insta::assert_snapshot!(
        "directory_index",
        String::from_utf8(files[&"index.md".parse().expect("path should be valid")].clone())
            .expect("rendered Markdown should be UTF-8")
    );
}

#[test]
fn artifact_paths_reject_absolute_and_parent_traversal() {
    for invalid in ["", "/index.md", "../index.md", "tables/../../index.md"] {
        assert!(
            invalid.parse::<ArtifactPath>().is_err(),
            "accepted {invalid}"
        );
    }
}
