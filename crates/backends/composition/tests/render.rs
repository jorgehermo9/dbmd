use std::str::FromStr;

use dbmd_backends::{all_template_files, render_context, Catalog, DatabaseContext, Snapshot};
use dbmd_core::{SourceId, SourceSnapshot};
use dbmd_render::{RenderedArtifact, Renderer};

#[test]
fn heterogeneous_sources_render_in_context_order() {
    let duckdb = source_snapshot("warehouse", Catalog::Duckdb(Box::default()))
        .with_display_name("Warehouse");
    let sqlite = source_snapshot("local", Catalog::Sqlite(Box::default()));
    let database =
        DatabaseContext::new(vec![duckdb, sqlite]).expect("sources should form a context");

    let context = render_context(&database, true);
    assert_eq!(
        context
            .sources()
            .iter()
            .map(dbmd_render::RenderSource::backend)
            .collect::<Vec<_>>(),
        ["duckdb", "sqlite"]
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
        .expect("DuckDB source should render");
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
