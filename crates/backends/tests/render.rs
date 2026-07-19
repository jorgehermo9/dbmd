#[path = "support/sqlite.rs"]
mod support;

use std::str::FromStr;

use dbmd_backends::{
    all_template_files, introspect, render_context, Catalog, DatabaseContext, Snapshot, Source,
};
use dbmd_core::{SourceId, SourceSnapshot};
use dbmd_render::{RenderedArtifact, Renderer};
use support::TestDatabase;

#[test]
fn sqlite_module_maps_its_catalog_and_templates_into_a_renderable_source() {
    let database =
        TestDatabase::from_sql(include_str!("fixtures/sqlite/schema_objects/schema.sql"));
    let source = Source::from(dbmd_backends::sqlite::SqliteSource::new(
        SourceId::from_str("app").expect("test source ID should be valid"),
        database.path(),
    ));
    let snapshot = introspect(&source).expect("SQLite source should introspect");
    let database = DatabaseContext::new(vec![snapshot]).expect("source should form a context");

    let context = render_context(&database, false);

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

    let templates = all_template_files();
    let artifact = Renderer::embedded(&templates)
        .expect("composed templates should compile")
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

#[test]
fn heterogeneous_sources_render_in_context_order() {
    let postgres = source_snapshot("warehouse", Catalog::Postgres(Default::default()))
        .with_display_name("Warehouse");
    let sqlite = source_snapshot("local", Catalog::Sqlite(Default::default()));
    let database =
        DatabaseContext::new(vec![postgres, sqlite]).expect("sources should form a context");

    let context = render_context(&database, true);
    assert_eq!(
        context
            .sources()
            .iter()
            .map(dbmd_render::RenderSource::backend)
            .collect::<Vec<_>>(),
        ["postgres", "sqlite"]
    );

    let artifact = Renderer::embedded(&all_template_files())
        .expect("composed templates should compile")
        .render(&context)
        .expect("heterogeneous context should render");
    let RenderedArtifact::SingleFile(markdown) = artifact else {
        panic!("default rendering should produce a single file");
    };
    let markdown = String::from_utf8(markdown).expect("Markdown should be UTF-8");
    let warehouse = markdown
        .find("## Source: `warehouse`")
        .expect("PostgreSQL source should render");
    let local = markdown
        .find("## Source: `local`")
        .expect("SQLite source should render");
    assert!(warehouse < local, "selected source order must be preserved");
}

fn source_snapshot(id: &str, catalog: Catalog) -> Snapshot {
    SourceSnapshot::new(
        SourceId::from_str(id).expect("test source ID should be valid"),
        catalog,
    )
}
