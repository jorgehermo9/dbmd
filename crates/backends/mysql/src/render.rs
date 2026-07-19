use dbmd_core::SourceId;
use dbmd_relational::presentation::{
    self, ColumnView, ConstraintView, FactView, IndexView, NamespaceView, TableDetailsView,
    TableView, TriggerView, ViewPresentation,
};
use dbmd_render::{code_block, inline_code, object_file_name, text, RenderSource, TemplateFile};
use serde::Serialize;

use super::{Catalog, Column, Constraint, ConstraintKind, Index, Routine, Table, Trigger, View};

const SINGLE_FILE_TEMPLATE: &str = "backends/mysql/single_file/source.md.j2";
const DIRECTORY_TEMPLATE: &str = "backends/mysql/directory/source.md.j2";
pub(crate) const TEMPLATES: &[TemplateFile] = &[
    TemplateFile::new(
        "single_file/backends/mysql/source.md.j2",
        SINGLE_FILE_TEMPLATE,
        include_str!("templates/single_file/source.md.j2"),
    ),
    TemplateFile::new(
        "directory/backends/mysql/source.md.j2",
        DIRECTORY_TEMPLATE,
        include_str!("templates/directory/source.md.j2"),
    ),
];

#[derive(Serialize)]
struct RoutineView {
    qualified_name: String,
    file_name: String,
    comment: Option<String>,
    facts: Vec<FactView>,
    definition: Option<String>,
}
#[derive(Serialize)]
struct EventView {
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
    triggers: Vec<TriggerView>,
    routines: Vec<RoutineView>,
    events: Vec<EventView>,
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
            .schemas
            .iter()
            .map(|schema| {
                NamespaceView::new(
                    inline_code(&schema.name),
                    Some(format!(
                        "Default character set {}; collation {}.",
                        inline_code(&schema.default_character_set),
                        inline_code(&schema.default_collation)
                    )),
                )
            })
            .collect(),
        tables: catalog.tables.iter().map(render_table).collect(),
        views: catalog.views.iter().map(render_view).collect(),
        triggers: catalog.triggers.iter().map(render_trigger).collect(),
        routines: catalog.routines.iter().map(render_routine).collect(),
        events: catalog
            .events
            .iter()
            .map(|event| EventView {
                qualified_name: inline_code(&format!("{}.{}", event.schema, event.name)),
                file_name: object_file_name(&event.schema, &event.name),
                comment: event.comment.as_deref().map(text),
                facts: vec![
                    FactView::new("Type", inline_code(&event.event_type)),
                    FactView::new("Status", inline_code(&event.status)),
                    FactView::new("Time zone", inline_code(&event.time_zone)),
                    FactView::new("On completion", inline_code(&event.on_completion)),
                ],
                definition: Some(code_block("sql", &event.definition)),
            })
            .collect(),
    };
    let objects = data
        .tables
        .iter()
        .map(|value| {
            presentation::directory_object("tables", "table.md.j2", &value.file_name, value)
        })
        .chain(data.views.iter().map(|value| {
            presentation::directory_object("views", "view.md.j2", &value.file_name, value)
        }))
        .chain(data.triggers.iter().map(|value| {
            presentation::directory_object("triggers", "trigger.md.j2", &value.file_name, value)
        }))
        .chain(data.routines.iter().map(|value| {
            presentation::directory_object("routines", "function.md.j2", &value.file_name, value)
        }))
        .chain(data.events.iter().map(|value| {
            presentation::directory_object("events", "function.md.j2", &value.file_name, value)
        }))
        .collect();
    RenderSource::builder(
        id.as_str(),
        "mysql",
        (SINGLE_FILE_TEMPLATE, DIRECTORY_TEMPLATE),
        &data,
    )
    .display_name(display_name.map(inline_code))
    .nested(nested)
    .objects(objects)
    .build()
}

fn render_table(table: &Table) -> TableView {
    let mut facts = Vec::new();
    for (label, value) in [
        ("Engine", table.engine.as_deref()),
        ("Row format", table.row_format.as_deref()),
        ("Collation", table.collation.as_deref()),
        ("Create options", table.create_options.as_deref()),
    ] {
        if let Some(value) = value {
            facts.push(FactView::new(label, inline_code(value)));
        }
    }
    for partition in &table.partitions {
        facts.push(FactView::new(
            "Partition",
            format!(
                "{}: {} {}",
                inline_code(&partition.name),
                inline_code(partition.method.as_deref().unwrap_or("-")),
                inline_code(partition.description.as_deref().unwrap_or("-"))
            ),
        ));
    }
    TableView::builder()
        .qualified_name(inline_code(&table.qualified_name()))
        .file_name(object_file_name(&table.schema, &table.name))
        .comment(table.comment.as_deref().map(text))
        .columns(table.columns.iter().map(render_column).collect())
        .constraints(table.constraints.iter().map(render_constraint).collect())
        .indexes(table.indexes.iter().map(render_index).collect())
        .backend(
            TableDetailsView::builder()
                .title("MySQL")
                .facts(facts)
                .notices(Vec::new())
                .definition(Some(code_block("sql", &table.definition)))
                .build(),
        )
        .build()
}

fn render_column(column: &Column) -> ColumnView {
    let mut notes = Vec::new();
    if let Some(comment) = &column.comment {
        notes.push(text(comment));
    }
    if !column.extra.is_empty() {
        notes.push(inline_code(&column.extra));
    }
    if let Some(expression) = &column.generation_expression {
        notes.push(format!("generated as {}", inline_code(expression)));
    }
    if column.visible == Some(false) {
        notes.push("invisible".to_string());
    }
    ColumnView::builder()
        .name(inline_code(&column.name))
        .data_type(inline_code(&column.column_type))
        .nullable(presentation::nullable(Some(column.nullable)))
        .default_value(
            column
                .default
                .as_deref()
                .map_or_else(|| "-".to_string(), inline_code),
        )
        .notes(notes.join("; "))
        .build()
}

fn render_constraint(value: &Constraint) -> ConstraintView {
    let kind = match value.kind {
        ConstraintKind::PrimaryKey => "primary_key",
        ConstraintKind::Unique => "unique",
        ConstraintKind::ForeignKey => "foreign_key",
        ConstraintKind::Check => "check",
        ConstraintKind::Unknown => "unknown",
    };
    let mut details = value
        .expression
        .as_deref()
        .map_or_else(|| "-".to_string(), inline_code);
    if let Some(table) = &value.referenced_table {
        details = format!(
            "references {}.{} ({})",
            inline_code(value.referenced_schema.as_deref().unwrap_or("-")),
            inline_code(table),
            inline_code(&value.referenced_columns.join(", "))
        );
    }
    if value.enforced == Some(false) {
        details.push_str("; not enforced");
    }
    ConstraintView::builder()
        .name(inline_code(&value.name))
        .kind(inline_code(kind))
        .columns(inline_code(&value.columns.join(", ")))
        .details(details)
        .build()
}

fn render_index(value: &Index) -> IndexView {
    let terms = value
        .terms
        .iter()
        .map(|term| {
            let mut value = term
                .column
                .as_deref()
                .or(term.expression.as_deref())
                .unwrap_or("-")
                .to_string();
            if let Some(prefix) = term.prefix_length {
                value.push_str(&format!("({prefix})"));
            }
            if term.descending == Some(true) {
                value.push_str(" DESC");
            }
            value
        })
        .collect::<Vec<_>>()
        .join(", ");
    let mut origin = value.index_type.clone();
    if value.visible == Some(false) {
        origin.push_str("; invisible");
    }
    IndexView::builder()
        .name(inline_code(&value.name))
        .terms(inline_code(&terms))
        .unique(if value.unique { "yes" } else { "no" })
        .origin(origin)
        .predicate("-".to_string())
        .build()
}

fn render_view(view: &View) -> ViewPresentation {
    ViewPresentation::builder()
        .qualified_name(inline_code(&view.qualified_name()))
        .file_name(object_file_name(&view.schema, &view.name))
        .comment(None)
        .facts(vec![
            FactView::new("Check option", inline_code(&view.check_option)),
            FactView::new("Updatable", if view.updatable { "yes" } else { "no" }),
            FactView::new("Security", inline_code(&view.security_type)),
            FactView::new("Definer", inline_code(&view.definer)),
        ])
        .definition(code_block("sql", &view.create_statement))
        .build()
}

fn render_trigger(trigger: &Trigger) -> TriggerView {
    TriggerView::builder()
        .qualified_name(inline_code(&format!("{}.{}", trigger.schema, trigger.name)))
        .file_name(object_file_name(&trigger.schema, &trigger.name))
        .target(inline_code(&format!(
            "{}.{}",
            trigger.schema, trigger.table
        )))
        .event(format!(
            "{} {}",
            inline_code(&trigger.timing),
            inline_code(&trigger.event)
        ))
        .with_facts(vec![
            FactView::new("Orientation", inline_code(&trigger.orientation)),
            FactView::new("Order", trigger.action_order.to_string()),
            FactView::new("Definer", inline_code(&trigger.definer)),
        ])
        .definition(code_block("sql", &trigger.statement))
        .build()
}

fn render_routine(routine: &Routine) -> RoutineView {
    RoutineView {
        qualified_name: inline_code(&format!("{}.{}", routine.schema, routine.name)),
        file_name: object_file_name(&routine.schema, &routine.name),
        comment: routine.comment.as_deref().map(text),
        facts: vec![
            FactView::new("Kind", inline_code(&routine.kind)),
            FactView::new("Data access", inline_code(&routine.sql_data_access)),
            FactView::new(
                "Deterministic",
                if routine.deterministic { "yes" } else { "no" },
            ),
            FactView::new("Security", inline_code(&routine.security_type)),
            FactView::new(
                "Parameters",
                inline_code(
                    &routine
                        .parameters
                        .iter()
                        .map(|parameter| {
                            format!(
                                "{} {}",
                                parameter.name.as_deref().unwrap_or("return"),
                                parameter.dtd_identifier
                            )
                        })
                        .collect::<Vec<_>>()
                        .join(", "),
                ),
            ),
        ],
        definition: routine
            .definition
            .as_deref()
            .map(|value| code_block("sql", value)),
    }
}
