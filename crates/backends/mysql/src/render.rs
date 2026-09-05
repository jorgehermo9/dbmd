use dbmd_core::SourceId;
use dbmd_relational::presentation::{
    self, ColumnView, ConstraintView, FactView, IndexView, NamespaceView, TableDetailsView,
    TableView, TriggerView, ViewPresentation,
};
use dbmd_relational::{ForeignKeyAction, ForeignKeyMatch, IndexSortOrder};
use dbmd_render::{
    code_block, inline_code, object_file_name, text, RenderObject, RenderSource, TemplateFile,
};
use serde::Serialize;

use super::{
    Account, Catalog, Column, Component, Constraint, ConstraintKind, DefaultRole, Index, Library,
    LoadableFunction, Plugin, Privilege, ResourceGroup, RoleGrant, Routine, ServerDefinition,
    SpatialReferenceSystem, Table, Tablespace, Trigger, View, ViewKind,
};

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
    libraries: Vec<RoutineView>,
    servers: Vec<RoutineView>,
    spatial_reference_systems: Vec<RoutineView>,
    tablespaces: Vec<RoutineView>,
    resource_groups: Vec<RoutineView>,
    loadable_functions: Vec<RoutineView>,
    plugins: Vec<RoutineView>,
    components: Vec<RoutineView>,
    accounts: Vec<RoutineView>,
    role_grants: Vec<RoutineView>,
    default_roles: Vec<RoutineView>,
    privileges: Vec<RoutineView>,
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
                        "Default character set {}; collation {}; encryption {}; read-only {}.",
                        inline_code(&schema.default_character_set),
                        inline_code(&schema.default_collation),
                        if schema.default_encryption {
                            "yes"
                        } else {
                            "no"
                        },
                        if schema.read_only { "yes" } else { "no" }
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
                    FactView::new("Definer", inline_code(&event.definer)),
                    FactView::new("Type", event.kind.display_name()),
                    FactView::new("Status", event.status.display_name()),
                    FactView::new("Time zone", inline_code(&event.time_zone)),
                    FactView::new("On completion", event.completion.display_name()),
                    FactView::new("Schedule", inline_code(&event_schedule(event))),
                    FactView::new("SQL mode", inline_code(&event.sql_mode)),
                    FactView::new("Originator server ID", event.originator.to_string()),
                    FactView::new(
                        "Character set client",
                        inline_code(&event.character_set_client),
                    ),
                    FactView::new(
                        "Connection collation",
                        inline_code(&event.collation_connection),
                    ),
                    FactView::new("Database collation", inline_code(&event.database_collation)),
                ],
                definition: Some(code_block("sql", &event.create_statement)),
            })
            .collect(),
        libraries: catalog.libraries.iter().map(render_library).collect(),
        servers: catalog.servers.iter().map(render_server).collect(),
        spatial_reference_systems: catalog
            .spatial_reference_systems
            .iter()
            .map(render_spatial_reference_system)
            .collect(),
        tablespaces: catalog.tablespaces.iter().map(render_tablespace).collect(),
        resource_groups: catalog
            .resource_groups
            .iter()
            .map(render_resource_group)
            .collect(),
        loadable_functions: catalog
            .loadable_functions
            .iter()
            .map(render_loadable_function)
            .collect(),
        plugins: catalog.plugins.iter().map(render_plugin).collect(),
        components: catalog.components.iter().map(render_component).collect(),
        accounts: catalog.accounts.iter().map(render_account).collect(),
        role_grants: catalog.role_grants.iter().map(render_role_grant).collect(),
        default_roles: catalog
            .default_roles
            .iter()
            .map(render_default_role)
            .collect(),
        privileges: catalog.privileges.iter().map(render_privilege).collect(),
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
        .chain(directory_objects("libraries", &data.libraries))
        .chain(directory_objects("servers", &data.servers))
        .chain(directory_objects(
            "spatial-reference-systems",
            &data.spatial_reference_systems,
        ))
        .chain(directory_objects("tablespaces", &data.tablespaces))
        .chain(directory_objects("resource-groups", &data.resource_groups))
        .chain(directory_objects(
            "loadable-functions",
            &data.loadable_functions,
        ))
        .chain(directory_objects("plugins", &data.plugins))
        .chain(directory_objects("components", &data.components))
        .chain(directory_objects("accounts", &data.accounts))
        .chain(directory_objects("role-grants", &data.role_grants))
        .chain(directory_objects("default-roles", &data.default_roles))
        .chain(directory_objects("privileges", &data.privileges))
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

fn directory_objects<'a>(
    directory: &'static str,
    objects: &'a [RoutineView],
) -> impl Iterator<Item = RenderObject> + 'a {
    objects.iter().map(move |value| {
        presentation::directory_object(directory, "function.md.j2", &value.file_name, value)
    })
}

fn render_server(server: &ServerDefinition) -> RoutineView {
    RoutineView {
        qualified_name: inline_code(&server.name),
        file_name: object_file_name("server", &server.name),
        comment: None,
        facts: vec![
            FactView::new("Wrapper", inline_code(&server.wrapper)),
            FactView::new("Host", inline_code(&server.host)),
            FactView::new("Database", inline_code(&server.database)),
            FactView::new("User", inline_code(&server.username)),
            FactView::new("Port", server.port.to_string()),
            FactView::new("Socket", inline_code(&server.socket)),
            FactView::new("Owner", inline_code(&server.owner)),
            FactView::new(
                "Password configured",
                if server.password_configured {
                    "yes"
                } else {
                    "no"
                },
            ),
        ],
        definition: None,
    }
}

fn render_spatial_reference_system(srs: &SpatialReferenceSystem) -> RoutineView {
    let mut facts = vec![FactView::new("ID", srs.id.to_string())];
    if let Some(organization) = &srs.organization {
        facts.push(FactView::new("Organization", inline_code(organization)));
    }
    if let Some(id) = srs.organization_id {
        facts.push(FactView::new("Organization ID", id.to_string()));
    }
    RoutineView {
        qualified_name: inline_code(&format!("{} ({})", srs.name, srs.id)),
        file_name: object_file_name("srs", &srs.id.to_string()),
        comment: srs.description.as_deref().map(text),
        facts,
        definition: Some(code_block("text", &srs.definition)),
    }
}

fn render_tablespace(tablespace: &Tablespace) -> RoutineView {
    let mut facts = vec![
        FactView::new("Engine", inline_code(&tablespace.engine)),
        FactView::new("Space type", inline_code(&tablespace.space_type)),
        FactView::new("File locations", inline_code("<redacted>")),
    ];
    if let Some(value) = &tablespace.row_format {
        facts.push(FactView::new("Row format", inline_code(value)));
    }
    if let Some(value) = tablespace.page_size {
        facts.push(FactView::new("Page size", value.to_string()));
    }
    facts.push(FactView::new(
        "Autoextend size",
        tablespace.autoextend_size.to_string(),
    ));
    if let Some(value) = &tablespace.encryption {
        facts.push(FactView::new("Encryption", inline_code(value)));
    }
    if let Some(value) = &tablespace.engine_attribute {
        facts.push(FactView::new("Engine attribute", inline_code(value)));
    }
    RoutineView {
        qualified_name: inline_code(&tablespace.name),
        file_name: object_file_name("tablespace", &tablespace.name),
        comment: None,
        facts,
        definition: None,
    }
}

fn render_resource_group(group: &ResourceGroup) -> RoutineView {
    RoutineView {
        qualified_name: inline_code(&group.name),
        file_name: object_file_name("resource-group", &group.name),
        comment: None,
        facts: vec![
            FactView::new("Type", group.kind.display_name()),
            FactView::new("Enabled", if group.enabled { "yes" } else { "no" }),
            FactView::new("VCPUs", inline_code(&group.virtual_cpus)),
            FactView::new("Thread priority", group.thread_priority.to_string()),
        ],
        definition: None,
    }
}

fn render_loadable_function(function: &LoadableFunction) -> RoutineView {
    RoutineView {
        qualified_name: inline_code(&function.name),
        file_name: object_file_name("loadable-function", &function.name),
        comment: None,
        facts: vec![
            FactView::new("Kind", inline_code(function.kind.display_name())),
            FactView::new("Returns", inline_code(function.return_type.display_name())),
            FactView::new(
                "Library",
                inline_code(function.library.as_deref().unwrap_or("component/plugin")),
            ),
        ],
        definition: None,
    }
}

fn render_library(library: &Library) -> RoutineView {
    RoutineView {
        qualified_name: inline_code(&format!("{}.{}", library.schema, library.name)),
        file_name: object_file_name(&library.schema, &library.name),
        comment: library.comment.as_deref().map(text),
        facts: vec![
            FactView::new("Language", inline_code(&library.language)),
            FactView::new("Creator", inline_code(&library.creator)),
            FactView::new("SQL mode", inline_code(&library.sql_mode)),
        ],
        definition: Some(code_block(
            if library.language == "JAVASCRIPT" {
                "javascript"
            } else {
                "text"
            },
            &library.definition,
        )),
    }
}

fn render_plugin(plugin: &Plugin) -> RoutineView {
    let mut facts = vec![
        FactView::new("Version", inline_code(&plugin.version)),
        FactView::new("Status", plugin.status.display_name()),
        FactView::new("Type", inline_code(plugin.kind.display_name())),
        FactView::new("Type version", inline_code(&plugin.type_version)),
        FactView::new("License", inline_code(plugin.license.display_name())),
        FactView::new("Load option", plugin.load_option.display_name()),
    ];
    if let Some(library) = &plugin.library {
        facts.push(FactView::new("Library", inline_code(library)));
    }
    if let Some(version) = &plugin.library_version {
        facts.push(FactView::new("Library version", inline_code(version)));
    }
    if let Some(author) = &plugin.author {
        facts.push(FactView::new("Author", text(author)));
    }
    RoutineView {
        qualified_name: inline_code(&plugin.name),
        file_name: object_file_name("plugin", &plugin.name),
        comment: plugin.description.as_deref().map(text),
        facts,
        definition: None,
    }
}

fn render_component(component: &Component) -> RoutineView {
    RoutineView {
        qualified_name: inline_code(&component.urn),
        file_name: object_file_name("component", &component.urn),
        comment: None,
        facts: Vec::new(),
        definition: None,
    }
}

fn render_account(account: &Account) -> RoutineView {
    let identity = format!("{}@{}", account.user, account.host);
    let mut facts = vec![
        FactView::new("Locked", if account.locked { "yes" } else { "no" }),
        FactView::new(
            "Password expired",
            if account.password_expired {
                "yes"
            } else {
                "no"
            },
        ),
        FactView::new("TLS requirement", account.tls_requirement.display_name()),
        FactView::new("Max queries/hour", account.max_queries_per_hour.to_string()),
        FactView::new("Max updates/hour", account.max_updates_per_hour.to_string()),
        FactView::new(
            "Max connections/hour",
            account.max_connections_per_hour.to_string(),
        ),
        FactView::new(
            "Max user connections",
            account.max_user_connections.to_string(),
        ),
        FactView::new(
            "Custom attributes configured",
            if account.attributes_configured {
                "yes"
            } else {
                "no"
            },
        ),
        FactView::new(
            "Dual password configured",
            if account.dual_password_configured {
                "yes"
            } else {
                "no"
            },
        ),
    ];
    for factor in &account.authentication_factors {
        facts.push(FactView::new(
            "Authentication factor",
            format!(
                "{}: plugin={}, credential={}, passwordless={}, registration_required={}",
                factor.position,
                inline_code(&factor.plugin),
                if factor.credential_configured {
                    "configured"
                } else {
                    "not configured"
                },
                factor.passwordless,
                factor.registration_required
            ),
        ));
    }
    if let Some(value) = account.password_lifetime_days {
        facts.push(FactView::new("Password lifetime days", value.to_string()));
    }
    if let Some(value) = account.password_reuse_history {
        facts.push(FactView::new("Password history length", value.to_string()));
    }
    if let Some(value) = account.password_reuse_interval_days {
        facts.push(FactView::new(
            "Password reuse interval days",
            value.to_string(),
        ));
    }
    if let Some(value) = account.require_current_password {
        facts.push(FactView::new(
            "Require current password",
            if value { "yes" } else { "no" },
        ));
    }
    for (label, value) in [
        ("TLS cipher", account.tls_cipher.as_deref()),
        ("X509 issuer", account.x509_issuer.as_deref()),
        ("X509 subject", account.x509_subject.as_deref()),
    ] {
        if let Some(value) = value {
            facts.push(FactView::new(label, inline_code(value)));
        }
    }
    RoutineView {
        qualified_name: inline_code(&identity),
        file_name: object_file_name("account", &identity),
        comment: account.comment.as_deref().map(text),
        facts,
        definition: None,
    }
}

fn render_role_grant(grant: &RoleGrant) -> RoutineView {
    let identity = format!(
        "{}@{} → {}@{}",
        grant.role_user, grant.role_host, grant.member_user, grant.member_host
    );
    RoutineView {
        qualified_name: inline_code(&identity),
        file_name: object_file_name("role-grant", &identity),
        comment: None,
        facts: vec![FactView::new(
            "Admin option",
            if grant.admin_option { "yes" } else { "no" },
        )],
        definition: None,
    }
}

fn render_default_role(role: &DefaultRole) -> RoutineView {
    let identity = format!(
        "{}@{} defaults to {}@{}",
        role.user, role.host, role.role_user, role.role_host
    );
    RoutineView {
        qualified_name: inline_code(&identity),
        file_name: object_file_name("default-role", &identity),
        comment: None,
        facts: Vec::new(),
        definition: None,
    }
}

fn render_privilege(privilege: &Privilege) -> RoutineView {
    let identity = format!(
        "{} {} on {} {}",
        privilege.grantee,
        privilege.privilege,
        privilege.object_kind.display_name(),
        privilege.object_identity
    );
    RoutineView {
        qualified_name: inline_code(&identity),
        file_name: object_file_name("privilege", &identity),
        comment: None,
        facts: vec![FactView::new(
            "Grant option",
            if privilege.grantable { "yes" } else { "no" },
        )],
        definition: None,
    }
}

fn render_table(table: &Table) -> TableView {
    let mut facts = Vec::new();
    for (label, value) in [
        ("Engine", table.engine.as_deref()),
        ("Row format", table.row_format.as_deref()),
        ("Collation", table.collation.as_deref()),
        ("Create options", table.create_options.as_deref()),
        ("Engine attribute", table.engine_attribute.as_deref()),
        (
            "Secondary engine attribute",
            table.secondary_engine_attribute.as_deref(),
        ),
    ] {
        if let Some(value) = value {
            facts.push(FactView::new(label, inline_code(value)));
        }
    }
    for partition in &table.partitions {
        let (label, name, method, expression, ordinal) = if let Some(name) = &partition.subpartition
        {
            (
                "Subpartition",
                name,
                partition.subpartition_method.as_deref(),
                partition.subpartition_expression.as_deref(),
                partition.subpartition_ordinal,
            )
        } else {
            (
                "Partition",
                &partition.name,
                partition.method.as_deref(),
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
    if let Some(srs_id) = column.srs_id {
        notes.push(format!("SRID {srs_id}"));
    }
    if let Some(value) = &column.engine_attribute {
        notes.push(format!("engine attribute {}", inline_code(value)));
    }
    if let Some(value) = &column.secondary_engine_attribute {
        notes.push(format!("secondary engine attribute {}", inline_code(value)));
    }
    if column.masking_policy_configured {
        notes.push("masking policy configured".to_string());
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
        if let Some(match_type) = &value.match_type {
            details.push_str(&format!("; match {}", foreign_key_match_name(match_type)));
        }
        if let Some(action) = value.on_update {
            details.push_str(&format!("; on update {}", foreign_key_action_name(action)));
        }
        if let Some(action) = value.on_delete {
            details.push_str(&format!("; on delete {}", foreign_key_action_name(action)));
        }
    }
    if value.enforced == Some(false) {
        details.push_str("; not enforced");
    }
    if let Some(attribute) = &value.engine_attribute {
        details.push_str(&format!("; engine attribute {}", inline_code(attribute)));
    }
    if let Some(attribute) = &value.secondary_engine_attribute {
        details.push_str(&format!(
            "; secondary engine attribute {}",
            inline_code(attribute)
        ));
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
            let mut value = term
                .column
                .as_deref()
                .or(term.expression.as_deref())
                .unwrap_or("-")
                .to_string();
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
    if value.visible == Some(false) {
        origin.push_str("; invisible");
    }
    if let Some(comment) = &value.comment {
        origin.push_str(&format!("; comment {}", inline_code(comment)));
    }
    if let Some(reason) = &value.disabled_reason {
        origin.push_str(&format!("; disabled reason {}", inline_code(reason)));
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
    let mut facts = vec![
        FactView::new(
            "Kind",
            inline_code(match view.kind {
                ViewKind::Sql => "sql",
                ViewKind::JsonRelationalDuality => "json_relational_duality",
            }),
        ),
        FactView::new("Check option", view.check_option.display_name()),
        FactView::new("Updatable", if view.updatable { "yes" } else { "no" }),
        FactView::new("Security", view.security.display_name()),
        FactView::new("Definer", inline_code(&view.definer)),
    ];
    if let Some(duality) = &view.duality {
        facts.extend([
            FactView::new("JSON column", inline_code(&duality.json_column_name)),
            FactView::new(
                "Root table",
                inline_code(&format!(
                    "{}.{}",
                    duality.root_table_schema, duality.root_table_name
                )),
            ),
            FactView::new("Status", inline_code(duality.status.display_name())),
            FactView::new(
                "Operations",
                inline_code(&format!(
                    "insert={}, update={}, delete={}, read_only={}",
                    duality.allow_insert,
                    duality.allow_update,
                    duality.allow_delete,
                    duality.read_only
                )),
            ),
        ]);
        facts.extend(duality.tables.iter().map(|table| {
            FactView::new(
                "Mapped table",
                inline_code(&format!(
                    "#{} {}.{} parent={:?} relationship={} where={} permissions={}/{}/{} read_only={} root={}",
                    table.id,
                    table.schema,
                    table.name,
                    table.parent_id,
                    table.parent_relationship.as_deref().unwrap_or("-"),
                    table.where_clause.as_deref().unwrap_or("-"),
                    table.allow_insert,
                    table.allow_update,
                    table.allow_delete,
                    table.read_only,
                    table.root
                )),
            )
        }));
        facts.extend(duality.columns.iter().map(|column| {
            FactView::new(
                "JSON field",
                inline_code(&format!(
                    "{} -> #{} {}.{}.{}, permissions={}/{}/{} read_only={} root={}",
                    column.json_key_name,
                    column.table_id,
                    column.table_schema,
                    column.table_name,
                    column.column_name,
                    column.allow_insert,
                    column.allow_update,
                    column.allow_delete,
                    column.read_only,
                    column.root_table
                )),
            )
        }));
        facts.extend(duality.links.iter().map(|link| {
            FactView::new(
                "Table link",
                inline_code(&format!(
                    "{}.{}.{} -> {}.{}.{} join={} json_key={}",
                    link.parent_schema,
                    link.parent_table,
                    link.parent_column,
                    link.child_schema,
                    link.child_table,
                    link.child_column,
                    link.join_type,
                    link.json_key_name.as_deref().unwrap_or("-")
                )),
            )
        }));
    }
    ViewPresentation::builder()
        .qualified_name(inline_code(&view.qualified_name()))
        .file_name(object_file_name(&view.schema, &view.name))
        .comment(None)
        .facts(facts)
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
            inline_code(trigger.timing.display_name()),
            inline_code(trigger.event.display_name())
        ))
        .with_facts(vec![
            FactView::new("Orientation", trigger.orientation.display_name()),
            FactView::new("Order", trigger.action_order.to_string()),
            FactView::new("Definer", inline_code(&trigger.definer)),
            FactView::new("SQL mode", inline_code(&trigger.sql_mode)),
            FactView::new("Character set client", inline_code(&trigger.character_set)),
            FactView::new("Connection collation", inline_code(&trigger.collation)),
            FactView::new(
                "Database collation",
                inline_code(&trigger.database_collation),
            ),
        ])
        .definition(code_block("sql", &trigger.create_statement))
        .build()
}

fn render_routine(routine: &Routine) -> RoutineView {
    let mut facts = vec![
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
                    .map(|parameter| {
                        format!(
                            "{}{} {}",
                            parameter
                                .mode
                                .map(|mode| format!("{} ", mode.display_name()))
                                .unwrap_or_default(),
                            parameter.name.as_deref().unwrap_or("return"),
                            parameter.dtd_identifier
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(", "),
            ),
        ),
    ];
    if let Some(language) = &routine.external_language {
        facts.push(FactView::new("External language", inline_code(language)));
    }
    facts.extend(routine.libraries.iter().map(|library| {
        FactView::new(
            "Imported library",
            inline_code(&format!(
                "{}.{}{}",
                library.schema,
                library.name,
                library
                    .version
                    .as_deref()
                    .map_or_else(String::new, |version| format!("@{version}"))
            )),
        )
    }));
    RoutineView {
        qualified_name: inline_code(&format!("{}.{}", routine.schema, routine.name)),
        file_name: object_file_name(&routine.schema, &routine.name),
        comment: routine.comment.as_deref().map(text),
        facts,
        definition: Some(code_block("sql", &routine.create_statement)),
    }
}

fn event_schedule(event: &super::Event) -> String {
    if let Some(execute_at) = &event.execute_at {
        return format!("AT {execute_at}");
    }
    let mut schedule = format!(
        "EVERY {} {}",
        event.interval_value.as_deref().unwrap_or("?"),
        event.interval_field.as_deref().unwrap_or("?")
    );
    if let Some(starts) = &event.starts {
        schedule.push_str(&format!(" STARTS {starts}"));
    }
    if let Some(ends) = &event.ends {
        schedule.push_str(&format!(" ENDS {ends}"));
    }
    schedule
}
