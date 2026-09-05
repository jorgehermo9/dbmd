use std::fmt::Write as _;

use dbmd_render::{code_block, inline_code, object_file_name, text, RenderSource, TemplateFile};
use serde::Serialize;

use super::catalog::{
    Catalog, Column, ColumnKind, ConflictResolution, Constraint, ConstraintKind, Index,
    IndexOrigin, IndexTarget, IndexTerm, Table, TableKind, Trigger, TriggerEvent, TriggerTiming,
    View,
};
use dbmd_core::SourceId;
use dbmd_relational::presentation::{
    self, ColumnView as RenderColumn, ConstraintView as RenderConstraint, FactView as RenderFact,
    IndexView as RenderIndex, NamespaceView, TableDetailsView as RenderTableDetails,
    TableView as RenderTable, TriggerView as RenderTrigger, ViewPresentation as RenderView,
};
use dbmd_relational::{ForeignKeyAction, ForeignKeyInitialTiming, ForeignKeyMatch, IndexSortOrder};

pub(super) const SINGLE_FILE_TEMPLATE: &str = "backends/sqlite/single_file/source.md.j2";
pub(super) const DIRECTORY_TEMPLATE: &str = "backends/sqlite/directory/source.md.j2";

pub(crate) const TEMPLATES: &[TemplateFile] = &[
    TemplateFile::new(
        "single_file/backends/sqlite/source.md.j2",
        SINGLE_FILE_TEMPLATE,
        include_str!("templates/single_file/source.md.j2"),
    ),
    TemplateFile::new(
        "directory/backends/sqlite/source.md.j2",
        DIRECTORY_TEMPLATE,
        include_str!("templates/directory/source.md.j2"),
    ),
];

#[derive(Serialize)]
struct SourceData {
    section_heading: &'static str,
    object_heading: &'static str,
    detail_heading: &'static str,
    namespaces: Vec<NamespaceView>,
    tables: Vec<RenderTable>,
    views: Vec<RenderView>,
    triggers: Vec<RenderTrigger>,
}

pub(crate) fn source(
    id: &SourceId,
    display_name: Option<&str>,
    catalog: &Catalog,
    nested: bool,
) -> RenderSource {
    let (section_heading, object_heading, detail_heading) = headings(nested);
    let data = SourceData {
        section_heading,
        object_heading,
        detail_heading,
        namespaces: presentation::namespaces(&catalog.namespaces),
        tables: catalog.tables.iter().map(render_table).collect(),
        views: catalog.views.iter().map(render_view).collect(),
        triggers: catalog.triggers.iter().map(render_trigger).collect(),
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
        .chain(data.triggers.iter().map(|object| {
            presentation::directory_object("triggers", "trigger.md.j2", &object.file_name, object)
        }))
        .collect();
    RenderSource::builder(
        id.as_str(),
        "sqlite",
        (SINGLE_FILE_TEMPLATE, DIRECTORY_TEMPLATE),
        &data,
    )
    .display_name(display_name.map(inline_code))
    .nested(nested)
    .objects(objects)
    .build()
}

fn headings(nested: bool) -> (&'static str, &'static str, &'static str) {
    if nested {
        ("###", "####", "#####")
    } else {
        ("##", "###", "####")
    }
}

fn render_table(table: &Table) -> RenderTable {
    let kind = match &table.kind {
        TableKind::Ordinary => inline_code("ordinary"),
        TableKind::Virtual { module, arguments } => {
            let mut value = format!("{} using {}", inline_code("virtual"), inline_code(module));
            if !arguments.is_empty() {
                let _ = write!(
                    value,
                    " with arguments {}",
                    inline_code(&arguments.join(", "))
                );
            }
            value
        }
        TableKind::Shadow { virtual_table } => {
            let mut value = inline_code("shadow");
            if let Some(owner) = virtual_table {
                let _ = write!(value, " owned by {}", inline_code(owner));
            }
            value
        }
    };
    let mut notices = Vec::new();
    if table.strict {
        notices.push("Strict table.");
    }
    if table.without_rowid {
        notices.push("Without rowid.");
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
                .title("SQLite")
                .facts(vec![RenderFact::new("Kind", kind)])
                .notices(notices)
                .definition(
                    table
                        .definition
                        .as_deref()
                        .map(|definition| code_block("sql", definition)),
                )
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
    if column.kind != ColumnKind::Normal {
        notes.push(
            match column.kind {
                ColumnKind::Normal => "normal",
                ColumnKind::VirtualTableHidden => "virtual_table_hidden",
                ColumnKind::VirtualGenerated => "virtual_generated",
                ColumnKind::StoredGenerated => "stored_generated",
            }
            .to_string(),
        );
    }
    if column.collation != "BINARY" {
        notes.push(format!("collate {}", inline_code(&column.collation)));
    }
    if let Some(expression) = &column.generated_expression {
        notes.push(format!("as {}", inline_code(expression)));
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
    let mut details = relational_constraint_details(constraint);
    if let Some(conflict) = constraint.conflict_resolution {
        let conflict = match conflict {
            ConflictResolution::Rollback => "rollback",
            ConflictResolution::Abort => "abort",
            ConflictResolution::Fail => "fail",
            ConflictResolution::Ignore => "ignore",
            ConflictResolution::Replace => "replace",
        };
        let _ = write!(details, "; conflict {}", inline_code(conflict));
    }
    if constraint.auto_increment {
        details.push_str("; autoincrement");
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

fn relational_constraint_details(constraint: &Constraint) -> String {
    if let Some(reference) = &constraint.references {
        let mut details = format!(
            "references {}; update {}, delete {}",
            inline_code(&format!(
                "{}.{}({})",
                reference.namespace,
                reference.table,
                reference.columns.join(", ")
            )),
            inline_code(foreign_key_action(reference.on_update)),
            inline_code(foreign_key_action(reference.on_delete))
        );
        if let Some(match_type) = &reference.match_type {
            let match_name = match match_type {
                ForeignKeyMatch::Simple => "simple",
                ForeignKeyMatch::Partial => "partial",
                ForeignKeyMatch::Full => "full",
                ForeignKeyMatch::Named(name) => name,
                _ => {
                    unreachable!("new foreign-key match variants require SQLite rendering support")
                }
            };
            let _ = write!(details, "; match {}", inline_code(match_name));
        }
        if reference.deferrability.deferrable {
            let _ = write!(
                details,
                "; deferrable initially {}",
                inline_code(initial_timing(reference.deferrability.initially))
            );
        } else {
            details.push_str("; not deferrable");
        }
        details
    } else if let Some(expression) = &constraint.expression {
        inline_code(expression)
    } else {
        "-".to_string()
    }
}

fn render_index(index: &Index) -> RenderIndex {
    RenderIndex::builder()
        .name(inline_code(&index.name))
        .terms(index_terms(&index.terms))
        .unique(if index.unique { "yes" } else { "no" })
        .origin(inline_code(match index.origin {
            IndexOrigin::CreateIndex => "create_index",
            IndexOrigin::UniqueConstraint => "unique_constraint",
            IndexOrigin::PrimaryKey => "primary_key",
        }))
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
        ConstraintKind::NotNull => "not_null",
    }
}

fn index_terms(terms: &[IndexTerm]) -> String {
    terms
        .iter()
        .map(|term| {
            let target = match &term.target {
                IndexTarget::Column(value) | IndexTarget::Expression(value) => inline_code(value),
                IndexTarget::RowId => inline_code("rowid"),
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
            rendered
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn render_view(view: &View) -> RenderView {
    RenderView::builder()
        .qualified_name(inline_code(&format!("{}.{}", view.namespace, view.name)))
        .file_name(object_file_name(&view.namespace, &view.name))
        .comment(None)
        .facts(Vec::new())
        .columns(view.columns.iter().map(render_column).collect())
        .definition(code_block("sql", &view.definition))
        .build()
}

fn render_trigger(trigger: &Trigger) -> RenderTrigger {
    let identity = format!("{}.{}.{}", trigger.namespace, trigger.target, trigger.name);
    let event = match &trigger.event {
        TriggerEvent::Delete => "DELETE".to_string(),
        TriggerEvent::Insert => "INSERT".to_string(),
        TriggerEvent::Update { columns } if columns.is_empty() => "UPDATE".to_string(),
        TriggerEvent::Update { columns } => format!("UPDATE OF {}", columns.join(", ")),
    };
    RenderTrigger::builder()
        .qualified_name(inline_code(&identity))
        .file_name(object_file_name(
            &trigger.namespace,
            &format!("{}.{}", trigger.target, trigger.name),
        ))
        .event(format!("{} {event}", trigger_timing(trigger.timing)))
        .target(inline_code(&format!(
            "{}.{}",
            trigger.target_namespace, trigger.target
        )))
        .with_when_expression(trigger.when_expression.as_deref().map(inline_code))
        .definition(code_block("sql", &trigger.definition))
        .build()
}

fn foreign_key_action(action: ForeignKeyAction) -> &'static str {
    match action {
        ForeignKeyAction::NoAction => "no_action",
        ForeignKeyAction::Restrict => "restrict",
        ForeignKeyAction::SetNull => "set_null",
        ForeignKeyAction::SetDefault => "set_default",
        ForeignKeyAction::Cascade => "cascade",
    }
}

fn initial_timing(timing: ForeignKeyInitialTiming) -> &'static str {
    match timing {
        ForeignKeyInitialTiming::Immediate => "immediate",
        ForeignKeyInitialTiming::Deferred => "deferred",
    }
}

fn trigger_timing(timing: TriggerTiming) -> &'static str {
    match timing {
        TriggerTiming::Before => "BEFORE",
        TriggerTiming::After => "AFTER",
        TriggerTiming::InsteadOf => "INSTEAD OF",
    }
}
