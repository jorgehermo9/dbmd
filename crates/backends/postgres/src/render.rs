use std::fmt::Write as _;

use dbmd_render::{code_block, inline_code, object_file_name, text, RenderSource, TemplateFile};
use serde::Serialize;

use super::catalog::{
    Catalog, Column, Constraint, ConstraintKind, Function, FunctionParallel, FunctionVolatility,
    Index, IndexNullsOrder, IndexTarget, IndexTerm, PolicyCommand, Table, TableKind, Trigger,
    TriggerEnabled, TriggerEvent, TriggerOrientation, TriggerTiming, View,
};
use dbmd_core::SourceId;
use dbmd_relational::presentation::{
    self, ColumnView as RenderColumn, ConstraintView as RenderConstraint, FactView as RenderFact,
    IndexView as RenderIndex, NamespaceView, TableDetailsView as RenderTableDetails,
    TableView as RenderTable, TriggerView as RenderTrigger, ViewPresentation as RenderView,
};
use dbmd_relational::IndexSortOrder;

pub(super) const SINGLE_FILE_TEMPLATE: &str = "backends/postgres/single_file/source.md.j2";
pub(super) const DIRECTORY_TEMPLATE: &str = "backends/postgres/directory/source.md.j2";
pub(crate) const TEMPLATES: &[TemplateFile] = &[
    TemplateFile::new(
        "single_file/backends/postgres/source.md.j2",
        SINGLE_FILE_TEMPLATE,
        include_str!("templates/single_file/source.md.j2"),
    ),
    TemplateFile::new(
        "directory/backends/postgres/source.md.j2",
        DIRECTORY_TEMPLATE,
        include_str!("templates/directory/source.md.j2"),
    ),
];

#[derive(Serialize)]
struct RenderEnum {
    qualified_name: String,
    file_name: String,
    comment: Option<String>,
    values: String,
}

#[derive(Serialize)]
struct RenderFunction {
    qualified_name: String,
    file_name: String,
    comment: Option<String>,
    facts: Vec<RenderFact>,
    definition: Option<String>,
}

#[derive(Serialize)]
struct SourceData {
    section_heading: &'static str,
    object_heading: &'static str,
    detail_heading: &'static str,
    namespaces: Vec<NamespaceView>,
    enums: Vec<RenderEnum>,
    tables: Vec<RenderTable>,
    views: Vec<RenderView>,
    triggers: Vec<RenderTrigger>,
    functions: Vec<RenderFunction>,
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
        namespaces: presentation::namespaces(&catalog.namespaces),
        enums: catalog
            .enums
            .iter()
            .map(|enum_type| RenderEnum {
                qualified_name: inline_code(&format!("{}.{}", enum_type.namespace, enum_type.name)),
                file_name: object_file_name(&enum_type.namespace, &enum_type.name),
                comment: enum_type.comment.as_deref().map(text),
                values: inline_code(&enum_type.values.join(", ")),
            })
            .collect(),
        tables: catalog.tables.iter().map(render_table).collect(),
        views: catalog.views.iter().map(render_view).collect(),
        triggers: catalog.triggers.iter().map(render_trigger).collect(),
        functions: catalog.functions.iter().map(render_function).collect(),
    };
    let objects = data
        .enums
        .iter()
        .map(|object| {
            presentation::directory_object("enums", "enum.md.j2", &object.file_name, object)
        })
        .chain(data.tables.iter().map(|object| {
            presentation::directory_object("tables", "table.md.j2", &object.file_name, object)
        }))
        .chain(data.views.iter().map(|object| {
            presentation::directory_object("views", "view.md.j2", &object.file_name, object)
        }))
        .chain(data.triggers.iter().map(|object| {
            presentation::directory_object("triggers", "trigger.md.j2", &object.file_name, object)
        }))
        .chain(data.functions.iter().map(|object| {
            presentation::directory_object("functions", "function.md.j2", &object.file_name, object)
        }))
        .collect();
    RenderSource::builder(
        id.as_str(),
        "postgres",
        (SINGLE_FILE_TEMPLATE, DIRECTORY_TEMPLATE),
        &data,
    )
    .display_name(display_name.map(inline_code))
    .nested(nested)
    .objects(objects)
    .build()
}

fn render_table(table: &Table) -> RenderTable {
    let mut facts = vec![RenderFact::new(
        "Kind",
        inline_code(match table.kind {
            TableKind::Table => "table",
            TableKind::PartitionedTable => "partitioned_table",
            TableKind::Partition => "partition",
            TableKind::ForeignTable => "foreign_table",
        }),
    )];
    if let Some(tablespace) = &table.tablespace {
        facts.push(RenderFact::new("Tablespace", inline_code(tablespace)));
    }
    if !table.inherits.is_empty() {
        facts.push(RenderFact::new(
            "Inherits",
            inline_code(&table.inherits.join(", ")),
        ));
    }
    if let Some(value) = &table.partition_key {
        facts.push(RenderFact::new("Partition key", inline_code(value)));
    }
    if let Some(value) = &table.partition_parent {
        facts.push(RenderFact::new("Partition parent", inline_code(value)));
    }
    if let Some(value) = &table.partition_bound {
        facts.push(RenderFact::new("Partition bound", inline_code(value)));
    }
    for policy in &table.policies {
        let mut value = format!(
            "{} {} to {} ({})",
            inline_code(&policy.name),
            inline_code(match policy.command {
                PolicyCommand::All => "all",
                PolicyCommand::Select => "select",
                PolicyCommand::Insert => "insert",
                PolicyCommand::Update => "update",
                PolicyCommand::Delete => "delete",
            }),
            inline_code(&policy.roles.join(", ")),
            if policy.permissive {
                "permissive"
            } else {
                "restrictive"
            }
        );
        if let Some(expression) = &policy.using_expression {
            let _ = write!(value, "; using {}", inline_code(expression));
        }
        if let Some(expression) = &policy.check_expression {
            let _ = write!(value, "; check {}", inline_code(expression));
        }
        facts.push(RenderFact::new("Policy", value));
    }
    let mut notices = Vec::new();
    if table.row_level_security {
        notices.push("Row-level security enabled.");
    }
    if table.force_row_level_security {
        notices.push("Row-level security forced for the table owner.");
    }
    RenderTable::builder()
        .qualified_name(inline_code(&table.qualified_name()))
        .file_name(object_file_name(&table.namespace, &table.name))
        .comment(table.comment.as_deref().map(text))
        .columns(table.columns.iter().map(render_column).collect())
        .constraints(table.constraints.iter().map(render_constraint).collect())
        .indexes(table.indexes.iter().map(render_index).collect())
        .backend(
            RenderTableDetails::builder()
                .title("PostgreSQL")
                .facts(facts)
                .notices(notices)
                .definition(None)
                .build(),
        )
        .build()
}

fn render_column(column: &Column) -> RenderColumn {
    let mut notes = column
        .comment
        .as_deref()
        .map(text)
        .into_iter()
        .collect::<Vec<_>>();
    if let Some(identity) = &column.identity {
        notes.push(format!("identity {}", inline_code(identity)));
    }
    if let Some(generated) = &column.generated {
        notes.push(format!("generated as {}", inline_code(generated)));
    }
    if !column.enum_values.is_empty() {
        notes.push(format!(
            "enum values {}",
            inline_code(&column.enum_values.join(", "))
        ));
    }
    RenderColumn::builder()
        .name(inline_code(&column.name))
        .data_type(inline_code(&column.data_type))
        .nullable(presentation::nullable(column.nullable))
        .default_value(
            column
                .default
                .as_deref()
                .map_or_else(|| "-".to_string(), inline_code),
        )
        .notes(notes.join("; "))
        .build()
}

fn render_constraint(constraint: &Constraint) -> RenderConstraint {
    let mut details = inline_code(&constraint.definition);
    if !constraint.validated {
        details.push_str("; not validated");
    }
    if !constraint.locally_defined {
        details.push_str("; inherited");
    }
    if constraint.no_inherit {
        details.push_str("; no inherit");
    }
    RenderConstraint::builder()
        .name(
            constraint
                .name
                .as_deref()
                .map_or_else(|| "-".to_string(), inline_code),
        )
        .kind(inline_code(constraint_kind(constraint.kind)))
        .columns(inline_code(&constraint.columns.join(", ")))
        .details(details)
        .build()
}

fn render_index(index: &Index) -> RenderIndex {
    let mut origin = format!("postgres {}", inline_code(&index.method));
    if !index.included_columns.is_empty() {
        let _ = write!(
            origin,
            "; include {}",
            inline_code(&index.included_columns.join(", "))
        );
    }
    if index.nulls_not_distinct {
        origin.push_str("; nulls not distinct");
    }
    if !index.valid {
        origin.push_str("; invalid");
    }
    if !index.ready {
        origin.push_str("; not ready");
    }
    if index.clustered {
        origin.push_str("; clustered");
    }
    if index.replica_identity {
        origin.push_str("; replica identity");
    }
    RenderIndex::builder()
        .name(inline_code(&index.name))
        .terms(index_terms(&index.terms))
        .unique(if index.unique { "yes" } else { "no" })
        .origin(origin)
        .predicate(
            index
                .predicate
                .as_deref()
                .map_or_else(|| "-".to_string(), inline_code),
        )
        .build()
}

const fn constraint_kind(kind: ConstraintKind) -> &'static str {
    match kind {
        ConstraintKind::PrimaryKey => "primary_key",
        ConstraintKind::ForeignKey => "foreign_key",
        ConstraintKind::Unique => "unique",
        ConstraintKind::Check => "check",
        ConstraintKind::Exclusion => "exclusion",
    }
}

fn index_terms(terms: &[IndexTerm]) -> String {
    terms
        .iter()
        .map(|term| {
            let target = match &term.target {
                IndexTarget::Column(value) | IndexTarget::Expression(value) => inline_code(value),
            };
            let mut rendered = format!(
                "{target} {}",
                match term.order {
                    IndexSortOrder::Ascending => "ascending",
                    IndexSortOrder::Descending => "descending",
                }
            );
            if let Some(collation) = &term.collation {
                let _ = write!(rendered, " collate {}", inline_code(collation));
            }
            if let Some(operator_class) = &term.operator_class {
                let _ = write!(rendered, " opclass {}", inline_code(operator_class));
            }
            if let Some(nulls_order) = term.nulls_order {
                let _ = write!(
                    rendered,
                    " nulls {}",
                    inline_code(match nulls_order {
                        IndexNullsOrder::First => "first",
                        IndexNullsOrder::Last => "last",
                    })
                );
            }
            rendered
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn render_view(view: &View) -> RenderView {
    RenderView::builder()
        .qualified_name(inline_code(&format!("{}.{}", view.namespace, view.name)))
        .file_name(object_file_name(&view.namespace, &view.name))
        .comment(view.comment.as_deref().map(text))
        .facts(vec![RenderFact::new(
            "Kind",
            inline_code(if view.materialized {
                "materialized_view"
            } else {
                "view"
            }),
        )])
        .columns(view.columns.iter().map(render_column).collect())
        .definition(code_block("sql", &view.definition))
        .build()
}

fn render_trigger(trigger: &Trigger) -> RenderTrigger {
    let identity = format!("{}.{}.{}", trigger.namespace, trigger.target, trigger.name);
    let events = trigger
        .events
        .iter()
        .map(|event| match event {
            TriggerEvent::Delete => "DELETE".to_string(),
            TriggerEvent::Insert => "INSERT".to_string(),
            TriggerEvent::Update { columns } if columns.is_empty() => "UPDATE".to_string(),
            TriggerEvent::Update { columns } => format!("UPDATE OF {}", columns.join(", ")),
            TriggerEvent::Truncate => "TRUNCATE".to_string(),
        })
        .collect::<Vec<_>>()
        .join(" OR ");
    let mut facts = vec![
        RenderFact::new(
            "Orientation",
            inline_code(match trigger.orientation {
                TriggerOrientation::Row => "row",
                TriggerOrientation::Statement => "statement",
            }),
        ),
        RenderFact::new("Function", inline_code(&trigger.function)),
        RenderFact::new(
            "Enabled",
            inline_code(match trigger.enabled {
                TriggerEnabled::Origin => "origin",
                TriggerEnabled::Disabled => "disabled",
                TriggerEnabled::Replica => "replica",
                TriggerEnabled::Always => "always",
            }),
        ),
    ];
    if !trigger.arguments.is_empty() {
        facts.push(RenderFact::new(
            "Arguments",
            inline_code(&trigger.arguments.join(", ")),
        ));
    }
    if let Some(constraint) = &trigger.constraint {
        let mut value = if constraint.deferrable {
            if constraint.initially_deferred {
                "deferrable initially deferred".to_string()
            } else {
                "deferrable initially immediate".to_string()
            }
        } else {
            "not deferrable".to_string()
        };
        if let Some(table) = &constraint.referenced_table {
            let _ = write!(value, "; from {}", inline_code(table));
        }
        facts.push(RenderFact::new("Constraint trigger", value));
    }
    if let Some(value) = &trigger.old_transition_table {
        facts.push(RenderFact::new("Old transition table", inline_code(value)));
    }
    if let Some(value) = &trigger.new_transition_table {
        facts.push(RenderFact::new("New transition table", inline_code(value)));
    }
    if let Some(value) = &trigger.parent_trigger {
        facts.push(RenderFact::new("Parent trigger", inline_code(value)));
    }
    RenderTrigger::builder()
        .qualified_name(inline_code(&identity))
        .file_name(object_file_name(
            &trigger.namespace,
            &format!("{}.{}", trigger.target, trigger.name),
        ))
        .event(format!(
            "{} {events}",
            match trigger.timing {
                TriggerTiming::Before => "BEFORE",
                TriggerTiming::After => "AFTER",
                TriggerTiming::InsteadOf => "INSTEAD OF",
            }
        ))
        .target(inline_code(&format!(
            "{}.{}",
            trigger.target_namespace, trigger.target
        )))
        .with_comment(trigger.comment.as_deref().map(text))
        .with_facts(facts)
        .with_when_expression(trigger.when_expression.as_deref().map(inline_code))
        .definition(code_block("sql", &trigger.definition))
        .build()
}

fn render_function(function: &Function) -> RenderFunction {
    RenderFunction {
        qualified_name: inline_code(&format!(
            "{}.{}{}",
            function.namespace, function.name, function.signature
        )),
        file_name: object_file_name(
            &function.namespace,
            &format!("{}{}", function.name, function.signature),
        ),
        comment: function.comment.as_deref().map(text),
        facts: vec![
            RenderFact::new("Returns", inline_code(&function.return_type)),
            RenderFact::new("Language", inline_code(&function.language)),
            RenderFact::new(
                "Volatility",
                inline_code(match function.volatility {
                    FunctionVolatility::Immutable => "immutable",
                    FunctionVolatility::Stable => "stable",
                    FunctionVolatility::Volatile => "volatile",
                }),
            ),
            RenderFact::new(
                "Parallel",
                inline_code(match function.parallel {
                    FunctionParallel::Safe => "safe",
                    FunctionParallel::Restricted => "restricted",
                    FunctionParallel::Unsafe => "unsafe",
                }),
            ),
            RenderFact::new(
                "Security",
                inline_code(if function.security_definer {
                    "definer"
                } else {
                    "invoker"
                }),
            ),
        ],
        definition: function
            .definition
            .as_deref()
            .map(|definition| code_block("sql", definition)),
    }
}
