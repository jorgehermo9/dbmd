use std::fs;

use dbmd_render::{
    embedded_template_files, ArtifactPath, OutputLayout, RenderContext, RenderError, RenderObject,
    RenderOptions, RenderSource, RenderedArtifact, Renderer, SourceLayout, TemplateFile,
};
use serde::Serialize;

const TEST_TEMPLATES: &[TemplateFile] = &[
    TemplateFile::new(
        "single_file/backends/test/source.md.j2",
        "backends/test/single_file/source.md.j2",
        "{% if source.nested %}## Source: `{{ source.id }}`\n\n{% endif %}{% for object in source.data.widgets %}{{ source.data.object_heading }} {{ object.qualified_name }}\n\n{{ object.definition }}\n{% endfor %}",
    ),
    TemplateFile::new(
        "directory/backends/test/source.md.j2",
        "backends/test/directory/source.md.j2",
        "# Database: {{ source.name }}\n{% for object in source.data.widgets %}- [{{ object.qualified_name }}](widgets/{{ object.file_name }})\n{% endfor %}",
    ),
    TemplateFile::new(
        "directory/backends/test/widget.md.j2",
        "backends/test/directory/widget.md.j2",
        "{{ heading }} {{ object.qualified_name }}\n\n{{ object.definition }}\n",
    ),
];

#[derive(Clone, Serialize)]
struct TestObject {
    qualified_name: String,
    file_name: String,
    definition: String,
}

#[derive(Serialize)]
struct TestData {
    object_heading: &'static str,
    widgets: Vec<TestObject>,
}

fn source(id: &str, display_name: Option<&str>, object_name: &str, nested: bool) -> RenderSource {
    let object = TestObject {
        qualified_name: format!("`main.{object_name}`"),
        file_name: format!("main.{object_name}.md"),
        definition: format!("Definition of {object_name}"),
    };
    let data = TestData {
        object_heading: if nested { "####" } else { "###" },
        widgets: vec![object.clone()],
    };
    RenderSource::builder(
        id,
        "test",
        (
            TEST_TEMPLATES[0].template_name,
            TEST_TEMPLATES[1].template_name,
        ),
        data,
    )
    .display_name(display_name.map(|name| format!("`{name}`")))
    .nested(nested)
    .objects(vec![RenderObject::new(
        format!("widgets/{}", object.file_name),
        TEST_TEMPLATES[2].template_name,
        object,
    )])
    .build()
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
fn renders_backend_declared_directory_objects_without_knowing_their_family() {
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
            "analytics/widgets/main.events.md",
            "app/index.md",
            "app/widgets/main.users.md",
            "index.md",
        ]
    );
    let object_path = "analytics/widgets/main.events.md"
        .parse::<ArtifactPath>()
        .expect("object path should be valid");
    assert_eq!(
        String::from_utf8(files[&object_path].clone()).expect("Markdown should be UTF-8"),
        "# `main.events`\n\nDefinition of events"
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
fn directory_manifests_reject_duplicate_and_reserved_paths() {
    let data = TestData {
        object_heading: "###",
        widgets: Vec::new(),
    };
    let object = TestObject {
        qualified_name: "`main.widget`".to_string(),
        file_name: "main.widget.md".to_string(),
        definition: "Definition".to_string(),
    };
    for objects in [
        vec![
            RenderObject::new(
                "widgets/main.widget.md",
                TEST_TEMPLATES[2].template_name,
                &object,
            ),
            RenderObject::new(
                "widgets/main.widget.md",
                TEST_TEMPLATES[2].template_name,
                &object,
            ),
        ],
        vec![RenderObject::new(
            "index.md",
            TEST_TEMPLATES[2].template_name,
            &object,
        )],
    ] {
        let source = RenderSource::builder(
            "app",
            "test",
            (
                TEST_TEMPLATES[0].template_name,
                TEST_TEMPLATES[1].template_name,
            ),
            &data,
        )
        .objects(objects)
        .build();
        let error = Renderer::embedded(TEST_TEMPLATES)
            .expect("embedded templates should compile")
            .render_with_options(
                &RenderContext::new(vec![source]),
                RenderOptions {
                    layout: OutputLayout::Directory,
                    source_layout: SourceLayout::Auto,
                },
            )
            .expect_err("colliding manifest paths must be rejected");
        assert!(matches!(error, RenderError::DuplicateArtifactPath(_)));
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
