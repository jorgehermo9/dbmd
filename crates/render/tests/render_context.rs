use std::fs;

use dbmd_render::{
    embedded_template_files, ArtifactPath, OutputLayout, RenderContext, RenderError, RenderObject,
    RenderOptions, RenderSource, RenderedArtifact, Renderer, SourceLayout, TemplateFile,
};
use serde::Serialize;
use serde_json::Value;

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
fn one_source_auto_and_nested_layouts_have_distinct_single_file_and_directory_shapes() {
    let renderer = Renderer::embedded(TEST_TEMPLATES).expect("templates should compile");
    let auto_context = RenderContext::new(vec![source("app", None, "users", false)]);
    let nested_context = RenderContext::new(vec![source("app", None, "users", true)]);

    let RenderedArtifact::SingleFile(auto_file) = renderer
        .render(&auto_context)
        .expect("one-source auto file should render")
    else {
        panic!("default options should render one file");
    };
    let RenderedArtifact::SingleFile(nested_file) = renderer
        .render_with_options(
            &nested_context,
            RenderOptions {
                source_layout: SourceLayout::Nested,
                ..RenderOptions::default()
            },
        )
        .expect("one-source nested file should render")
    else {
        panic!("nested single-file options should render one file");
    };
    assert!(!String::from_utf8(auto_file)
        .expect("auto Markdown should be UTF-8")
        .contains("Source: `app`"));
    assert!(String::from_utf8(nested_file)
        .expect("nested Markdown should be UTF-8")
        .contains("Source: `app`"));

    let RenderedArtifact::Directory(auto_directory) = renderer
        .render_with_options(
            &auto_context,
            RenderOptions {
                layout: OutputLayout::Directory,
                source_layout: SourceLayout::Auto,
            },
        )
        .expect("one-source auto directory should render")
    else {
        panic!("directory options should render a directory");
    };
    let RenderedArtifact::Directory(nested_directory) = renderer
        .render_with_options(
            &nested_context,
            RenderOptions {
                layout: OutputLayout::Directory,
                source_layout: SourceLayout::Nested,
            },
        )
        .expect("one-source nested directory should render")
    else {
        panic!("directory options should render a directory");
    };
    assert_eq!(
        auto_directory
            .keys()
            .map(ArtifactPath::as_str)
            .collect::<Vec<_>>(),
        ["index.md", "widgets/main.users.md"]
    );
    assert_eq!(
        nested_directory
            .keys()
            .map(ArtifactPath::as_str)
            .collect::<Vec<_>>(),
        ["app/index.md", "app/widgets/main.users.md", "index.md"]
    );
}

#[test]
fn artifact_paths_reject_absolute_and_parent_traversal() {
    for invalid in [
        "",
        "/index.md",
        "../index.md",
        "tables/../../index.md",
        "./index.md",
        "tables//index.md",
        "tables\\index.md",
        "tables/index.md\0ignored",
    ] {
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

#[test]
fn render_context_serializes_the_version_two_common_envelope() {
    let context = RenderContext::new(vec![source(
        "analytics",
        Some("Analytics"),
        "events",
        false,
    )]);

    let serialized = serde_json::to_value(&context).expect("render context should serialize");
    let source = &serialized["sources"][0];

    assert_eq!(serialized["version"], Value::from(2));
    assert_eq!(source["id"], "analytics");
    assert_eq!(source["name"], "`Analytics`");
    assert_eq!(source["has_display_name"], true);
    assert_eq!(source["backend"], "test");
    assert_eq!(
        source["single_file_template"],
        "backends/test/single_file/source.md.j2"
    );
    assert_eq!(
        source["directory_template"],
        "backends/test/directory/source.md.j2"
    );
    assert_eq!(source["nested"], false);
    assert!(source.get("data").is_some());
}

#[test]
fn source_layout_requires_backend_context_to_match_the_resolved_cardinality() {
    let renderer = Renderer::embedded(TEST_TEMPLATES).expect("templates should compile");
    let cases = [
        (
            "single auto expects no nesting",
            vec![source("app", None, "users", true)],
            RenderOptions::default(),
        ),
        (
            "single nested expects nesting",
            vec![source("app", None, "users", false)],
            RenderOptions {
                source_layout: SourceLayout::Nested,
                ..RenderOptions::default()
            },
        ),
        (
            "multiple auto expects nesting",
            vec![
                source("app", None, "users", false),
                source("analytics", None, "events", false),
            ],
            RenderOptions::default(),
        ),
    ];

    for (case, sources, options) in cases {
        let error = renderer
            .render_with_options(&RenderContext::new(sources), options)
            .expect_err(case);
        assert!(
            matches!(error, RenderError::InconsistentSourceLayout),
            "{case}: {error}"
        );
    }
}

#[test]
fn strict_undefined_template_values_fail_with_template_and_line_context() {
    const INVALID_TEMPLATES: &[TemplateFile] = &[
        TemplateFile::new(
            "single_file/backends/test/source.md.j2",
            "backends/test/single_file/source.md.j2",
            "{{ source.data.missing_value }}",
        ),
        TEST_TEMPLATES[1],
        TEST_TEMPLATES[2],
    ];
    let renderer = Renderer::embedded(INVALID_TEMPLATES).expect("templates should compile");
    let error = renderer
        .render(&RenderContext::new(vec![source(
            "app", None, "users", false,
        )]))
        .expect_err("undefined backend data must fail loudly");
    let message = error.to_string();

    assert!(
        message.contains("backends/test/single_file/source.md.j2"),
        "{message}"
    );
    assert!(message.contains("could not render include"), "{message}");
    assert!(message.contains("database.md.j2:9"), "{message}");
}

#[test]
fn invalid_template_syntax_reports_the_owning_template_and_line() {
    const INVALID_TEMPLATES: &[TemplateFile] = &[
        TemplateFile::new(
            "single_file/backends/test/source.md.j2",
            "backends/test/single_file/source.md.j2",
            "line one\n{% if source %}",
        ),
        TEST_TEMPLATES[1],
        TEST_TEMPLATES[2],
    ];

    let error = Renderer::embedded(INVALID_TEMPLATES)
        .err()
        .expect("invalid template syntax should fail compilation");
    let message = error.to_string();

    assert!(
        message.contains("backends/test/single_file/source.md.j2"),
        "{message}"
    );
    assert!(message.contains(":2)"), "{message}");
}

#[test]
fn custom_profile_names_cannot_escape_or_ambiguously_address_the_root() {
    let root = tempfile::tempdir().expect("template root should be created");

    for profile in [
        "",
        "../agent",
        "agent/profile",
        "agent.profile",
        "agent profile",
    ] {
        let error = Renderer::from_template_root(root.path(), profile, TEST_TEMPLATES)
            .err()
            .expect("unsafe profile name should fail before reading templates");
        assert!(
            matches!(error, RenderError::InvalidProfile(ref value) if value == profile),
            "profile {profile:?} returned {error}"
        );
    }
}
