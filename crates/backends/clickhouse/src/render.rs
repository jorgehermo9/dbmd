//! ClickHouse presentation mapping.

use dbmd_core::SourceId;
use dbmd_relational::presentation::{
    self, ColumnView, ConstraintView, FactView, IndexView, NamespaceView, TableDetailsView,
    TableView, ViewPresentation,
};
use dbmd_render::{code_block, inline_code, object_file_name, text, RenderSource, TemplateFile};
use serde::Serialize;

use super::{
    AccessTarget, Catalog, Column, Grant, QuotaLimit, RefreshSchedule, ResourceOperation, Table,
    TableKind, TtlAction, TtlDestination, ViewSqlSecurity,
};

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
    access_objects: Vec<FunctionView>,
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
                            "Engine {}; UUID {}; {}{}",
                            inline_code(if database.engine_full.is_empty() {
                                &database.engine
                            } else {
                                &database.engine_full
                            }),
                            inline_code(&database.uuid),
                            text(comment),
                            if database.external { "; external" } else { "" }
                        ),
                        None => format!(
                            "Engine {}; UUID {}{}",
                            inline_code(if database.engine_full.is_empty() {
                                &database.engine
                            } else {
                                &database.engine_full
                            }),
                            inline_code(&database.uuid),
                            if database.external { "; external" } else { "" }
                        ),
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
                facts: function_facts(function),
                definition: Some(code_block("sql", &function.definition)),
            })
            .collect(),
        access_objects: render_access_objects(catalog),
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
        .chain(data.access_objects.iter().map(|object| {
            presentation::directory_object(
                "access-and-workloads",
                "function.md.j2",
                &object.file_name,
                object,
            )
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

fn function_facts(function: &super::UserDefinedFunction) -> Vec<FactView> {
    let mut facts = vec![
        FactView::new("Kind", inline_code("user_defined_function")),
        FactView::new("Origin", inline_code(function.origin.display_name())),
    ];
    for (label, value) in [
        ("Syntax", &function.syntax),
        ("Arguments", &function.arguments),
        ("Returns", &function.returned_value),
    ] {
        if let Some(value) = value {
            facts.push(FactView::new(label, inline_code(value)));
        }
    }
    facts
}

fn render_access_objects(catalog: &Catalog) -> Vec<FunctionView> {
    let mut objects = Vec::new();
    for user in &catalog.users {
        let mut facts = vec![
            FactView::new("Kind", inline_code("user")),
            FactView::new("Storage", inline_code(&user.storage)),
            FactView::new(
                "Authentication",
                inline_code(&user.authentication_types.join(", ")),
            ),
            FactView::new("Hosts", inline_code(&render_hosts(&user.hosts))),
            FactView::new(
                "Default roles",
                inline_code(&render_target(&user.default_roles)),
            ),
            FactView::new("Grantees", inline_code(&render_target(&user.grantees))),
        ];
        if let Some(database) = &user.default_database {
            facts.push(FactView::new("Default database", inline_code(database)));
        }
        let expirations = user
            .valid_until
            .iter()
            .flatten()
            .cloned()
            .collect::<Vec<_>>();
        if !expirations.is_empty() {
            facts.push(FactView::new(
                "Credentials valid until",
                inline_code(&expirations.join(", ")),
            ));
        }
        facts.extend(render_subject_grants(catalog, Some(&user.name), None));
        objects.push(object_view("user", &user.name, facts, None));
    }
    for role in &catalog.roles {
        let mut facts = vec![
            FactView::new("Kind", inline_code("role")),
            FactView::new("Storage", inline_code(&role.storage)),
        ];
        facts.extend(render_subject_grants(catalog, None, Some(&role.name)));
        objects.push(object_view("role", &role.name, facts, None));
    }
    for policy in &catalog.row_policies {
        let mut facts = vec![
            FactView::new("Kind", inline_code("row_policy")),
            FactView::new("Database", inline_code(&policy.database)),
            FactView::new(
                "Table",
                policy
                    .table
                    .as_deref()
                    .map_or_else(|| "-".to_string(), inline_code),
            ),
            FactView::new(
                "Mode",
                inline_code(if policy.restrictive {
                    "restrictive"
                } else {
                    "permissive"
                }),
            ),
            FactView::new("Applies to", inline_code(&render_target(&policy.target))),
            FactView::new("Storage", inline_code(&policy.storage)),
        ];
        if let Some(filter) = &policy.select_filter {
            facts.push(FactView::new("SELECT filter", inline_code(filter)));
        }
        objects.push(object_view("row-policy", &policy.name, facts, None));
    }
    for quota in &catalog.quotas {
        let mut facts = vec![
            FactView::new("Kind", inline_code("quota")),
            FactView::new("Storage", inline_code(&quota.storage)),
            FactView::new("Keys", inline_code(&quota.keys.join(", "))),
            FactView::new("Applies to", inline_code(&render_target(&quota.target))),
        ];
        if let Some(bits) = quota.ipv4_prefix_bits {
            facts.push(FactView::new("IPv4 prefix bits", bits.to_string()));
        }
        if let Some(bits) = quota.ipv6_prefix_bits {
            facts.push(FactView::new("IPv6 prefix bits", bits.to_string()));
        }
        facts.extend(
            quota
                .limits
                .iter()
                .map(|limit| FactView::new("Limit", inline_code(&render_quota_limit(limit)))),
        );
        objects.push(object_view("quota", &quota.name, facts, None));
    }
    for profile in &catalog.settings_profiles {
        let mut facts = vec![
            FactView::new("Kind", inline_code("settings_profile")),
            FactView::new("Storage", inline_code(&profile.storage)),
            FactView::new("Applies to", inline_code(&render_target(&profile.target))),
        ];
        facts.extend(profile.elements.iter().map(|element| {
            let mut parts = Vec::new();
            if let Some(name) = &element.setting_name {
                parts.push(name.clone());
            }
            if let Some(value) = &element.value {
                parts.push(format!("= {value}"));
            }
            if let Some(minimum) = &element.minimum {
                parts.push(format!("min {minimum}"));
            }
            if let Some(maximum) = &element.maximum {
                parts.push(format!("max {maximum}"));
            }
            if let Some(writability) = &element.writability {
                parts.push(writability.to_ascii_lowercase());
            }
            if let Some(parent) = &element.inherited_profile {
                parts.push(format!("inherits {parent}"));
            }
            FactView::new("Element", inline_code(&parts.join(" ")))
        }));
        objects.push(object_view("settings-profile", &profile.name, facts, None));
    }
    for collection in &catalog.named_collections {
        let mut facts = vec![
            FactView::new("Kind", inline_code("named_collection")),
            FactView::new("Source", inline_code(&collection.source)),
        ];
        facts.extend(collection.entries.iter().map(|entry| {
            let override_contract = match entry.overridable {
                Some(true) => "overridable",
                Some(false) => "not overridable",
                None => "override policy unknown",
            };
            FactView::new(
                "Entry",
                format!("{}; {override_contract}", inline_code(&entry.key)),
            )
        }));
        objects.push(object_view(
            "named-collection",
            &collection.name,
            facts,
            collection
                .definition
                .as_deref()
                .map(|definition| code_block("sql", definition)),
        ));
    }
    for resource in &catalog.resources {
        let mut facts = vec![
            FactView::new("Kind", inline_code("resource")),
            FactView::new("Unit", inline_code(&resource.unit)),
        ];
        facts.extend(resource.operations.iter().map(|operation| {
            FactView::new(
                "Operation",
                inline_code(&render_resource_operation(operation)),
            )
        }));
        if !resource.read_disks.is_empty() {
            facts.push(FactView::new(
                "Read disks",
                inline_code(&resource.read_disks.join(", ")),
            ));
        }
        if !resource.write_disks.is_empty() {
            facts.push(FactView::new(
                "Write disks",
                inline_code(&resource.write_disks.join(", ")),
            ));
        }
        objects.push(object_view(
            "resource",
            &resource.name,
            facts,
            Some(code_block("sql", &resource.definition)),
        ));
    }
    for workload in &catalog.workloads {
        let mut facts = vec![FactView::new("Kind", inline_code("workload"))];
        if let Some(parent) = &workload.parent {
            facts.push(FactView::new("Parent", inline_code(parent)));
        }
        facts.extend(workload.settings.iter().map(|setting| {
            FactView::new(
                "Setting",
                format!(
                    "{} = {}{}",
                    inline_code(&setting.name),
                    inline_code(&setting.value),
                    setting
                        .resource
                        .as_ref()
                        .map_or_else(String::new, |resource| {
                            format!(" for {}", inline_code(resource))
                        })
                ),
            )
        }));
        objects.push(object_view(
            "workload",
            &workload.name,
            facts,
            Some(code_block("sql", &workload.definition)),
        ));
    }
    objects
}

fn render_resource_operation(operation: &ResourceOperation) -> String {
    match operation {
        ResourceOperation::MasterThread => "master thread".to_string(),
        ResourceOperation::WorkerThread => "worker thread".to_string(),
        ResourceOperation::Query => "query".to_string(),
        ResourceOperation::MemoryReservation => "memory reservation".to_string(),
        ResourceOperation::ReadDisk { disk } => disk.as_ref().map_or_else(
            || "read any disk".to_string(),
            |disk| format!("read disk {disk}"),
        ),
        ResourceOperation::WriteDisk { disk } => disk.as_ref().map_or_else(
            || "write any disk".to_string(),
            |disk| format!("write disk {disk}"),
        ),
        ResourceOperation::Unknown { raw } => raw.clone(),
    }
}

fn object_view(
    kind: &str,
    name: &str,
    facts: Vec<FactView>,
    definition: Option<String>,
) -> FunctionView {
    FunctionView {
        qualified_name: inline_code(name),
        file_name: object_file_name(kind, name),
        comment: None,
        facts,
        definition,
    }
}

fn render_hosts(hosts: &super::UserHosts) -> String {
    let mut values = Vec::new();
    values.extend(hosts.ip.iter().map(|value| format!("ip {value}")));
    values.extend(hosts.names.iter().map(|value| format!("name {value}")));
    values.extend(
        hosts
            .name_regexps
            .iter()
            .map(|value| format!("regexp {value}")),
    );
    values.extend(
        hosts
            .name_like_patterns
            .iter()
            .map(|value| format!("like {value}")),
    );
    if values.is_empty() {
        "none".to_string()
    } else {
        values.join(", ")
    }
}

fn render_target(target: &AccessTarget) -> String {
    if target.all {
        if target.except.is_empty() {
            "all".to_string()
        } else {
            format!("all except {}", target.except.join(", "))
        }
    } else if target.include.is_empty() {
        "none".to_string()
    } else {
        target.include.join(", ")
    }
}

fn render_subject_grants(
    catalog: &Catalog,
    user: Option<&str>,
    role: Option<&str>,
) -> Vec<FactView> {
    let privileges = catalog
        .grants
        .iter()
        .filter(|grant| grant.user.as_deref() == user && grant.role.as_deref() == role);
    let role_grants = catalog
        .role_grants
        .iter()
        .filter(|grant| grant.user.as_deref() == user && grant.role.as_deref() == role);
    privileges
        .map(|grant| FactView::new("Privilege", inline_code(&render_grant(grant))))
        .chain(role_grants.map(|grant| {
            let mut value = grant.granted_role.clone();
            if grant.default {
                value.push_str(" default");
            }
            if grant.admin_option {
                value.push_str(" with admin option");
            }
            FactView::new("Role grant", inline_code(&value))
        }))
        .collect()
}

fn render_grant(grant: &Grant) -> String {
    let mut value = if grant.partial_revoke {
        format!("revoke {}", grant.access_type)
    } else {
        grant.access_type.clone()
    };
    let mut scope = Vec::new();
    if let Some(access_object) = &grant.access_object {
        scope.push(access_object.clone());
    }
    if let Some(database) = &grant.database {
        scope.push(database.clone());
    }
    if let Some(table) = &grant.table {
        scope.push(table.clone());
    }
    if let Some(column) = &grant.column {
        scope.push(column.clone());
    }
    if !scope.is_empty() {
        value.push_str(" on ");
        value.push_str(&scope.join("."));
    }
    if grant.grant_option {
        value.push_str(" with grant option");
    }
    value
}

fn render_quota_limit(limit: &QuotaLimit) -> String {
    let mut values = vec![format!("{} seconds", limit.duration_seconds)];
    if limit.randomized {
        values.push("randomized".to_string());
    }
    for (name, value) in [
        ("queries", limit.max_queries),
        ("query_selects", limit.max_query_selects),
        ("query_inserts", limit.max_query_inserts),
        ("errors", limit.max_errors),
        ("result_rows", limit.max_result_rows),
        ("result_bytes", limit.max_result_bytes),
        ("read_rows", limit.max_read_rows),
        ("read_bytes", limit.max_read_bytes),
        ("written_bytes", limit.max_written_bytes),
        (
            "failed_sequential_authentications",
            limit.max_failed_sequential_authentications,
        ),
        (
            "queries_per_normalized_hash",
            limit.max_queries_per_normalized_hash,
        ),
    ] {
        if let Some(value) = value {
            values.push(format!("{name}={value}"));
        }
    }
    if let Some(value) = &limit.max_execution_time {
        values.push(format!("execution_time={value}"));
    }
    values.join(", ")
}

fn render_table(table: &Table) -> TableView {
    let mut facts = table_facts(table);
    for projection in &table.projections {
        let role = projection.index.as_ref().map_or_else(
            || {
                format!(
                    "{} sorted by {}",
                    projection.projection_type, projection.sorting_key
                )
            },
            |index| format!("index {} type {}", index.expression, index.index_type),
        );
        let settings = if projection.settings.is_empty() {
            String::new()
        } else {
            format!(
                "; settings {}",
                projection
                    .settings
                    .iter()
                    .map(|(key, value)| format!("{key}={value}"))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        };
        facts.push(FactView::new(
            "Projection",
            format!(
                "{} {}: {}{}",
                inline_code(&projection.name),
                inline_code(&role),
                inline_code(&projection.query),
                settings
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
                        .kind(inline_code(constraint.kind.display_name()))
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
    let mut facts = vec![
        FactView::new("Kind", inline_code(kind_name(table.kind))),
        FactView::new("UUID", inline_code(&table.uuid)),
    ];
    if table.temporary {
        facts.push(FactView::new("Temporary", "yes"));
    }
    if !table.engine_full.is_empty() {
        facts.push(FactView::new("Engine", inline_code(&table.engine_full)));
    }
    for (position, argument) in table.engine_arguments.iter().enumerate() {
        facts.push(FactView::new(
            "Engine argument",
            format!("{}: {}", position + 1, inline_code(argument)),
        ));
    }
    for (name, value) in &table.engine_parameters {
        facts.push(FactView::new(
            "Engine parameter",
            format!("{} = {}", inline_code(name), inline_code(value)),
        ));
    }
    for (label, value) in [
        ("Partition key", &table.partition_key),
        ("Primary key", &table.primary_key),
        ("Sorting key", &table.sorting_key),
        ("Sampling key", &table.sampling_key),
        ("Unique key", &table.unique_key),
        ("Storage policy", &table.storage_policy),
    ] {
        if !value.is_empty() {
            facts.push(FactView::new(label, inline_code(value)));
        }
    }
    for ttl in &table.ttl_rules {
        facts.push(FactView::new(
            "TTL",
            format!(
                "{}; {}",
                inline_code(&ttl.expression),
                inline_code(&render_ttl_action(&ttl.action))
            ),
        ));
    }
    for (name, value) in &table.settings {
        facts.push(FactView::new(
            "Setting",
            format!("{} = {}", inline_code(name), inline_code(value)),
        ));
    }
    if let Some(target) = &table.target {
        facts.push(FactView::new("Target", inline_code(target)));
    }
    if let Some(refresh) = &table.refresh {
        facts.push(FactView::new(
            "Refresh",
            inline_code(&match &refresh.schedule {
                RefreshSchedule::Every { interval } => format!("every {interval}"),
                RefreshSchedule::After { interval } => format!("after {interval}"),
                RefreshSchedule::DependenciesOnly => "after dependencies".to_string(),
            }),
        ));
        if let Some(offset) = &refresh.offset {
            facts.push(FactView::new("Refresh offset", inline_code(offset)));
        }
        if let Some(randomize_for) = &refresh.randomize_for {
            facts.push(FactView::new(
                "Refresh randomization",
                inline_code(randomize_for),
            ));
        }
        facts.push(FactView::new(
            "Refresh mode",
            inline_code(if refresh.append { "append" } else { "replace" }),
        ));
        for dependency in &refresh.dependencies {
            facts.push(FactView::new(
                "Refresh depends on",
                inline_code(&format!("{}.{}", dependency.database, dependency.table)),
            ));
        }
        for (name, value) in &refresh.settings {
            facts.push(FactView::new(
                "Refresh setting",
                format!("{} = {}", inline_code(name), inline_code(value)),
            ));
        }
    }
    if let Some(window) = &table.window {
        for (label, value) in [
            ("Window inner engine", &window.inner_engine),
            ("Window storage engine", &window.storage_engine),
            ("Watermark", &window.watermark),
            ("Allowed lateness", &window.allowed_lateness),
        ] {
            if let Some(value) = value {
                facts.push(FactView::new(label, inline_code(value)));
            }
        }
    }
    if let Some(definer) = &table.definer {
        facts.push(FactView::new("Definer", inline_code(definer)));
    }
    if let Some(security) = &table.sql_security {
        facts.push(FactView::new(
            "SQL security",
            inline_code(match security {
                ViewSqlSecurity::Definer => "definer",
                ViewSqlSecurity::Invoker => "invoker",
                ViewSqlSecurity::None => "none",
                ViewSqlSecurity::Unknown { raw } => raw,
            }),
        ));
    }
    if let Some(query) = &table.as_select {
        facts.push(FactView::new("AS SELECT", inline_code(query)));
    }
    for parameter in &table.parameters {
        facts.push(FactView::new(
            "Parameter",
            format!(
                "{} {}",
                inline_code(&parameter.name),
                inline_code(&parameter.data_type)
            ),
        ));
    }
    for (label, references) in [
        ("Depends on", &table.dependencies),
        ("Loads after", &table.loading_dependencies),
        ("Loads before", &table.loading_dependents),
    ] {
        for reference in references {
            facts.push(FactView::new(
                label,
                inline_code(&format!("{}.{}", reference.database, reference.table)),
            ));
        }
    }
    if let Some(dictionary) = &table.dictionary {
        facts.push(FactView::new(
            "Dictionary layout",
            inline_code(&dictionary.layout),
        ));
        facts.push(FactView::new(
            "Dictionary keys",
            inline_code(
                &dictionary
                    .keys
                    .iter()
                    .map(render_dictionary_field)
                    .collect::<Vec<_>>()
                    .join(", "),
            ),
        ));
        facts.push(FactView::new(
            "Dictionary attributes",
            inline_code(
                &dictionary
                    .attributes
                    .iter()
                    .map(render_dictionary_field)
                    .collect::<Vec<_>>()
                    .join(", "),
            ),
        ));
        facts.push(FactView::new(
            "Dictionary source",
            inline_code(&dictionary.source),
        ));
        facts.push(FactView::new(
            "Dictionary lifetime",
            format!(
                "{}..{} seconds",
                dictionary.lifetime_min_seconds, dictionary.lifetime_max_seconds
            ),
        ));
        if let (Some(min), Some(max)) = (&dictionary.range_min, &dictionary.range_max) {
            facts.push(FactView::new(
                "Dictionary range",
                format!("MIN {} MAX {}", inline_code(min), inline_code(max)),
            ));
        }
        for (name, value) in &dictionary.settings {
            facts.push(FactView::new(
                "Dictionary setting",
                format!("{} = {}", inline_code(name), inline_code(value)),
            ));
        }
    }
    facts
}

fn render_dictionary_field(field: &super::DictionaryField) -> String {
    let mut value = format!("{} {}", field.name, field.data_type);
    if let Some(default) = &field.default_expression {
        value.push_str(&format!(" DEFAULT {default}"));
    }
    if let Some(expression) = &field.expression {
        value.push_str(&format!(" EXPRESSION {expression}"));
    }
    if field.hierarchical {
        value.push_str(" HIERARCHICAL");
    }
    if field.injective {
        value.push_str(" INJECTIVE");
    }
    if field.object_id {
        value.push_str(" IS_OBJECT_ID");
    }
    value
}

fn render_column(column: &Column) -> ColumnView {
    let mut notes = Vec::new();
    if let Some(comment) = &column.comment {
        notes.push(text(comment));
    }
    if let Some(codec) = &column.compression_codec {
        notes.push(format!("codec {}", inline_code(codec)));
    }
    if let Some(serialization_hint) = &column.serialization_hint {
        notes.push(format!("serialization {}", inline_code(serialization_hint)));
    }
    if let Some(statistics) = &column.statistics {
        notes.push(format!("statistics {}", inline_code(statistics)));
    }
    if let Some(ttl) = &column.ttl {
        notes.push(format!("TTL {}", inline_code(ttl)));
    }
    if let Some(precision) = column.numeric_precision {
        let radix = column.numeric_precision_radix.unwrap_or(10);
        notes.push(format!("precision {precision} base {radix}"));
    }
    if let Some(scale) = column.numeric_scale {
        notes.push(format!("scale {scale}"));
    }
    if let Some(precision) = column.datetime_precision {
        notes.push(format!("datetime precision {precision}"));
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

fn render_ttl_action(action: &TtlAction) -> String {
    match action {
        TtlAction::Delete { predicate: None } => "delete".to_string(),
        TtlAction::Delete {
            predicate: Some(predicate),
        } => format!("delete where {predicate}"),
        TtlAction::Move {
            destination,
            target,
        } => format!(
            "move to {} {target}",
            match destination {
                TtlDestination::Disk => "disk",
                TtlDestination::Volume => "volume",
            }
        ),
        TtlAction::Recompress { codec } => format!("recompress {codec}"),
        TtlAction::GroupBy { keys, assignments } => {
            format!("group by {keys} set {}", assignments.join(", "))
        }
        TtlAction::Unknown { raw } => raw.clone(),
    }
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
