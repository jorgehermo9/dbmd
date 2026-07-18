#[path = "support/sqlite.rs"]
mod support;

use std::str::FromStr;

use dbmd_backends::{all_template_files, introspect, render_context, DatabaseContext, Source};
use dbmd_core::SourceId;
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

    assert_eq!(context.sources[0].backend, "sqlite");
    assert_eq!(
        context.sources[0].single_file_template,
        "backends/sqlite/single_file/source.md.j2"
    );
    assert_eq!(context.sources[0].tables.len(), 6);
    assert_eq!(context.sources[0].views.len(), 2);
    assert_eq!(context.sources[0].triggers.len(), 3);

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
    assert!(markdown.contains("### `main.documents_touch_updated_at`"));
    assert!(markdown.contains("**AFTER UPDATE OF title, body**"));
}
