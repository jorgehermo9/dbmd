use std::fmt::Write as _;

use dbmd_core::{
    Backend, ClickHouseTable, Column, ColumnBackend, Constraint, ConstraintBackend, ConstraintKind,
    DatabaseContext, EnumType, ForeignKeyAction, ForeignKeyInitialTiming, Function,
    FunctionBackend, Index, IndexBackend, IndexNullsOrder, IndexSortOrder, IndexTarget, Namespace,
    PostgresFunctionParallel, PostgresFunctionVolatility, PostgresPolicyCommand, PostgresTable,
    PostgresTableKind, SourceSnapshot, SqliteColumnKind, SqliteConflictResolution,
    SqliteIndexOrigin, SqliteTable, SqliteTableKind, Table, TableBackend, Trigger, TriggerEvent,
    TriggerTiming, View,
};
use serde::Serialize;

/// Presentation-only data supplied to templates.
///
/// This type contains no connection settings, environment values, driver
/// handles, or internal error values.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RenderContext {
    version: u32,
    pub(crate) sources: Vec<RenderSource>,
}

impl From<&DatabaseContext> for RenderContext {
    fn from(database: &DatabaseContext) -> Self {
        Self::new(database, database.sources().len() > 1)
    }
}

impl RenderContext {
    pub(crate) fn new(database: &DatabaseContext, nested: bool) -> Self {
        Self {
            version: 1,
            sources: database
                .sources()
                .iter()
                .map(|source| RenderSource::new(source, nested))
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct RenderSource {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) has_display_name: bool,
    pub(crate) backend: &'static str,
    pub(crate) nested: bool,
    pub(crate) section_heading: &'static str,
    pub(crate) object_heading: &'static str,
    pub(crate) detail_heading: &'static str,
    pub(crate) namespaces: Vec<RenderNamespace>,
    pub(crate) enums: Vec<RenderEnum>,
    pub(crate) tables: Vec<RenderTable>,
    pub(crate) views: Vec<RenderView>,
    pub(crate) triggers: Vec<RenderTrigger>,
    pub(crate) functions: Vec<RenderFunction>,
}

impl RenderSource {
    fn new(source: &SourceSnapshot, nested: bool) -> Self {
        let (section_heading, object_heading, detail_heading) = if nested {
            ("###", "####", "#####")
        } else {
            ("##", "###", "####")
        };
        Self {
            id: source.id.as_str().to_string(),
            name: inline_code(
                source
                    .display_name
                    .as_deref()
                    .unwrap_or_else(|| source.id.as_str()),
            ),
            has_display_name: source.display_name.is_some(),
            backend: backend_name(source.backend),
            nested,
            section_heading,
            object_heading,
            detail_heading,
            namespaces: source
                .namespaces
                .iter()
                .map(RenderNamespace::from)
                .collect(),
            enums: source.enums.iter().map(RenderEnum::from).collect(),
            tables: source
                .tables
                .iter()
                .map(|table| RenderTable::new(table, object_heading, detail_heading))
                .collect(),
            views: source
                .views
                .iter()
                .map(|view| RenderView::new(view, object_heading))
                .collect(),
            triggers: source
                .triggers
                .iter()
                .map(|trigger| RenderTrigger::new(trigger, object_heading))
                .collect(),
            functions: source
                .functions
                .iter()
                .map(|function| RenderFunction::new(function, object_heading))
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct RenderNamespace {
    name: String,
    comment: Option<String>,
}

impl From<&Namespace> for RenderNamespace {
    fn from(namespace: &Namespace) -> Self {
        Self {
            name: inline_code(&namespace.name),
            comment: namespace.comment.as_deref().map(text),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct RenderEnum {
    pub(crate) heading: &'static str,
    qualified_name: String,
    pub(crate) file_name: String,
    comment: Option<String>,
    values: String,
}

impl From<&EnumType> for RenderEnum {
    fn from(enum_type: &EnumType) -> Self {
        Self {
            heading: "###",
            qualified_name: inline_code(&format!("{}.{}", enum_type.namespace, enum_type.name)),
            file_name: object_file_name(&enum_type.namespace, &enum_type.name),
            comment: enum_type.comment.as_deref().map(text),
            values: inline_code(&enum_type.values.join(", ")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct RenderTable {
    pub(crate) heading: &'static str,
    pub(crate) detail_heading: &'static str,
    pub(crate) qualified_name: String,
    pub(crate) file_name: String,
    pub(crate) comment: Option<String>,
    pub(crate) columns: Vec<RenderColumn>,
    pub(crate) constraints: Vec<RenderConstraint>,
    pub(crate) indexes: Vec<RenderIndex>,
    pub(crate) backend: RenderTableDetails,
}

impl RenderTable {
    pub(crate) fn new(table: &Table, heading: &'static str, detail_heading: &'static str) -> Self {
        Self {
            heading,
            detail_heading,
            qualified_name: inline_code(&table.qualified_name()),
            file_name: object_file_name(&table.namespace, &table.name),
            comment: table.comment.as_deref().map(text),
            columns: table.columns.iter().map(RenderColumn::from).collect(),
            constraints: table
                .constraints
                .iter()
                .map(RenderConstraint::from)
                .collect(),
            indexes: table.indexes.iter().map(RenderIndex::from).collect(),
            backend: RenderTableDetails::from(&table.backend),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct RenderColumn {
    name: String,
    data_type: String,
    nullable: &'static str,
    default: String,
    notes: String,
}

impl From<&Column> for RenderColumn {
    fn from(column: &Column) -> Self {
        let mut notes = Vec::new();
        if let Some(comment) = &column.comment {
            notes.push(text(comment));
        }
        match &column.backend {
            ColumnBackend::Common => {}
            ColumnBackend::Postgres(postgres) => {
                if let Some(identity) = &postgres.identity {
                    notes.push(format!("identity {}", inline_code(identity)));
                }
                if let Some(generated) = &postgres.generated {
                    notes.push(format!("generated as {}", inline_code(generated)));
                }
                if !postgres.enum_values.is_empty() {
                    notes.push(format!(
                        "enum values {}",
                        inline_code(&postgres.enum_values.join(", "))
                    ));
                }
            }
            ColumnBackend::ClickHouse(clickhouse) => {
                if let Some(codec) = &clickhouse.codec {
                    notes.push(format!("codec {}", inline_code(codec)));
                }
                if let Some(ttl) = &clickhouse.ttl {
                    notes.push(format!("TTL {}", inline_code(ttl)));
                }
            }
            ColumnBackend::Sqlite(sqlite) => {
                if sqlite.kind != SqliteColumnKind::Normal {
                    notes.push(sqlite_column_kind(sqlite.kind).to_string());
                }
                if sqlite.collation != "BINARY" {
                    notes.push(format!("collate {}", inline_code(&sqlite.collation)));
                }
                if let Some(expression) = &sqlite.generated_expression {
                    notes.push(format!("as {}", inline_code(expression)));
                }
            }
        }
        Self {
            name: inline_code(&column.name),
            data_type: inline_code(&column.data_type),
            nullable: match column.nullable {
                Some(true) => "yes",
                Some(false) => "no",
                None => "unknown",
            },
            default: column
                .default
                .as_deref()
                .map_or_else(|| "-".to_string(), inline_code),
            notes: notes.join("; "),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct RenderConstraint {
    name: String,
    kind: String,
    columns: String,
    details: String,
}

impl From<&Constraint> for RenderConstraint {
    fn from(constraint: &Constraint) -> Self {
        let mut details = if let Some(reference) = &constraint.references {
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
            if let Some(match_name) = &reference.match_name {
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
        };
        match &constraint.backend {
            ConstraintBackend::Common => {}
            ConstraintBackend::Postgres(postgres) => {
                details = inline_code(&postgres.definition);
                if !postgres.validated {
                    details.push_str("; not validated");
                }
                if !postgres.locally_defined {
                    details.push_str("; inherited");
                }
                if postgres.no_inherit {
                    details.push_str("; no inherit");
                }
            }
            ConstraintBackend::Sqlite(sqlite) => {
                if let Some(conflict) = sqlite.conflict_resolution {
                    let _ = write!(
                        details,
                        "; conflict {}",
                        inline_code(sqlite_conflict(conflict))
                    );
                }
                if sqlite.auto_increment {
                    details.push_str("; autoincrement");
                }
            }
        }
        Self {
            name: constraint
                .name
                .as_deref()
                .map_or_else(|| "-".to_string(), inline_code),
            kind: inline_code(constraint_kind(&constraint.kind)),
            columns: inline_code(&constraint.columns.join(", ")),
            details,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct RenderIndex {
    name: String,
    terms: String,
    unique: &'static str,
    origin: String,
    predicate: String,
}

impl From<&Index> for RenderIndex {
    fn from(index: &Index) -> Self {
        let terms = index
            .terms
            .iter()
            .map(|term| {
                let target = match &term.target {
                    IndexTarget::Column(column) | IndexTarget::Expression(column) => {
                        inline_code(column)
                    }
                    IndexTarget::RowId => inline_code("rowid"),
                };
                let mut rendered = format!("{target} {}", index_order(term.order));
                if let Some(collation) = &term.collation {
                    let _ = write!(rendered, " collate {}", inline_code(collation));
                }
                if let Some(operator_class) = &term.operator_class {
                    let _ = write!(rendered, " opclass {}", inline_code(operator_class));
                }
                if let Some(nulls_order) = term.nulls_order {
                    let nulls_order = match nulls_order {
                        IndexNullsOrder::First => "first",
                        IndexNullsOrder::Last => "last",
                    };
                    let _ = write!(rendered, " nulls {}", inline_code(nulls_order));
                }
                rendered
            })
            .collect::<Vec<_>>()
            .join(", ");
        let origin = match &index.backend {
            IndexBackend::Sqlite(sqlite) => inline_code(sqlite_index_origin(sqlite.origin)),
            IndexBackend::Postgres(postgres) => {
                let mut details = format!("postgres {}", inline_code(&postgres.method));
                if !postgres.included_columns.is_empty() {
                    let _ = write!(
                        details,
                        "; include {}",
                        inline_code(&postgres.included_columns.join(", "))
                    );
                }
                if postgres.nulls_not_distinct {
                    details.push_str("; nulls not distinct");
                }
                if !postgres.valid {
                    details.push_str("; invalid");
                }
                if !postgres.ready {
                    details.push_str("; not ready");
                }
                if postgres.clustered {
                    details.push_str("; clustered");
                }
                if postgres.replica_identity {
                    details.push_str("; replica identity");
                }
                details
            }
            IndexBackend::ClickHouse(_) => inline_code("clickhouse"),
            IndexBackend::Common => inline_code("common"),
        };
        Self {
            name: inline_code(&index.name),
            terms,
            unique: if index.unique { "yes" } else { "no" },
            origin,
            predicate: index
                .predicate
                .as_deref()
                .map_or_else(|| "-".to_string(), inline_code),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct RenderTableDetails {
    title: &'static str,
    facts: Vec<RenderFact>,
    notices: Vec<&'static str>,
    definition: Option<String>,
}

impl From<&TableBackend> for RenderTableDetails {
    fn from(backend: &TableBackend) -> Self {
        match backend {
            TableBackend::Sqlite(sqlite) => sqlite_table_details(sqlite),
            TableBackend::Postgres(postgres) => postgres_table_details(postgres),
            TableBackend::ClickHouse(clickhouse) => clickhouse_table_details(clickhouse),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct RenderFact {
    label: &'static str,
    value: String,
}

impl RenderFact {
    fn new(label: &'static str, value: impl Into<String>) -> Self {
        Self {
            label,
            value: value.into(),
        }
    }
}

fn sqlite_table_details(table: &SqliteTable) -> RenderTableDetails {
    let kind = match &table.kind {
        SqliteTableKind::Ordinary => inline_code("ordinary"),
        SqliteTableKind::Virtual { module, arguments } => {
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
        SqliteTableKind::Shadow { virtual_table } => {
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
    RenderTableDetails {
        title: "SQLite",
        facts: vec![RenderFact::new("Kind", kind)],
        notices,
        definition: table
            .definition
            .as_deref()
            .map(|definition| code_block("sql", definition)),
    }
}

fn postgres_table_details(table: &PostgresTable) -> RenderTableDetails {
    let mut facts = vec![RenderFact::new(
        "Kind",
        inline_code(postgres_table_kind(&table.table_kind)),
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
    if let Some(partition) = &table.partition_key {
        facts.push(RenderFact::new("Partition key", inline_code(partition)));
    }
    if let Some(parent) = &table.partition_parent {
        facts.push(RenderFact::new("Partition parent", inline_code(parent)));
    }
    if let Some(bound) = &table.partition_bound {
        facts.push(RenderFact::new("Partition bound", inline_code(bound)));
    }
    for policy in &table.policies {
        let mut value = format!(
            "{} {} to {} ({})",
            inline_code(&policy.name),
            inline_code(postgres_policy_command(policy.command)),
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
    RenderTableDetails {
        title: "PostgreSQL",
        facts,
        notices,
        definition: None,
    }
}

fn clickhouse_table_details(table: &ClickHouseTable) -> RenderTableDetails {
    let mut facts = vec![RenderFact::new(
        "Engine",
        inline_code(&table.engine_clause()),
    )];
    if !table.order_by.is_empty() {
        facts.push(RenderFact::new(
            "Order by",
            inline_code(&table.order_by.join(", ")),
        ));
    }
    if !table.primary_key.is_empty() {
        facts.push(RenderFact::new(
            "Primary key",
            inline_code(&table.primary_key.join(", ")),
        ));
    }
    if let Some(partition) = &table.partition_by {
        facts.push(RenderFact::new("Partition by", inline_code(partition)));
    }
    if let Some(sample) = &table.sample_by {
        facts.push(RenderFact::new("Sample by", inline_code(sample)));
    }
    if let Some(ttl) = &table.ttl {
        facts.push(RenderFact::new("TTL", inline_code(ttl)));
    }
    if !table.settings.is_empty() {
        facts.push(RenderFact::new(
            "Settings",
            table
                .settings
                .iter()
                .map(|(key, value)| inline_code(&format!("{key} = {value}")))
                .collect::<Vec<_>>()
                .join(", "),
        ));
    }
    RenderTableDetails {
        title: "ClickHouse",
        facts,
        notices: Vec::new(),
        definition: None,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct RenderView {
    pub(crate) heading: &'static str,
    qualified_name: String,
    pub(crate) file_name: String,
    columns: Vec<RenderColumn>,
    definition: String,
}

impl RenderView {
    fn new(view: &View, heading: &'static str) -> Self {
        Self {
            heading,
            qualified_name: inline_code(&format!("{}.{}", view.namespace, view.name)),
            file_name: object_file_name(&view.namespace, &view.name),
            columns: view.columns.iter().map(RenderColumn::from).collect(),
            definition: code_block("sql", &view.definition),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct RenderTrigger {
    pub(crate) heading: &'static str,
    qualified_name: String,
    pub(crate) file_name: String,
    event: String,
    target: String,
    when_expression: Option<String>,
    definition: String,
}

impl RenderTrigger {
    fn new(trigger: &Trigger, heading: &'static str) -> Self {
        let event = match &trigger.event {
            TriggerEvent::Delete => "DELETE".to_string(),
            TriggerEvent::Insert => "INSERT".to_string(),
            TriggerEvent::Update { columns } if columns.is_empty() => "UPDATE".to_string(),
            TriggerEvent::Update { columns } => {
                format!("UPDATE OF {}", columns.join(", "))
            }
        };
        Self {
            heading,
            qualified_name: inline_code(&format!("{}.{}", trigger.namespace, trigger.name)),
            file_name: object_file_name(&trigger.namespace, &trigger.name),
            event: format!("{} {event}", trigger_timing(trigger.timing)),
            target: inline_code(&format!("{}.{}", trigger.target_namespace, trigger.target)),
            when_expression: trigger.when_expression.as_deref().map(inline_code),
            definition: code_block("sql", &trigger.definition),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct RenderFunction {
    pub(crate) heading: &'static str,
    qualified_name: String,
    pub(crate) file_name: String,
    comment: Option<String>,
    facts: Vec<RenderFact>,
    definition: Option<String>,
}

impl RenderFunction {
    fn new(function: &Function, heading: &'static str) -> Self {
        Self {
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
            facts: match &function.backend {
                FunctionBackend::Common => Vec::new(),
                FunctionBackend::Postgres(postgres) => vec![
                    RenderFact::new("Returns", inline_code(&postgres.return_type)),
                    RenderFact::new("Language", inline_code(&postgres.language)),
                    RenderFact::new(
                        "Volatility",
                        inline_code(postgres_function_volatility(postgres.volatility)),
                    ),
                    RenderFact::new(
                        "Parallel",
                        inline_code(postgres_function_parallel(postgres.parallel)),
                    ),
                    RenderFact::new(
                        "Security",
                        inline_code(if postgres.security_definer {
                            "definer"
                        } else {
                            "invoker"
                        }),
                    ),
                ],
            },
            definition: function
                .definition
                .as_deref()
                .map(|definition| code_block("sql", definition)),
        }
    }
}

fn backend_name(backend: Backend) -> &'static str {
    match backend {
        Backend::Sqlite => "sqlite",
        Backend::Postgres => "postgres",
        Backend::ClickHouse => "clickhouse",
    }
}

fn constraint_kind(kind: &ConstraintKind) -> &'static str {
    match kind {
        ConstraintKind::PrimaryKey => "primary_key",
        ConstraintKind::ForeignKey => "foreign_key",
        ConstraintKind::Unique => "unique",
        ConstraintKind::Check => "check",
        ConstraintKind::NotNull => "not_null",
        ConstraintKind::Exclusion => "exclusion",
    }
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

fn sqlite_conflict(conflict: SqliteConflictResolution) -> &'static str {
    match conflict {
        SqliteConflictResolution::Rollback => "rollback",
        SqliteConflictResolution::Abort => "abort",
        SqliteConflictResolution::Fail => "fail",
        SqliteConflictResolution::Ignore => "ignore",
        SqliteConflictResolution::Replace => "replace",
    }
}

fn index_order(order: IndexSortOrder) -> &'static str {
    match order {
        IndexSortOrder::Ascending => "ascending",
        IndexSortOrder::Descending => "descending",
    }
}

fn sqlite_index_origin(origin: SqliteIndexOrigin) -> &'static str {
    match origin {
        SqliteIndexOrigin::CreateIndex => "create_index",
        SqliteIndexOrigin::UniqueConstraint => "unique_constraint",
        SqliteIndexOrigin::PrimaryKey => "primary_key",
    }
}

fn sqlite_column_kind(kind: SqliteColumnKind) -> &'static str {
    match kind {
        SqliteColumnKind::Normal => "normal",
        SqliteColumnKind::VirtualTableHidden => "virtual_table_hidden",
        SqliteColumnKind::VirtualGenerated => "virtual_generated",
        SqliteColumnKind::StoredGenerated => "stored_generated",
    }
}

fn postgres_table_kind(kind: &PostgresTableKind) -> &'static str {
    match kind {
        PostgresTableKind::Table => "table",
        PostgresTableKind::PartitionedTable => "partitioned_table",
        PostgresTableKind::Partition => "partition",
        PostgresTableKind::ForeignTable => "foreign_table",
    }
}

fn postgres_policy_command(command: PostgresPolicyCommand) -> &'static str {
    match command {
        PostgresPolicyCommand::All => "all",
        PostgresPolicyCommand::Select => "select",
        PostgresPolicyCommand::Insert => "insert",
        PostgresPolicyCommand::Update => "update",
        PostgresPolicyCommand::Delete => "delete",
    }
}

fn postgres_function_volatility(volatility: PostgresFunctionVolatility) -> &'static str {
    match volatility {
        PostgresFunctionVolatility::Immutable => "immutable",
        PostgresFunctionVolatility::Stable => "stable",
        PostgresFunctionVolatility::Volatile => "volatile",
    }
}

fn postgres_function_parallel(parallel: PostgresFunctionParallel) -> &'static str {
    match parallel {
        PostgresFunctionParallel::Safe => "safe",
        PostgresFunctionParallel::Restricted => "restricted",
        PostgresFunctionParallel::Unsafe => "unsafe",
    }
}

fn trigger_timing(timing: TriggerTiming) -> &'static str {
    match timing {
        TriggerTiming::Before => "BEFORE",
        TriggerTiming::After => "AFTER",
        TriggerTiming::InsteadOf => "INSTEAD OF",
    }
}

fn inline_code(value: &str) -> String {
    let longest_run = longest_backtick_run(value);
    let fence = "`".repeat(longest_run.saturating_add(1).max(1));
    let padding = value.starts_with('`') || value.ends_with('`');
    let rendered = if padding {
        format!("{fence} {value} {fence}")
    } else {
        format!("{fence}{value}{fence}")
    };
    table_cell(&rendered)
}

fn code_block(language: &str, value: &str) -> String {
    let fence = "`".repeat(longest_backtick_run(value).saturating_add(1).max(3));
    format!("{fence}{language}\n{value}\n{fence}")
}

fn longest_backtick_run(value: &str) -> usize {
    value
        .split(|character| character != '`')
        .map(str::len)
        .max()
        .unwrap_or(0)
}

fn text(value: &str) -> String {
    table_cell(value)
}

fn table_cell(value: &str) -> String {
    value
        .replace('|', "\\|")
        .replace("\r\n", "<br>")
        .replace(['\r', '\n'], "<br>")
}

fn object_file_name(namespace: &str, name: &str) -> String {
    format!("{}.{}.md", path_component(namespace), path_component(name))
}

fn path_component(value: &str) -> String {
    let mut encoded = String::with_capacity(value.len());
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-') {
            encoded.push(char::from(byte));
        } else {
            let _ = write!(encoded, "%{byte:02X}");
        }
    }
    encoded
}
