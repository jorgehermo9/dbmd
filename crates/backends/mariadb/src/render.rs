use dbmd_core::SourceId;
use dbmd_relational::presentation::{
    self, ColumnView, ConstraintView, FactView, IndexView, NamespaceView, TableDetailsView,
    TableView, TriggerView, ViewPresentation,
};
use dbmd_relational::{ForeignKeyAction, ForeignKeyMatch, IndexSortOrder};
use dbmd_render::{code_block, inline_code, object_file_name, text, RenderSource, TemplateFile};
use serde::Serialize;

use super::{
    Account, AccountKind, Catalog, Column, Constraint, ConstraintKind, Index, LoadableFunction,
    Package, Plugin, Privilege, PrivilegeObjectKind, Routine, ServerDefinition, Table, Trigger,
    View,
};

const SINGLE_FILE_TEMPLATE: &str = "backends/mariadb/single_file/source.md.j2";
const DIRECTORY_TEMPLATE: &str = "backends/mariadb/directory/source.md.j2";
pub(crate) const TEMPLATES: &[TemplateFile] = &[
    TemplateFile::new(
        "single_file/backends/mariadb/source.md.j2",
        SINGLE_FILE_TEMPLATE,
        include_str!("templates/single_file/source.md.j2"),
    ),
    TemplateFile::new(
        "directory/backends/mariadb/source.md.j2",
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
    triggers: Vec<TriggerView>,
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
    let mut functions = catalog
        .routines
        .iter()
        .map(render_routine)
        .collect::<Vec<_>>();
    functions.extend(catalog.sequences.iter().map(|value| FunctionView {
        qualified_name: inline_code(&format!("{}.{}", value.schema, value.name)),
        file_name: object_file_name(&value.schema, &value.name),
        comment: value.comment.as_deref().map(text),
        facts: {
            let mut facts = vec![
                FactView::new("Kind", inline_code("sequence")),
                FactView::new("Type", inline_code(&value.data_type)),
                FactView::new("Start", inline_code(&value.start_value)),
                FactView::new("Minimum", inline_code(&value.minimum_value)),
                FactView::new("Maximum", inline_code(&value.maximum_value)),
                FactView::new("Increment", inline_code(&value.increment)),
                FactView::new("Cycle", if value.cycle { "yes" } else { "no" }),
            ];
            if let Some(cache) = value.cache {
                facts.push(FactView::new("Cache", inline_code(&cache.to_string())));
            }
            if let Some(engine) = &value.engine {
                facts.push(FactView::new("Engine", inline_code(engine)));
            }
            facts
        },
        definition: Some(code_block("sql", &value.definition)),
    }));
    functions.extend(catalog.events.iter().map(|value| FunctionView {
        qualified_name: inline_code(&format!("{}.{}", value.schema, value.name)),
        file_name: object_file_name(&value.schema, &value.name),
        comment: value.comment.as_deref().map(text),
        facts: {
            let mut facts = vec![
                FactView::new("Kind", inline_code("event")),
                FactView::new("Status", value.status.display_name()),
                FactView::new("Schedule", value.kind.display_name()),
                FactView::new("Completion", value.completion.display_name()),
                FactView::new("Definer", inline_code(&value.definer)),
                FactView::new("Time zone", inline_code(&value.time_zone)),
                FactView::new("SQL mode", inline_code(&value.sql_mode)),
                FactView::new("Originator", value.originator.to_string()),
                FactView::new(
                    "Character set client",
                    inline_code(&value.character_set_client),
                ),
                FactView::new(
                    "Connection collation",
                    inline_code(&value.collation_connection),
                ),
                FactView::new("Database collation", inline_code(&value.database_collation)),
            ];
            if let Some(execute_at) = &value.execute_at {
                facts.push(FactView::new("Execute at", inline_code(execute_at)));
            }
            if let Some(interval) = &value.interval_value {
                let unit = value.interval_unit.map_or("-", |unit| unit.display_name());
                facts.push(FactView::new(
                    "Interval",
                    format!("{} {}", inline_code(interval), unit),
                ));
            }
            for (label, instant) in [("Starts", &value.starts), ("Ends", &value.ends)] {
                if let Some(instant) = instant {
                    facts.push(FactView::new(label, inline_code(instant)));
                }
            }
            facts
        },
        definition: Some(code_block("sql", &value.create_statement)),
    }));
    functions.extend(catalog.servers.iter().map(render_server));
    functions.extend(
        catalog
            .loadable_functions
            .iter()
            .map(render_loadable_function),
    );
    functions.extend(catalog.plugins.iter().map(render_plugin));
    functions.extend(catalog.packages.iter().map(render_package));
    functions.extend(
        catalog
            .accounts
            .iter()
            .map(|account| render_account(account, catalog)),
    );
    let data = SourceData {
        section_heading,
        object_heading,
        detail_heading,
        namespaces: catalog
            .schemas
            .iter()
            .map(|value| {
                let mut description = format!(
                    "Default character set {}; collation {}.",
                    inline_code(&value.default_character_set),
                    inline_code(&value.default_collation)
                );
                if let Some(comment) = &value.comment {
                    description.push(' ');
                    description.push_str(&text(comment));
                }
                NamespaceView::new(inline_code(&value.name), Some(description))
            })
            .collect(),
        tables: catalog.tables.iter().map(render_table).collect(),
        views: catalog.views.iter().map(render_view).collect(),
        triggers: catalog.triggers.iter().map(render_trigger).collect(),
        functions,
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
        .chain(data.functions.iter().map(|value| {
            presentation::directory_object("objects", "function.md.j2", &value.file_name, value)
        }))
        .collect();
    RenderSource::builder(
        id.as_str(),
        "mariadb",
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
    if table.system_versioned {
        facts.push(FactView::new("System versioning", "enabled"));
    }
    if let Some(period) = &table.system_time_period {
        facts.push(FactView::new(
            "System-time period",
            inline_code(&format!("{}, {}", period.start_column, period.end_column)),
        ));
    }
    for period in &table.application_time_periods {
        facts.push(FactView::new(
            "Application-time period",
            format!(
                "{}: {} to {}",
                inline_code(&period.name),
                inline_code(&period.start_column),
                inline_code(&period.end_column)
            ),
        ));
    }
    for partition in &table.partitions {
        let (label, name, method, expression, ordinal) = if let Some(name) = &partition.subpartition
        {
            (
                "Subpartition",
                name,
                partition
                    .subpartition_method
                    .map(|method| method.display_name()),
                partition.subpartition_expression.as_deref(),
                partition.subpartition_ordinal,
            )
        } else {
            (
                "Partition",
                &partition.name,
                partition.method.map(|method| method.display_name()),
                partition.expression.as_deref(),
                Some(partition.ordinal),
            )
        };
        let mut value = format!(
            "{}: method={}, expression={}, boundary={}, position={}",
            inline_code(name),
            inline_code(method.unwrap_or("-")),
            inline_code(expression.unwrap_or("-")),
            inline_code(partition.description.as_deref().unwrap_or("-")),
            ordinal.map_or_else(|| "-".to_string(), |value| value.to_string())
        );
        for (property, property_value) in [
            ("tablespace", partition.tablespace.as_deref()),
            ("nodegroup", partition.nodegroup.as_deref()),
            ("comment", partition.comment.as_deref()),
        ] {
            if let Some(property_value) = property_value {
                value.push_str(&format!("; {property}={}", inline_code(property_value)));
            }
        }
        facts.push(FactView::new(label, value));
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
                .title("MariaDB")
                .facts(facts)
                .notices(Vec::new())
                .definition(Some(code_block("sql", &table.definition)))
                .build(),
        )
        .build()
}
fn render_column(column: &Column) -> ColumnView {
    let mut notes = column
        .comment
        .as_deref()
        .map(text)
        .into_iter()
        .collect::<Vec<_>>();
    if !column.extra.is_empty() {
        notes.push(inline_code(&column.extra));
    }
    if let Some(value) = &column.generation_expression {
        let storage = column
            .generated_storage
            .map_or("generated", |storage| storage.display_name());
        notes.push(format!("{storage} generated as {}", inline_code(value)));
    }
    if !column.visible {
        notes.push("invisible".to_string());
    }
    if column.system_time_period_start {
        notes.push("system-time period start".to_string());
    }
    if column.system_time_period_end {
        notes.push("system-time period end".to_string());
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
    };
    let mut details = value.referenced_table.as_ref().map_or_else(
        || {
            value
                .expression
                .as_deref()
                .map_or_else(|| "-".to_string(), inline_code)
        },
        |table| {
            let mut details = format!(
                "references {}.{} ({})",
                inline_code(value.referenced_schema.as_deref().unwrap_or("-")),
                inline_code(table),
                inline_code(&value.referenced_columns.join(", "))
            );
            if let Some(match_type) = &value.match_type {
                details.push_str(&format!("; match {}", foreign_key_match_name(match_type)));
            }
            if let Some(action) = value.on_update {
                details.push_str(&format!("; on update {}", foreign_key_action_name(action)));
            }
            if let Some(action) = value.on_delete {
                details.push_str(&format!("; on delete {}", foreign_key_action_name(action)));
            }
            details
        },
    );
    if let Some(period) = &value.period {
        let suffix = format!("period {} without overlaps", inline_code(period));
        if details == "-" {
            details = suffix;
        } else {
            details.push_str("; ");
            details.push_str(&suffix);
        }
    }
    if let Some(level) = value.check_level {
        details.push_str(&format!("; declared at {} level", level.display_name()));
    }
    ConstraintView::builder()
        .name(inline_code(&value.name))
        .kind(inline_code(kind))
        .columns(inline_code(&value.columns.join(", ")))
        .details(details)
        .build()
}

fn foreign_key_match_name(value: &ForeignKeyMatch) -> &str {
    match value {
        ForeignKeyMatch::Simple => "simple",
        ForeignKeyMatch::Partial => "partial",
        ForeignKeyMatch::Full => "full",
        ForeignKeyMatch::Named(name) => name,
        _ => "backend-defined",
    }
}

const fn foreign_key_action_name(value: ForeignKeyAction) -> &'static str {
    match value {
        ForeignKeyAction::NoAction => "no action",
        ForeignKeyAction::Restrict => "restrict",
        ForeignKeyAction::SetNull => "set null",
        ForeignKeyAction::SetDefault => "set default",
        ForeignKeyAction::Cascade => "cascade",
    }
}

fn render_index(value: &Index) -> IndexView {
    let terms = value
        .terms
        .iter()
        .map(|term| {
            let mut value = term.column.clone();
            if let Some(prefix) = term.prefix_length {
                value.push_str(&format!("({prefix})"));
            }
            match term.sort_order {
                Some(IndexSortOrder::Descending) => value.push_str(" DESC"),
                Some(IndexSortOrder::Ascending) | None => {}
            }
            value
        })
        .collect::<Vec<_>>()
        .join(", ");
    let mut origin = value.index_type.clone();
    if value.ignored == Some(true) {
        origin.push_str("; ignored");
    }
    if let Some(comment) = &value.comment {
        origin.push_str(&format!("; comment {}", inline_code(comment)));
    }
    if let Some(comment) = &value.catalog_comment {
        origin.push_str(&format!("; catalog status {}", inline_code(comment)));
    }
    if let Some(period) = &value.period {
        origin.push_str(&format!("; period {period} without overlaps"));
    }
    if let Some(options) = &value.vector_options {
        if let Some(m) = options.m {
            origin.push_str(&format!("; M={m}"));
        }
        if let Some(distance) = &options.distance {
            origin.push_str(&format!("; distance={distance}"));
        }
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
            FactView::new("Check option", view.check_option.display_name()),
            FactView::new("Updatable", if view.updatable { "yes" } else { "no" }),
            FactView::new("Security", view.security.display_name()),
            FactView::new("Algorithm", view.algorithm.display_name()),
            FactView::new("Definer", inline_code(&view.definer)),
        ])
        .definition(code_block("sql", &view.create_statement))
        .build()
}
fn render_trigger(trigger: &Trigger) -> TriggerView {
    TriggerView::builder()
        .qualified_name(inline_code(&format!("{}.{}", trigger.schema, trigger.name)))
        .file_name(object_file_name(&trigger.schema, &trigger.name))
        .event(format!(
            "{} {}",
            inline_code(trigger.timing.display_name()),
            inline_code(
                &trigger
                    .events
                    .iter()
                    .map(|event| event.display_name())
                    .collect::<Vec<_>>()
                    .join(", "),
            )
        ))
        .target(inline_code(&format!(
            "{}.{}",
            trigger.schema, trigger.table
        )))
        .with_facts({
            let mut facts = vec![
                FactView::new("Orientation", trigger.orientation.display_name()),
                FactView::new("Order", trigger.action_order.to_string()),
                FactView::new("Definer", inline_code(&trigger.definer)),
                FactView::new("SQL mode", inline_code(&trigger.sql_mode)),
                FactView::new(
                    "Character set client",
                    inline_code(&trigger.character_set_client),
                ),
                FactView::new(
                    "Connection collation",
                    inline_code(&trigger.collation_connection),
                ),
                FactView::new(
                    "Database collation",
                    inline_code(&trigger.database_collation),
                ),
            ];
            if !trigger.update_columns.is_empty() {
                facts.push(FactView::new(
                    "Update columns",
                    inline_code(&trigger.update_columns.join(", ")),
                ));
            }
            facts
        })
        .definition(code_block("sql", &trigger.create_statement))
        .build()
}
fn render_routine(routine: &Routine) -> FunctionView {
    FunctionView {
        qualified_name: inline_code(&format!("{}.{}", routine.schema, routine.name)),
        file_name: object_file_name(&routine.schema, &routine.name),
        comment: routine.comment.as_deref().map(text),
        facts: vec![
            FactView::new("Kind", routine.kind.display_name()),
            FactView::new("Data access", routine.data_access.display_name()),
            FactView::new(
                "Deterministic",
                if routine.deterministic { "yes" } else { "no" },
            ),
            FactView::new("Security", routine.security.display_name()),
            FactView::new("Definer", inline_code(&routine.definer)),
            FactView::new("SQL mode", inline_code(&routine.sql_mode)),
            FactView::new(
                "Character set client",
                inline_code(&routine.character_set_client),
            ),
            FactView::new(
                "Connection collation",
                inline_code(&routine.collation_connection),
            ),
            FactView::new(
                "Database collation",
                inline_code(&routine.database_collation),
            ),
            FactView::new(
                "Parameters",
                inline_code(
                    &routine
                        .parameters
                        .iter()
                        .map(|value| {
                            let mut parameter = format!(
                                "{}{} {}",
                                value
                                    .mode
                                    .map(|mode| format!("{} ", mode.display_name()))
                                    .unwrap_or_default(),
                                value.name.as_deref().unwrap_or("return"),
                                value.dtd_identifier
                            );
                            if let Some(default) = &value.default {
                                parameter.push_str(" default ");
                                parameter.push_str(default);
                            }
                            parameter
                        })
                        .collect::<Vec<_>>()
                        .join(", "),
                ),
            ),
        ],
        definition: Some(code_block("sql", &routine.create_statement)),
    }
}

fn render_loadable_function(function: &LoadableFunction) -> FunctionView {
    FunctionView {
        qualified_name: inline_code(&function.name),
        file_name: object_file_name("loadable-function", &function.name),
        comment: None,
        facts: vec![
            FactView::new("Kind", function.kind.display_name()),
            FactView::new("Return type", function.return_type.display_name()),
            FactView::new("Library", inline_code(&function.library)),
        ],
        definition: None,
    }
}

fn render_plugin(plugin: &Plugin) -> FunctionView {
    let identity = format!("{} ({})", plugin.name, plugin.kind.display_name());
    let mut facts = vec![
        FactView::new("Kind", inline_code("plugin")),
        FactView::new("Type", plugin.kind.display_name()),
        FactView::new("Version", inline_code(&plugin.version)),
        FactView::new("Status", plugin.status.display_name()),
        FactView::new("Type version", inline_code(&plugin.type_version)),
        FactView::new("License", plugin.license.display_name()),
        FactView::new("Load option", plugin.load_option.display_name()),
        FactView::new("Maturity", plugin.maturity.display_name()),
    ];
    for (label, value) in [
        ("Library", plugin.library.as_deref()),
        ("Library version", plugin.library_version.as_deref()),
        (
            "Authentication version",
            plugin.authentication_version.as_deref(),
        ),
        ("Author", plugin.author.as_deref()),
    ] {
        if let Some(value) = value {
            facts.push(FactView::new(label, inline_code(value)));
        }
    }
    FunctionView {
        qualified_name: inline_code(&identity),
        file_name: object_file_name("plugin", &identity),
        comment: plugin.description.as_deref().map(text),
        facts,
        definition: None,
    }
}

fn render_server(server: &ServerDefinition) -> FunctionView {
    let mut facts = vec![
        FactView::new("Kind", inline_code("server")),
        FactView::new("Wrapper", inline_code(&server.wrapper)),
    ];
    for (label, value) in [
        ("Host", server.host.as_deref()),
        ("Database", server.database.as_deref()),
        ("Username", server.username.as_deref()),
        ("Socket", server.socket.as_deref()),
        ("Owner", server.owner.as_deref()),
    ] {
        if let Some(value) = value {
            facts.push(FactView::new(label, inline_code(value)));
        }
    }
    if let Some(port) = server.port {
        facts.push(FactView::new("Port", inline_code(&port.to_string())));
    }
    for option in &server.options {
        let value = if option.sensitive {
            "[redacted]".to_string()
        } else {
            option
                .value
                .as_deref()
                .map_or_else(|| "-".to_string(), inline_code)
        };
        facts.push(FactView::new(
            "Option",
            format!("{}: {value}", inline_code(&option.name)),
        ));
    }
    FunctionView {
        qualified_name: inline_code(&server.name),
        file_name: object_file_name("server", &server.name),
        comment: None,
        facts,
        definition: None,
    }
}

fn render_package(package: &Package) -> FunctionView {
    let mut definition = package.specification.definition.clone();
    if let Some(body) = &package.body {
        definition.push_str("\n\n");
        definition.push_str(&body.definition);
    }
    FunctionView {
        qualified_name: inline_code(&format!("{}.{}", package.schema, package.name)),
        file_name: object_file_name(&package.schema, &package.name),
        comment: package.comment.as_deref().map(text),
        facts: vec![
            FactView::new("Kind", inline_code("package")),
            FactView::new("Security", package.security.display_name()),
            FactView::new("Definer", inline_code(&package.definer)),
            FactView::new("SQL mode", inline_code(&package.specification.sql_mode)),
            FactView::new("Body", if package.body.is_some() { "yes" } else { "no" }),
        ],
        definition: Some(code_block("sql", &definition)),
    }
}

fn render_account(account: &Account, catalog: &Catalog) -> FunctionView {
    let kind = match account.kind {
        AccountKind::User => "user",
        AccountKind::Role => "role",
    };
    let mut facts = vec![
        FactView::new("Kind", inline_code(kind)),
        FactView::new("Host", inline_code(&account.host)),
    ];
    if !account.authentication_plugins.is_empty() {
        facts.push(FactView::new(
            "Authentication",
            inline_code(&account.authentication_plugins.join(", ")),
        ));
    }
    if account.password_expired {
        facts.push(FactView::new("Password expired", "yes"));
    }
    if let Some(days) = account.password_lifetime_days {
        facts.push(FactView::new("Password lifetime", format!("{days} days")));
    }
    if account.account_locked {
        facts.push(FactView::new("Account locked", "yes"));
    }
    if let Some(role) = &account.default_role {
        facts.push(FactView::new("Default role", inline_code(role)));
    }
    facts.push(FactView::new("TLS", account.tls_requirement.display_name()));
    for (label, value) in [
        ("TLS cipher", account.tls_cipher.as_deref()),
        ("X.509 issuer", account.x509_issuer.as_deref()),
        ("X.509 subject", account.x509_subject.as_deref()),
    ] {
        if let Some(value) = value {
            facts.push(FactView::new(label, inline_code(value)));
        }
    }
    for (label, value) in [
        ("Queries per hour", account.max_queries_per_hour),
        ("Updates per hour", account.max_updates_per_hour),
        ("Connections per hour", account.max_connections_per_hour),
        ("Concurrent connections", account.max_user_connections),
    ] {
        if let Some(value) = value {
            facts.push(FactView::new(label, value.to_string()));
        }
    }
    if let Some(value) = &account.max_statement_time {
        facts.push(FactView::new("Maximum statement time", inline_code(value)));
    }
    for membership in catalog
        .role_memberships
        .iter()
        .filter(|membership| membership.user == account.name && membership.host == account.host)
    {
        facts.push(FactView::new(
            "Role",
            format!(
                "{}{}",
                inline_code(&membership.role),
                if membership.admin_option {
                    " with admin option"
                } else {
                    ""
                }
            ),
        ));
    }
    let grantee = quoted_account(&account.name, &account.host);
    for privilege in catalog
        .privileges
        .iter()
        .filter(|privilege| privilege.grantee == grantee)
    {
        facts.push(FactView::new("Privilege", render_privilege(privilege)));
    }
    let identity = format!("{}@{}", account.name, account.host);
    FunctionView {
        qualified_name: inline_code(&identity),
        file_name: object_file_name("account", &identity),
        comment: None,
        facts,
        definition: None,
    }
}

fn quoted_account(name: &str, host: &str) -> String {
    format!(
        "'{}'@'{}'",
        name.replace('\'', "''"),
        host.replace('\'', "''")
    )
}

fn render_privilege(privilege: &Privilege) -> String {
    let target = match privilege.object_kind {
        PrivilegeObjectKind::Global => "global".to_string(),
        PrivilegeObjectKind::Schema => {
            format!("schema {}", privilege.schema.as_deref().unwrap_or("-"))
        }
        PrivilegeObjectKind::Table
        | PrivilegeObjectKind::Function
        | PrivilegeObjectKind::Procedure
        | PrivilegeObjectKind::Package
        | PrivilegeObjectKind::PackageBody => format!(
            "{} {}.{}",
            privilege.object_kind.display_name(),
            privilege.schema.as_deref().unwrap_or("-"),
            privilege.object.as_deref().unwrap_or("-")
        ),
        PrivilegeObjectKind::Column => format!(
            "column {}.{}.{}",
            privilege.schema.as_deref().unwrap_or("-"),
            privilege.object.as_deref().unwrap_or("-"),
            privilege.column.as_deref().unwrap_or("-")
        ),
        PrivilegeObjectKind::Proxy => format!(
            "proxy account {}",
            privilege.object.as_deref().unwrap_or("-")
        ),
    };
    format!(
        "{} on {}{}",
        inline_code(&privilege.privilege),
        inline_code(&target),
        if privilege.grantable {
            " with grant option"
        } else {
            ""
        }
    )
}
