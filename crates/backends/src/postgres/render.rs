use std::fmt::Write as _;

use dbmd_render::{
    code_block, inline_code, object_file_name, text, RenderColumn, RenderConstraint, RenderEnum,
    RenderFact, RenderFunction, RenderIndex, RenderSource, RenderTable, RenderTableDetails,
    RenderTrigger, RenderView, TemplateFile,
};

use super::catalog::{
    Column, Constraint, Function, FunctionParallel, FunctionVolatility, Index, PolicyCommand,
    Snapshot, Table, TableKind, Trigger, TriggerEnabled, TriggerEvent, TriggerOrientation,
    TriggerTiming, View,
};
use crate::render_support;

pub(super) const SINGLE_FILE_TEMPLATE: &str = "backends/postgres/single_file/source.md.j2";
pub(super) const DIRECTORY_TEMPLATE: &str = "backends/postgres/directory/source.md.j2";

pub(crate) const TEMPLATES: &[TemplateFile] = &[
    TemplateFile {
        relative_path: "single_file/backends/postgres/source.md.j2",
        template_name: SINGLE_FILE_TEMPLATE,
        contents: include_str!("templates/single_file/source.md.j2"),
    },
    TemplateFile {
        relative_path: "directory/backends/postgres/source.md.j2",
        template_name: DIRECTORY_TEMPLATE,
        contents: include_str!("templates/directory/source.md.j2"),
    },
];

pub(crate) fn source(snapshot: &Snapshot, nested: bool) -> RenderSource {
    let catalog = snapshot.catalog();
    let (section_heading, object_heading, detail_heading) = if nested {
        ("###", "####", "#####")
    } else {
        ("##", "###", "####")
    };
    RenderSource {
        id: snapshot.id().as_str().to_string(),
        name: inline_code(snapshot.display_name().unwrap_or(snapshot.id().as_str())),
        has_display_name: snapshot.display_name().is_some(),
        backend: "postgres",
        single_file_template: SINGLE_FILE_TEMPLATE,
        directory_template: DIRECTORY_TEMPLATE,
        nested,
        section_heading,
        object_heading,
        detail_heading,
        namespaces: render_support::namespaces(&catalog.namespaces),
        enums: catalog
            .enums
            .iter()
            .map(|enum_type| RenderEnum {
                heading: object_heading,
                qualified_name: inline_code(&format!("{}.{}", enum_type.namespace, enum_type.name)),
                file_name: object_file_name(&enum_type.namespace, &enum_type.name),
                comment: enum_type.comment.as_deref().map(text),
                values: inline_code(&enum_type.values.join(", ")),
            })
            .collect(),
        tables: catalog
            .tables
            .iter()
            .map(|table| render_table(table, object_heading, detail_heading))
            .collect(),
        views: catalog
            .views
            .iter()
            .map(|view| render_view(view, object_heading))
            .collect(),
        triggers: catalog
            .triggers
            .iter()
            .map(|trigger| render_trigger(trigger, object_heading))
            .collect(),
        functions: catalog
            .functions
            .iter()
            .map(|function| render_function(function, object_heading))
            .collect(),
    }
}

fn render_table(table: &Table, heading: &'static str, detail_heading: &'static str) -> RenderTable {
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
    RenderTable {
        heading,
        detail_heading,
        qualified_name: inline_code(&table.qualified_name()),
        file_name: object_file_name(&table.namespace, &table.name),
        comment: table.comment.as_deref().map(text),
        columns: table.columns.iter().map(render_column).collect(),
        constraints: table.constraints.iter().map(render_constraint).collect(),
        indexes: table.indexes.iter().map(render_index).collect(),
        backend: RenderTableDetails {
            title: "PostgreSQL",
            facts,
            notices,
            definition: None,
        },
    }
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
    RenderColumn {
        name: inline_code(&column.name),
        data_type: inline_code(&column.data_type),
        nullable: render_support::nullable(column.nullable),
        default: column
            .default
            .as_deref()
            .map_or_else(|| "-".to_string(), inline_code),
        notes: notes.join("; "),
    }
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
    RenderConstraint {
        name: constraint
            .name
            .as_deref()
            .map_or_else(|| "-".to_string(), inline_code),
        kind: inline_code(render_support::constraint_kind(&constraint.kind)),
        columns: inline_code(&constraint.columns.join(", ")),
        details,
    }
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
    RenderIndex {
        name: inline_code(&index.name),
        terms: render_support::index_terms(&index.terms),
        unique: if index.unique { "yes" } else { "no" },
        origin,
        predicate: index
            .predicate
            .as_deref()
            .map_or_else(|| "-".to_string(), inline_code),
    }
}

fn render_view(view: &View, heading: &'static str) -> RenderView {
    RenderView {
        heading,
        qualified_name: inline_code(&format!("{}.{}", view.namespace, view.name)),
        file_name: object_file_name(&view.namespace, &view.name),
        columns: view.columns.iter().map(render_column).collect(),
        definition: code_block("sql", &view.definition),
    }
}

fn render_trigger(trigger: &Trigger, heading: &'static str) -> RenderTrigger {
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
    RenderTrigger {
        heading,
        qualified_name: inline_code(&format!("{}.{}", trigger.namespace, trigger.name)),
        file_name: object_file_name(&trigger.namespace, &trigger.name),
        event: format!(
            "{} {events}",
            match trigger.timing {
                TriggerTiming::Before => "BEFORE",
                TriggerTiming::After => "AFTER",
                TriggerTiming::InsteadOf => "INSTEAD OF",
            }
        ),
        target: inline_code(&format!("{}.{}", trigger.target_namespace, trigger.target)),
        facts,
        when_expression: trigger.when_expression.as_deref().map(inline_code),
        definition: code_block("sql", &trigger.definition),
    }
}

fn render_function(function: &Function, heading: &'static str) -> RenderFunction {
    RenderFunction {
        heading,
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
