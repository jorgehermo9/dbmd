use std::{fs, str::FromStr};

use dbmd_core::{
    Backend, Column, ColumnBackend, DatabaseContext, SourceId, SourceSnapshot, SqliteColumn,
    SqliteColumnKind, SqliteTable, SqliteTableKind, Table, TableBackend,
};
use dbmd_render::{
    embedded_template_files, ArtifactPath, OutputLayout, RenderContext, RenderOptions,
    RenderedArtifact, Renderer, SourceLayout,
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

#[test]
fn custom_template_root_is_a_complete_independent_profile() {
    let root = tempfile::tempdir().expect("template root should be created");
    for file in embedded_template_files() {
        let path = root.path().join("agent").join(file.relative_path);
        fs::create_dir_all(path.parent().expect("template should have a parent"))
            .expect("template directories should be created");
        let contents = if file.template_name == "database.md.j2" {
            "# Custom database for `{{ context.sources[0].id }}`\n"
        } else {
            file.contents
        };
        fs::write(path, contents).expect("custom template should be written");
    }
    let database = DatabaseContext::new(vec![source("app", None, "users")])
        .expect("source should form a database context");

    let artifact = Renderer::from_template_root(root.path(), "agent")
        .expect("complete custom profile should load")
        .render(&database)
        .expect("custom profile should render");
    let RenderedArtifact::SingleFile(markdown) = artifact else {
        panic!("custom single-file profile should produce one file");
    };

    assert_eq!(
        String::from_utf8(markdown).expect("Markdown should be UTF-8"),
        "# Custom database for `app`"
    );
}

#[test]
fn custom_template_root_does_not_fall_back_to_embedded_files() {
    let root = tempfile::tempdir().expect("template root should be created");
    let database_template = root.path().join("agent/single_file/database.md.j2");
    fs::create_dir_all(
        database_template
            .parent()
            .expect("template should have a parent"),
    )
    .expect("template directory should be created");
    fs::write(database_template, "# Incomplete\n").expect("template should be written");

    let Err(error) = Renderer::from_template_root(root.path(), "agent") else {
        panic!("missing custom files must not fall back to embedded templates");
    };

    assert!(error.to_string().contains("directory/enum.md.j2"));
}

#[test]
fn directory_layout_renders_first_class_enum_objects() {
    let mut source = source("catalog", None, "accounts");
    source.enums.push(dbmd_core::EnumType {
        namespace: "catalog".to_string(),
        name: "account_state".to_string(),
        comment: Some("Lifecycle state".to_string()),
        values: vec!["active".to_string(), "suspended".to_string()],
    });
    let database = DatabaseContext::new(vec![source]).expect("source should form a context");

    let artifact = Renderer::embedded()
        .expect("embedded templates should compile")
        .render_with_options(
            &database,
            RenderOptions {
                layout: OutputLayout::Directory,
                source_layout: SourceLayout::Auto,
            },
        )
        .expect("enum directory should render");
    let RenderedArtifact::Directory(files) = artifact else {
        panic!("directory options should produce a directory artifact");
    };
    let path = "enums/catalog.account_state.md"
        .parse::<ArtifactPath>()
        .expect("enum artifact path should be valid");
    let markdown = String::from_utf8(files[&path].clone()).expect("Markdown should be UTF-8");

    assert!(markdown.contains("# `catalog.account_state`"));
    assert!(markdown.contains("Values: `active, suspended`"));
}
