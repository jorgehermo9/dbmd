use dbmd_core::SourceId;
use dbmd_relational::presentation::{
    self, ColumnView, ConstraintView, FactView, IndexView, NamespaceView, TableDetailsView,
    TableView, ViewPresentation,
};
use dbmd_render::{code_block, inline_code, object_file_name, text, RenderSource, TemplateFile};
use serde::Serialize;

use super::{Catalog, Column, Constraint, Function, Index, Table, View};

const SINGLE_FILE_TEMPLATE: &str = "backends/duckdb/single_file/source.md.j2";
const DIRECTORY_TEMPLATE: &str = "backends/duckdb/directory/source.md.j2";
pub(crate) const TEMPLATES: &[TemplateFile] = &[
    TemplateFile::new(
        "single_file/backends/duckdb/source.md.j2",
        SINGLE_FILE_TEMPLATE,
        include_str!("templates/single_file/source.md.j2"),
    ),
    TemplateFile::new(
        "directory/backends/duckdb/source.md.j2",
        DIRECTORY_TEMPLATE,
        include_str!("templates/directory/source.md.j2"),
    ),
];

#[derive(Serialize)]
struct ObjectView {
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
    objects: Vec<ObjectView>,
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
    let mut objects = catalog
        .types
        .iter()
        .map(|value| ObjectView {
            qualified_name: inline_code(&format!(
                "{}.{}.{}",
                value.database, value.schema, value.name
            )),
            file_name: object_file_name(
                &format!("{}.{}", value.database, value.schema),
                &value.name,
            ),
            comment: value.comment.as_deref().map(text),
            facts: std::iter::once(FactView::new("Kind", inline_code("type")))
                .chain(
                    value
                        .category
                        .as_deref()
                        .map(|category| FactView::new("Category", inline_code(category))),
                )
                .chain(std::iter::once(FactView::new(
                    "Logical type",
                    inline_code(&value.logical_type),
                )))
                .chain(
                    (!value.labels.is_empty())
                        .then(|| FactView::new("Labels", inline_code(&value.labels.join(", ")))),
                )
                .collect(),
            definition: None,
        })
        .collect::<Vec<_>>();
    objects.extend(catalog.sequences.iter().map(|value| {
        ObjectView {
            qualified_name: inline_code(&format!(
                "{}.{}.{}",
                value.database, value.schema, value.name
            )),
            file_name: object_file_name(
                &format!("{}.{}", value.database, value.schema),
                &value.name,
            ),
            comment: value.comment.as_deref().map(text),
            facts: vec![
                FactView::new("Kind", inline_code("sequence")),
                FactView::new("Start", value.start.to_string()),
                FactView::new("Increment", value.increment.to_string()),
                FactView::new("Cycle", if value.cycle { "yes" } else { "no" }),
            ],
            definition: value
                .definition
                .as_deref()
                .map(|sql| code_block("sql", sql)),
        }
    }));
    objects.extend(catalog.functions.iter().map(render_function));
    objects.extend(catalog.extensions.iter().map(|value| ObjectView {
        qualified_name: inline_code(&value.name),
        file_name: object_file_name("extensions", &value.name),
        comment: value.description.as_deref().map(text),
        facts: vec![
            FactView::new("Kind", inline_code("extension")),
            FactView::new("Loaded", if value.loaded { "yes" } else { "no" }),
            FactView::new("Installed", if value.installed { "yes" } else { "no" }),
            FactView::new(
                "Version",
                inline_code(value.version.as_deref().unwrap_or("-")),
            ),
        ],
        definition: None,
    }));
    let data = SourceData {
        section_heading,
        object_heading,
        detail_heading,
        namespaces: catalog
            .schemas
            .iter()
            .map(|value| {
                let database = catalog
                    .databases
                    .iter()
                    .find(|database| database.name == value.database);
                let database_details = database.map(|database| {
                    format!(
                        "{} catalog; {}",
                        inline_code(&database.database_type),
                        if database.readonly {
                            "read-only"
                        } else {
                            "read-write"
                        }
                    )
                });
                NamespaceView::new(
                    inline_code(&format!("{}.{}", value.database, value.name)),
                    match (database_details, value.comment.as_deref()) {
                        (Some(details), Some(comment)) => {
                            Some(format!("{details}; {}", text(comment)))
                        }
                        (Some(details), None) => Some(details),
                        (None, Some(comment)) => Some(text(comment)),
                        (None, None) => None,
                    },
                )
            })
            .collect(),
        tables: catalog.tables.iter().map(render_table).collect(),
        views: catalog.views.iter().map(render_view).collect(),
        objects,
    };
    let directory_objects = data
        .tables
        .iter()
        .map(|value| {
            presentation::directory_object("tables", "table.md.j2", &value.file_name, value)
        })
        .chain(data.views.iter().map(|value| {
            presentation::directory_object("views", "view.md.j2", &value.file_name, value)
        }))
        .chain(data.objects.iter().map(|value| {
            presentation::directory_object("objects", "function.md.j2", &value.file_name, value)
        }))
        .collect();
    RenderSource::builder(
        id.as_str(),
        "duckdb",
        (SINGLE_FILE_TEMPLATE, DIRECTORY_TEMPLATE),
        &data,
    )
    .display_name(display_name.map(inline_code))
    .nested(nested)
    .objects(directory_objects)
    .build()
}

fn render_table(table: &Table) -> TableView {
    TableView::builder()
        .qualified_name(inline_code(&table.qualified_name()))
        .file_name(object_file_name(
            &format!("{}.{}", table.database, table.schema),
            &table.name,
        ))
        .comment(table.comment.as_deref().map(text))
        .columns(table.columns.iter().map(render_column).collect())
        .constraints(table.constraints.iter().map(render_constraint).collect())
        .indexes(table.indexes.iter().map(render_index).collect())
        .backend(
            TableDetailsView::builder()
                .title("DuckDB")
                .facts(vec![FactView::new(
                    "Temporary",
                    if table.temporary { "yes" } else { "no" },
                )])
                .notices(Vec::new())
                .definition(Some(code_block("sql", &table.definition)))
                .build(),
        )
        .build()
}
fn render_column(value: &Column) -> ColumnView {
    let mut notes = value
        .comment
        .as_deref()
        .map(text)
        .into_iter()
        .collect::<Vec<_>>();
    if let Some(expression) = &value.generated_expression {
        notes.push(format!("generated as {}", inline_code(expression)));
    }
    ColumnView::builder()
        .name(inline_code(&value.name))
        .data_type(inline_code(&value.data_type))
        .nullable(presentation::nullable(Some(value.nullable)))
        .default_value(
            value
                .default
                .as_deref()
                .map_or_else(|| "-".to_string(), inline_code),
        )
        .notes(notes.join("; "))
        .build()
}
fn render_constraint(value: &Constraint) -> ConstraintView {
    ConstraintView::builder()
        .name("-".to_string())
        .kind(inline_code(&value.kind))
        .columns(inline_code(&value.columns.join(", ")))
        .details(inline_code(&value.text))
        .build()
}
fn render_index(value: &Index) -> IndexView {
    let mut origin = "duckdb".to_string();
    if value.primary {
        origin.push_str("; primary");
    }
    IndexView::builder()
        .name(inline_code(&value.name))
        .terms(inline_code(&value.expressions))
        .unique(if value.unique { "yes" } else { "no" })
        .origin(origin)
        .predicate("-".to_string())
        .build()
}
fn render_view(view: &View) -> ViewPresentation {
    ViewPresentation::builder()
        .qualified_name(inline_code(&view.qualified_name()))
        .file_name(object_file_name(
            &format!("{}.{}", view.database, view.schema),
            &view.name,
        ))
        .comment(view.comment.as_deref().map(text))
        .facts(vec![FactView::new(
            "Temporary",
            if view.temporary { "yes" } else { "no" },
        )])
        .definition(code_block("sql", &view.definition))
        .build()
}
fn render_function(value: &Function) -> ObjectView {
    ObjectView {
        qualified_name: inline_code(&format!(
            "{}.{}.{}",
            value.database, value.schema, value.name
        )),
        file_name: object_file_name(&format!("{}.{}", value.database, value.schema), &value.name),
        comment: value
            .comment
            .as_deref()
            .or(value.description.as_deref())
            .map(text),
        facts: vec![
            FactView::new("Kind", inline_code(&value.kind)),
            FactView::new(
                "Return type",
                inline_code(value.return_type.as_deref().unwrap_or("-")),
            ),
            FactView::new(
                "Parameters",
                inline_code(
                    &value
                        .parameters
                        .iter()
                        .enumerate()
                        .map(|(index, name)| {
                            value
                                .parameter_types
                                .get(index)
                                .map_or_else(|| name.clone(), |kind| format!("{name} {kind}"))
                        })
                        .collect::<Vec<_>>()
                        .join(", "),
                ),
            ),
            FactView::new(
                "Side effects",
                match value.side_effects {
                    Some(true) => "yes",
                    Some(false) => "no",
                    None => "unknown",
                },
            ),
        ],
        definition: value
            .definition
            .as_deref()
            .map(|definition| code_block("sql", definition)),
    }
}
