//! ClickHouse presentation mapping.

use dbmd_core::SourceId;
use dbmd_relational::presentation::{
    self, ColumnView, ConstraintView, FactView, IndexView, NamespaceView, TableDetailsView,
    TableView, ViewPresentation,
};
use dbmd_render::{code_block, inline_code, object_file_name, text, RenderSource, TemplateFile};
use serde::Serialize;

use super::{Catalog, Column, Table, TableKind};

pub(super) const SINGLE_FILE_TEMPLATE: &str = "backends/clickhouse/single_file/source.md.j2";
pub(super) const DIRECTORY_TEMPLATE: &str = "backends/clickhouse/directory/source.md.j2";
pub(crate) const TEMPLATES: &[TemplateFile] = &[
    TemplateFile::new(
        "single_file/backends/clickhouse/source.md.j2",
        SINGLE_FILE_TEMPLATE,
        include_str!("templates/single_file/source.md.j2"),
    ),
    TemplateFile::new(
        "directory/backends/clickhouse/source.md.j2",
        DIRECTORY_TEMPLATE,
        include_str!("templates/directory/source.md.j2"),
    ),
];

#[derive(Serialize)]
struct FunctionView {
    qualified_name: String,
    file_name: String,
    comment: Option<String>,
    facts: Vec<FactView>,
    definition: Option<String>,
}

#[derive(Serialize)]
struct SourceData {
    section_heading: &'static str,
    object_heading: &'static str,
    detail_heading: &'static str,
    namespaces: Vec<NamespaceView>,
    tables: Vec<TableView>,
    views: Vec<ViewPresentation>,
    functions: Vec<FunctionView>,
}

pub(crate) fn source(
    id: &SourceId,
    display_name: Option<&str>,
    catalog: &Catalog,
    nested: bool,
) -> RenderSource {
    let (section_heading, object_heading, detail_heading) = if nested {
        ("###", "####", "#####")
    } else {
        ("##", "###", "####")
    };
    let data = SourceData {
        section_heading,
        object_heading,
        detail_heading,
        namespaces: catalog
            .databases
            .iter()
            .map(|database| {
                NamespaceView::new(
                    inline_code(&database.name),
                    Some(match &database.comment {
                        Some(comment) => format!(
                            "Engine {}; {}",
                            inline_code(&database.engine),
                            text(comment)
                        ),
                        None => format!("Engine {}", inline_code(&database.engine)),
                    }),
                )
            })
            .collect(),
        tables: catalog
            .tables
            .iter()
            .filter(|table| matches!(table.kind, TableKind::Table | TableKind::Dictionary))
            .map(render_table)
            .collect(),
        views: catalog
            .tables
            .iter()
            .filter(|table| !matches!(table.kind, TableKind::Table | TableKind::Dictionary))
            .map(render_view)
            .collect(),
        functions: catalog
            .functions
            .iter()
            .map(|function| FunctionView {
                qualified_name: inline_code(&function.name),
                file_name: object_file_name("global", &function.name),
                comment: None,
                facts: vec![FactView::new("Kind", inline_code("sql_user_defined"))],
                definition: Some(code_block("sql", &function.definition)),
            })
            .collect(),
    };
    let objects = data
        .tables
        .iter()
        .map(|object| {
            presentation::directory_object("tables", "table.md.j2", &object.file_name, object)
        })
        .chain(data.views.iter().map(|object| {
            presentation::directory_object("views", "view.md.j2", &object.file_name, object)
        }))
        .chain(data.functions.iter().map(|object| {
            presentation::directory_object("functions", "function.md.j2", &object.file_name, object)
        }))
        .collect();
    RenderSource::builder(
        id.as_str(),
        "clickhouse",
        (SINGLE_FILE_TEMPLATE, DIRECTORY_TEMPLATE),
        &data,
    )
    .display_name(display_name.map(inline_code))
    .nested(nested)
    .objects(objects)
    .build()
}

fn render_table(table: &Table) -> TableView {
    let mut facts = table_facts(table);
    for projection in &table.projections {
        facts.push(FactView::new(
            "Projection",
            format!(
                "{} {} sorted by {}: {}",
                inline_code(&projection.name),
                inline_code(&projection.projection_type),
                inline_code(&projection.sorting_key),
                inline_code(&projection.query)
            ),
        ));
    }
    TableView::builder()
        .qualified_name(inline_code(&table.qualified_name()))
        .file_name(object_file_name(&table.database, &table.name))
        .comment(table.comment.as_deref().map(text))
        .columns(table.columns.iter().map(render_column).collect())
        .constraints(
            table
                .constraints
                .iter()
                .map(|constraint| {
                    ConstraintView::builder()
                        .name(inline_code(&constraint.name))
                        .kind(inline_code(&constraint.constraint_type))
                        .columns("-".to_string())
                        .details(inline_code(&constraint.expression))
                        .build()
                })
                .collect(),
        )
        .indexes(
            table
                .data_skipping_indexes
                .iter()
                .map(|index| {
                    let mut origin = format!(
                        "clickhouse {} granularity {}",
                        inline_code(&index.type_full),
                        index.granularity
                    );
                    if index.implicit == Some(true) {
                        origin.push_str("; implicit");
                    } else if index.implicit.is_none() {
                        origin.push_str("; creation origin unknown");
                    }
                    IndexView::builder()
                        .name(inline_code(&index.name))
                        .terms(inline_code(&index.expression))
                        .unique("no")
                        .origin(origin)
                        .predicate("-".to_string())
                        .build()
                })
                .collect(),
        )
        .backend(
            TableDetailsView::builder()
                .title("ClickHouse")
                .facts(facts)
                .notices(Vec::new())
                .definition(Some(code_block("sql", &table.definition)))
                .build(),
        )
        .build()
}

fn table_facts(table: &Table) -> Vec<FactView> {
    let mut facts = vec![FactView::new("Kind", inline_code(kind_name(table.kind)))];
    if !table.engine_full.is_empty() {
        facts.push(FactView::new("Engine", inline_code(&table.engine_full)));
    }
    for (label, value) in [
        ("Partition key", &table.partition_key),
        ("Primary key", &table.primary_key),
        ("Sorting key", &table.sorting_key),
        ("Sampling key", &table.sampling_key),
        ("Storage policy", &table.storage_policy),
    ] {
        if !value.is_empty() {
            facts.push(FactView::new(label, inline_code(value)));
        }
    }
    if let Some(target) = &table.target {
        facts.push(FactView::new("Target", inline_code(target)));
    }
    facts
}

fn render_column(column: &Column) -> ColumnView {
    let mut notes = Vec::new();
    if let Some(comment) = &column.comment {
        notes.push(text(comment));
    }
    if let Some(codec) = &column.compression_codec {
        notes.push(format!("codec {}", inline_code(codec)));
    }
    let mut key_roles = Vec::new();
    if column.in_partition_key {
        key_roles.push("partition");
    }
    if column.in_primary_key {
        key_roles.push("primary");
    }
    if column.in_sorting_key {
        key_roles.push("sorting");
    }
    if column.in_sampling_key {
        key_roles.push("sampling");
    }
    if !key_roles.is_empty() {
        notes.push(format!("keys {}", inline_code(&key_roles.join(", "))));
    }
    if column.default_kind.as_str() != "none" {
        notes.push(format!(
            "{} expression",
            inline_code(column.default_kind.as_str())
        ));
    }
    ColumnView::builder()
        .name(inline_code(&column.name))
        .data_type(inline_code(&column.data_type))
        .nullable(if column.data_type.starts_with("Nullable(") {
            "yes"
        } else {
            "no"
        })
        .default_value(
            column
                .default_expression
                .as_deref()
                .map_or_else(|| "-".to_string(), inline_code),
        )
        .notes(notes.join("; "))
        .build()
}

fn render_view(table: &Table) -> ViewPresentation {
    let mut facts = table_facts(table);
    if !table.projections.is_empty() {
        facts.push(FactView::new(
            "Projections",
            table.projections.len().to_string(),
        ));
    }
    ViewPresentation::builder()
        .qualified_name(inline_code(&table.qualified_name()))
        .file_name(object_file_name(&table.database, &table.name))
        .comment(table.comment.as_deref().map(text))
        .facts(facts)
        .columns(table.columns.iter().map(render_column).collect())
        .definition(code_block("sql", &table.definition))
        .build()
}

const fn kind_name(kind: TableKind) -> &'static str {
    match kind {
        TableKind::Table => "table",
        TableKind::View => "view",
        TableKind::MaterializedView => "materialized_view",
        TableKind::LiveView => "live_view",
        TableKind::WindowView => "window_view",
        TableKind::Dictionary => "dictionary",
    }
}
