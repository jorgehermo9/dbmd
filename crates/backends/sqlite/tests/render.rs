#[path = "support/sqlite.rs"]
mod support;

use std::str::FromStr;

use dbmd_backend_sqlite::{introspect, render_source, template_files, SqliteSource};
use dbmd_core::SourceId;
use dbmd_render::{RenderContext, RenderedArtifact, Renderer};
use support::TestDatabase;

#[test]
fn maps_the_sqlite_catalog_and_templates_into_a_renderable_source() {
    let database = TestDatabase::from_sql(include_str!("fixtures/schema_objects/schema.sql"));
    let source = SqliteSource::new(
        SourceId::from_str("app").expect("test source ID should be valid"),
        database.path(),
    );
    let snapshot = introspect(&source).expect("SQLite source should introspect");
    let source = render_source(
        snapshot.id(),
        snapshot.display_name(),
        snapshot.catalog(),
        false,
    );
    let context = RenderContext::new(vec![source]);

    let source = &context.sources()[0];
    assert_eq!(source.backend(), "sqlite");
    assert_eq!(
        source.single_file_template(),
        "backends/sqlite/single_file/source.md.j2"
    );
    assert_eq!(source.objects().len(), 11);
    assert_eq!(
        source
            .objects()
            .iter()
            .map(|object| (object.relative_path(), object.template()))
            .collect::<Vec<_>>(),
        [
            ("tables/main.document_search.md", "table.md.j2"),
            ("tables/main.document_search_config.md", "table.md.j2"),
            ("tables/main.document_search_data.md", "table.md.j2"),
            ("tables/main.document_search_docsize.md", "table.md.j2"),
            ("tables/main.document_search_idx.md", "table.md.j2"),
            ("tables/main.documents.md", "table.md.j2"),
            ("views/main.document_summaries.md", "view.md.j2"),
            ("views/main.document_titles.md", "view.md.j2"),
            (
                "triggers/main.document_summaries%2Edocument_summaries_insert.md",
                "trigger.md.j2"
            ),
            (
                "triggers/main.documents%2Edocuments_prevent_root_delete.md",
                "trigger.md.j2"
            ),
            (
                "triggers/main.documents%2Edocuments_touch_updated_at.md",
                "trigger.md.j2"
            ),
        ]
    );
    insta::assert_yaml_snapshot!("sqlite_render_context", context);

    let artifact = Renderer::embedded(template_files())
        .expect("SQLite templates should compile")
        .render(&context)
        .expect("backend-prepared source should render");
    let RenderedArtifact::SingleFile(markdown) = artifact else {
        panic!("default rendering should produce a single file");
    };
    let markdown = String::from_utf8(markdown).expect("Markdown should be UTF-8");

    assert!(markdown.contains("### `main.documents`"));
    assert!(markdown.contains("### `main.documents.documents_touch_updated_at`"));
    assert!(markdown.contains("**AFTER UPDATE OF title, body**"));
}
