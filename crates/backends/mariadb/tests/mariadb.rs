use std::str::FromStr;

use dbmd_backend_mariadb::{
    introspect, render_source, template_files, ConstraintKind, MariaDbSource,
};
use dbmd_core::SourceId;
use dbmd_render::{OutputLayout, RenderContext, RenderOptions, RenderedArtifact, Renderer};
use dbmd_test_support::MariaDbServer;

#[test]
fn introspects_and_renders_the_mariadb_schema_surface_deterministically() {
    let server = MariaDbServer::start(include_str!("fixtures/schema_surface.sql"))
        .expect("MariaDB test container should start");
    let source = MariaDbSource::new(
        SourceId::from_str("commerce").expect("test source ID should be valid"),
        server.url(),
    )
    .with_schema("test");

    let first = introspect(&source).expect("MariaDB introspection should succeed");
    let second = introspect(&source).expect("repeat introspection should succeed");
    assert_eq!(first, second);
    assert_eq!(first.catalog().tables.len(), 3);
    assert_eq!(first.catalog().sequences.len(), 1);
    let accounts = first
        .catalog()
        .tables
        .iter()
        .find(|table| table.name == "accounts")
        .expect("accounts table should be present");
    assert!(accounts.system_versioned);
    assert_eq!(
        accounts
            .system_time_period
            .as_ref()
            .map(|period| period.start_column.as_str()),
        Some("row_start")
    );
    assert!(accounts
        .columns
        .iter()
        .any(|column| column.name == "normalized_email" && column.generation_expression.is_some()));
    assert!(accounts
        .constraints
        .iter()
        .any(|constraint| constraint.kind == ConstraintKind::ForeignKey));
    assert!(accounts
        .indexes
        .iter()
        .any(|index| index.name == "accounts_email_ignored_idx" && index.ignored == Some(true)));
    assert_eq!(first.catalog().views.len(), 1);
    assert_eq!(first.catalog().routines.len(), 2);
    assert_eq!(first.catalog().triggers.len(), 1);
    assert_eq!(first.catalog().events.len(), 1);
    insta::assert_yaml_snapshot!("mariadb_schema_surface", first);

    let context = RenderContext::new(vec![render_source(
        first.id(),
        first.display_name(),
        first.catalog(),
        false,
    )]);
    let renderer = Renderer::embedded(template_files()).expect("MariaDB templates should compile");
    let RenderedArtifact::SingleFile(markdown) = renderer
        .render(&context)
        .expect("MariaDB catalog should render")
    else {
        panic!("default MariaDB rendering should produce one file");
    };
    let markdown = String::from_utf8(markdown).expect("Markdown should be UTF-8");
    assert!(markdown.contains("System versioning"));
    assert!(markdown.contains("order_number_seq"));
    insta::assert_snapshot!("mariadb_markdown", markdown);
    assert_directory_render(&renderer, &context, "objects/test.order_number_seq.md");
}

fn assert_directory_render(renderer: &Renderer, context: &RenderContext, object_path: &str) {
    let RenderedArtifact::Directory(files) = renderer
        .render_with_options(
            context,
            RenderOptions {
                layout: OutputLayout::Directory,
                ..RenderOptions::default()
            },
        )
        .expect("MariaDB directory profile should render")
    else {
        panic!("directory options should produce a directory artifact");
    };
    let index_path = "index.md"
        .parse()
        .expect("static artifact path should parse");
    let index = String::from_utf8(files[&index_path].clone()).expect("index should be UTF-8");
    assert!(index.contains("objects/test.order_number_seq.md"));
    assert!(files.keys().any(|path| path.as_str() == object_path));
}
