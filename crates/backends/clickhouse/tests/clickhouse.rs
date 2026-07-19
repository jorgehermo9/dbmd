use std::str::FromStr;

use dbmd_backend_clickhouse::{
    introspect, render_source, template_files, ClickHouseSource, TableKind,
};
use dbmd_core::SourceId;
use dbmd_render::{OutputLayout, RenderContext, RenderOptions, RenderedArtifact, Renderer};
use dbmd_test_support::ClickHouseServer;

#[test]
fn introspects_the_clickhouse_schema_surface_deterministically() {
    let server = ClickHouseServer::start(include_str!("fixtures/schema_surface.sql"))
        .expect("ClickHouse test container should start");
    let source = ClickHouseSource::new(
        SourceId::from_str("analytics").expect("test source ID should be valid"),
        server.endpoint(),
    )
    .with_database("analytics");

    let first = introspect(&source).expect("ClickHouse introspection should succeed");
    let second = introspect(&source).expect("repeated introspection should succeed");

    assert_eq!(first, second);
    assert_eq!(first.catalog().databases.len(), 1);
    assert_eq!(first.catalog().tables.len(), 6);
    assert!(first
        .catalog()
        .tables
        .iter()
        .any(|table| table.name == "country_names" && table.kind == TableKind::Dictionary));
    let events = first
        .catalog()
        .tables
        .iter()
        .find(|table| table.name == "events")
        .expect("events table should be present");
    assert_eq!(events.kind, TableKind::Table);
    assert_eq!(events.engine, "ReplacingMergeTree");
    assert_eq!(events.partition_key, "toYYYYMM(occurred_at)");
    assert_eq!(events.primary_key, "tenant_id, event_id");
    assert_eq!(events.sorting_key, "tenant_id, event_id, occurred_at");
    assert_eq!(events.data_skipping_indexes.len(), 1);
    assert_eq!(events.projections.len(), 1);
    assert_eq!(events.constraints.len(), 1);
    assert!(events.columns.iter().any(
        |column| column.name == "expires_at" && column.default_kind.as_str() == "materialized"
    ));
    let materialized_view = first
        .catalog()
        .tables
        .iter()
        .find(|table| table.name == "event_counts_mv")
        .expect("materialized view should be present");
    assert_eq!(materialized_view.kind, TableKind::MaterializedView);
    assert_eq!(
        materialized_view.target.as_deref(),
        Some("analytics.event_counts")
    );
    assert_eq!(first.catalog().functions.len(), 1);
    insta::assert_yaml_snapshot!("clickhouse_schema_surface", first);

    let context = RenderContext::new(vec![render_source(
        first.id(),
        first.display_name(),
        first.catalog(),
        false,
    )]);
    let renderer =
        Renderer::embedded(template_files()).expect("ClickHouse embedded templates should compile");
    let RenderedArtifact::SingleFile(markdown) = renderer
        .render(&context)
        .expect("ClickHouse presentation should render")
    else {
        panic!("default ClickHouse rendering should produce one file");
    };
    let markdown = String::from_utf8(markdown).expect("rendered Markdown should be UTF-8");
    assert!(markdown.contains("ReplacingMergeTree"));
    assert!(markdown.contains("payload_tokens"));
    assert!(markdown.contains("analytics_normalize"));
    insta::assert_snapshot!("clickhouse_markdown", markdown);
    assert_directory_render(&renderer, &context, "tables/analytics.events.md");
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
        .expect("ClickHouse directory profile should render")
    else {
        panic!("directory options should produce a directory artifact");
    };
    let index_path = "index.md"
        .parse()
        .expect("static artifact path should parse");
    let index = String::from_utf8(files[&index_path].clone()).expect("index should be UTF-8");
    assert!(index.contains("tables/analytics.events.md"));
    assert!(files.keys().any(|path| path.as_str() == object_path));
}
