use std::str::FromStr;

use dbmd_backend_mysql::{introspect, render_source, template_files, ConstraintKind, MysqlSource};
use dbmd_core::SourceId;
use dbmd_render::{OutputLayout, RenderContext, RenderOptions, RenderedArtifact, Renderer};
use dbmd_test_support::MysqlServer;

#[test]
fn introspects_and_renders_the_mysql_schema_surface_deterministically() {
    let server = MysqlServer::start(include_str!("fixtures/schema_surface.sql"))
        .expect("MySQL test container should start");
    let source = MysqlSource::new(
        SourceId::from_str("commerce").expect("test source ID should be valid"),
        server.url(),
    )
    .with_schema("test");

    let first = introspect(&source).expect("MySQL introspection should succeed");
    let second = introspect(&source).expect("repeat introspection should succeed");
    assert_eq!(first, second);
    assert_eq!(first.catalog().tables.len(), 3);
    let accounts = first
        .catalog()
        .tables
        .iter()
        .find(|table| table.name == "accounts")
        .expect("accounts table should be present");
    assert!(accounts
        .columns
        .iter()
        .any(|column| column.name == "normalized_email" && column.generation_expression.is_some()));
    assert!(accounts
        .columns
        .iter()
        .any(|column| column.name == "secret_token" && column.visible == Some(false)));
    assert!(accounts
        .constraints
        .iter()
        .any(|constraint| constraint.kind == ConstraintKind::ForeignKey));
    assert!(accounts.indexes.iter().any(
        |index| index.name == "accounts_normalized_idx" && index.terms[0].expression.is_some()
    ));
    assert_eq!(first.catalog().views.len(), 1);
    assert_eq!(first.catalog().routines.len(), 2);
    assert_eq!(first.catalog().triggers.len(), 1);
    assert_eq!(first.catalog().events.len(), 1);
    assert_eq!(
        first
            .catalog()
            .tables
            .iter()
            .find(|table| table.name == "monthly_metrics")
            .expect("partitioned table should be present")
            .partitions
            .len(),
        2
    );
    insta::assert_yaml_snapshot!("mysql_schema_surface", first);

    let context = RenderContext::new(vec![render_source(
        first.id(),
        first.display_name(),
        first.catalog(),
        false,
    )]);
    let renderer = Renderer::embedded(template_files()).expect("MySQL templates should compile");
    let RenderedArtifact::SingleFile(markdown) = renderer
        .render(&context)
        .expect("MySQL catalog should render")
    else {
        panic!("default MySQL rendering should produce one file");
    };
    let markdown = String::from_utf8(markdown).expect("Markdown should be UTF-8");
    assert!(markdown.contains("accounts_normalized_idx"));
    assert!(markdown.contains("purge_disabled_accounts"));
    insta::assert_snapshot!("mysql_markdown", markdown);
    assert_directory_render(&renderer, &context, "tables/test.accounts.md");
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
        .expect("MySQL directory profile should render")
    else {
        panic!("directory options should produce a directory artifact");
    };
    let index = String::from_utf8(files[&"index.md".parse().unwrap()].clone()).unwrap();
    assert!(index.contains("tables/test.accounts.md"));
    assert!(files.keys().any(|path| path.as_str() == object_path));
}
