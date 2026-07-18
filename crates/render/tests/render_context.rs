use std::fs;

use dbmd_render::{
    embedded_template_files, ArtifactPath, OutputLayout, RenderColumn, RenderContext, RenderEnum,
    RenderOptions, RenderSource, RenderTable, RenderTableDetails, RenderedArtifact, Renderer,
    SourceLayout, TemplateFile,
};

const TEST_TEMPLATES: &[TemplateFile] = &[
    TemplateFile {
        relative_path: "single_file/backends/test/source.md.j2",
        template_name: "backends/test/single_file/source.md.j2",
        contents: "{% if source.nested %}## Source: `{{ source.id }}`\n\n{% endif %}{% for table in source.tables %}{% include \"table.md.j2\" %}{% endfor %}",
    },
    TemplateFile {
        relative_path: "directory/backends/test/source.md.j2",
        template_name: "backends/test/directory/source.md.j2",
        contents: "# Database: {{ source.name }}\n{% for object in source.tables %}- [{{ object.qualified_name }}](tables/{{ object.file_name }})\n{% endfor %}",
    },
];

fn source(id: &str, display_name: Option<&str>, table_name: &str, nested: bool) -> RenderSource {
    RenderSource {
        id: id.to_string(),
        name: format!("`{}`", display_name.unwrap_or(id)),
        has_display_name: display_name.is_some(),
        backend: "test",
        single_file_template: TEST_TEMPLATES[0].template_name,
        directory_template: TEST_TEMPLATES[1].template_name,
        nested,
        section_heading: if nested { "###" } else { "##" },
        object_heading: if nested { "####" } else { "###" },
        detail_heading: if nested { "#####" } else { "####" },
        namespaces: Vec::new(),
        enums: Vec::new(),
        tables: vec![RenderTable {
            heading: if nested { "####" } else { "###" },
            detail_heading: if nested { "#####" } else { "####" },
            qualified_name: format!("`main.{table_name}`"),
            file_name: format!("main.{table_name}.md"),
            comment: None,
            columns: vec![RenderColumn {
                name: "`id`".to_string(),
                data_type: "`INTEGER`".to_string(),
                nullable: "no",
                default: "-".to_string(),
                notes: String::new(),
            }],
            constraints: Vec::new(),
            indexes: Vec::new(),
            backend: RenderTableDetails {
                title: "Test",
                facts: Vec::new(),
                notices: Vec::new(),
                definition: Some(format!(
                    "```sql\nCREATE TABLE {table_name} (id INTEGER)\n```"
                )),
            },
        }],
        views: Vec::new(),
        triggers: Vec::new(),
        functions: Vec::new(),
    }
}

#[test]
fn renders_multiple_backend_prepared_sources_as_one_document() {
    let context = RenderContext::new(vec![
        source("analytics", Some("Analytics"), "events", true),
        source("app", None, "users", true),
    ]);
    let artifact = Renderer::embedded(TEST_TEMPLATES)
        .expect("embedded templates should compile")
        .render(&context)
        .expect("render context should render");
    let RenderedArtifact::SingleFile(markdown) = artifact else {
        panic!("default renderer should produce one file");
    };
    let markdown = String::from_utf8(markdown).expect("rendered Markdown should be UTF-8");
    assert!(markdown.contains("## Source: `analytics`"));
    assert!(markdown.contains("#### `main.events`"));
    assert!(markdown.contains("## Source: `app`"));
}

#[test]
fn renders_directory_objects_with_validated_relative_paths() {
    let context = RenderContext::new(vec![
        source("analytics", Some("Analytics"), "events", true),
        source("app", None, "users", true),
    ]);
    let artifact = Renderer::embedded(TEST_TEMPLATES)
        .expect("embedded templates should compile")
        .render_with_options(
            &context,
            RenderOptions {
                layout: OutputLayout::Directory,
                source_layout: SourceLayout::Auto,
            },
        )
        .expect("render context should render");
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
    for file in embedded_template_files().iter().chain(TEST_TEMPLATES) {
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
    let context = RenderContext::new(vec![source("app", None, "users", false)]);
    let artifact = Renderer::from_template_root(root.path(), "agent", TEST_TEMPLATES)
        .expect("complete custom profile should load")
        .render(&context)
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
    let Err(error) = Renderer::from_template_root(root.path(), "agent", TEST_TEMPLATES) else {
        panic!("missing custom files must not fall back to embedded templates");
    };
    assert!(error.to_string().contains("directory/enum.md.j2"));
}

#[test]
fn directory_layout_renders_first_class_enum_objects() {
    let mut source = source("catalog", None, "accounts", false);
    source.enums.push(RenderEnum {
        heading: "###",
        qualified_name: "`catalog.account_state`".to_string(),
        file_name: "catalog.account_state.md".to_string(),
        comment: Some("Lifecycle state".to_string()),
        values: "`active, suspended`".to_string(),
    });
    let context = RenderContext::new(vec![source]);
    let artifact = Renderer::embedded(TEST_TEMPLATES)
        .expect("embedded templates should compile")
        .render_with_options(
            &context,
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
