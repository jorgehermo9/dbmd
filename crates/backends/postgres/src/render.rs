use std::{collections::BTreeMap, fmt::Write as _};

use dbmd_render::{code_block, inline_code, object_file_name, text, RenderSource, TemplateFile};
use serde::Serialize;

use super::catalog::{
    AccessMethod, AccessMethodKind, Aggregate, AggregateFinalModify, AggregateKind, BaseType, Cast,
    CastContext, CastMethod, Catalog, Collation, CollationProvider, Column, ColumnCompression,
    ColumnStorage, CompositeType, Constraint, ConstraintKind, Conversion, Database,
    DatabaseLocaleProvider, DefaultPrivilege, DefaultPrivilegeObject, Domain, EventTrigger,
    EventTriggerEvent, ExtendedStatistics, Extension, ForeignDataWrapper, ForeignServer, Function,
    FunctionKind, FunctionParallel, FunctionVolatility, GeneratedColumnKind, IdentityGeneration,
    Index, IndexNullsOrder, IndexTarget, IndexTerm, Language, LargeObject, ObjectPrivilege,
    Operator, OperatorClass, OperatorFamily, OperatorKind, OperatorPurpose, PolicyCommand,
    PrivilegeKind, PrivilegeObjectKind, Procedure, Publication, PublicationGeneratedColumns,
    RangeType, RelationPersistence, ReplicaIdentity, RewriteRule, RewriteRuleEvent, Role,
    RoleDatabaseSetting, SecurityLabel, SecurityLabelObjectKind, Sequence, SequencePersistence,
    StatisticsKind, Subscription, SubscriptionOrigin, SubscriptionStreaming, SubscriptionTwoPhase,
    SynchronousCommit, Table, TableKind, Tablespace, TextSearchConfiguration, TextSearchDictionary,
    TextSearchParser, TextSearchTemplate, Transform, Trigger, TriggerEnabled, TriggerEvent,
    TriggerOrientation, TriggerTiming, TypeAlignment, TypeStorage, UserMapping, View,
    ViewCheckOption,
};
use dbmd_core::SourceId;
use dbmd_relational::presentation::{
    self, ColumnView as RenderColumn, ConstraintView as RenderConstraint, FactView as RenderFact,
    IndexView as RenderIndex, TableDetailsView as RenderTableDetails, TableView as RenderTable,
    TriggerView as RenderTrigger, ViewPresentation as RenderView,
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
    owner: String,
    comment: Option<String>,
    values: String,
}

#[derive(Serialize)]
struct RenderObject {
    qualified_name: String,
    file_name: String,
    comment: Option<String>,
    facts: Vec<RenderFact>,
    definition: Option<String>,
}

#[derive(Serialize)]
struct RenderNamespace {
    name: String,
    owner: String,
    comment: Option<String>,
}

#[derive(Serialize)]
struct SourceData {
    section_heading: &'static str,
    object_heading: &'static str,
    detail_heading: &'static str,
    database: RenderObject,
    cluster_databases: Vec<RenderObject>,
    tablespaces: Vec<RenderObject>,
    namespaces: Vec<RenderNamespace>,
    enums: Vec<RenderEnum>,
    composite_types: Vec<RenderObject>,
    domains: Vec<RenderObject>,
    base_types: Vec<RenderObject>,
    range_types: Vec<RenderObject>,
    sequences: Vec<RenderObject>,
    tables: Vec<RenderTable>,
    views: Vec<RenderView>,
    triggers: Vec<RenderTrigger>,
    functions: Vec<RenderObject>,
    procedures: Vec<RenderObject>,
    aggregates: Vec<RenderObject>,
    casts: Vec<RenderObject>,
    conversions: Vec<RenderObject>,
    operators: Vec<RenderObject>,
    operator_families: Vec<RenderObject>,
    operator_classes: Vec<RenderObject>,
    access_methods: Vec<RenderObject>,
    languages: Vec<RenderObject>,
    transforms: Vec<RenderObject>,
    rules: Vec<RenderObject>,
    event_triggers: Vec<RenderObject>,
    statistics: Vec<RenderObject>,
    foreign_data_wrappers: Vec<RenderObject>,
    foreign_servers: Vec<RenderObject>,
    user_mappings: Vec<RenderObject>,
    text_search_parsers: Vec<RenderObject>,
    text_search_templates: Vec<RenderObject>,
    text_search_dictionaries: Vec<RenderObject>,
    text_search_configurations: Vec<RenderObject>,
    publications: Vec<RenderObject>,
    subscriptions: Vec<RenderObject>,
    roles: Vec<RenderObject>,
    role_database_settings: Vec<RenderObject>,
    privileges: Vec<RenderObject>,
    default_privileges: Vec<RenderObject>,
    security_labels: Vec<RenderObject>,
    large_objects: Vec<RenderObject>,
    collations: Vec<RenderObject>,
    extensions: Vec<RenderObject>,
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
        database: render_database(&catalog.database),
        cluster_databases: catalog
            .cluster_databases
            .iter()
            .map(render_database)
            .collect(),
        tablespaces: catalog.tablespaces.iter().map(render_tablespace).collect(),
        namespaces: catalog
            .namespaces
            .iter()
            .filter(|namespace| namespace.extension.is_none())
            .map(|namespace| RenderNamespace {
                name: inline_code(&namespace.name),
                owner: inline_code(&namespace.owner),
                comment: namespace.comment.as_deref().map(text),
            })
            .collect(),
        enums: catalog
            .enums
            .iter()
            .filter(|enum_type| enum_type.extension.is_none())
            .map(|enum_type| RenderEnum {
                qualified_name: inline_code(&format!("{}.{}", enum_type.namespace, enum_type.name)),
                file_name: object_file_name(&enum_type.namespace, &enum_type.name),
                owner: inline_code(&enum_type.owner),
                comment: enum_type.comment.as_deref().map(text),
                values: inline_code(&enum_type.values.join(", ")),
            })
            .collect(),
        composite_types: catalog
            .composite_types
            .iter()
            .filter(|composite| composite.extension.is_none())
            .map(render_composite_type)
            .collect(),
        domains: catalog
            .domains
            .iter()
            .filter(|domain| domain.extension.is_none())
            .map(render_domain)
            .collect(),
        base_types: catalog
            .base_types
            .iter()
            .filter(|base_type| base_type.extension.is_none())
            .map(render_base_type)
            .collect(),
        range_types: catalog
            .range_types
            .iter()
            .filter(|range_type| range_type.extension.is_none())
            .map(render_range_type)
            .collect(),
        sequences: catalog
            .sequences
            .iter()
            .filter(|sequence| sequence.extension.is_none())
            .map(render_sequence)
            .collect(),
        tables: catalog
            .tables
            .iter()
            .filter(|table| table.extension.is_none())
            .map(render_table)
            .collect(),
        views: catalog
            .views
            .iter()
            .filter(|view| view.extension.is_none())
            .map(render_view)
            .collect(),
        triggers: catalog
            .triggers
            .iter()
            .filter(|trigger| trigger.extension.is_none())
            .map(render_trigger)
            .collect(),
        functions: catalog
            .functions
            .iter()
            .filter(|function| function.extension.is_none())
            .map(render_function)
            .collect(),
        procedures: catalog
            .procedures
            .iter()
            .filter(|procedure| procedure.extension.is_none())
            .map(render_procedure)
            .collect(),
        aggregates: catalog
            .aggregates
            .iter()
            .filter(|aggregate| aggregate.extension.is_none())
            .map(render_aggregate)
            .collect(),
        casts: catalog
            .casts
            .iter()
            .filter(|object| object.extension.is_none())
            .map(render_cast)
            .collect(),
        conversions: catalog
            .conversions
            .iter()
            .filter(|object| object.extension.is_none())
            .map(render_conversion)
            .collect(),
        operators: catalog
            .operators
            .iter()
            .filter(|object| object.extension.is_none())
            .map(render_operator)
            .collect(),
        operator_families: catalog
            .operator_families
            .iter()
            .filter(|object| object.extension.is_none())
            .map(render_operator_family)
            .collect(),
        operator_classes: catalog
            .operator_classes
            .iter()
            .filter(|object| object.extension.is_none())
            .map(render_operator_class)
            .collect(),
        access_methods: catalog
            .access_methods
            .iter()
            .filter(|object| object.extension.is_none())
            .map(render_access_method)
            .collect(),
        languages: catalog
            .languages
            .iter()
            .filter(|object| object.extension.is_none())
            .map(render_language)
            .collect(),
        transforms: catalog
            .transforms
            .iter()
            .filter(|object| object.extension.is_none())
            .map(render_transform)
            .collect(),
        rules: catalog
            .rules
            .iter()
            .filter(|object| object.extension.is_none())
            .map(render_rule)
            .collect(),
        event_triggers: catalog
            .event_triggers
            .iter()
            .filter(|object| object.extension.is_none())
            .map(render_event_trigger)
            .collect(),
        statistics: catalog
            .statistics
            .iter()
            .filter(|object| object.extension.is_none())
            .map(render_statistics)
            .collect(),
        foreign_data_wrappers: catalog
            .foreign_data_wrappers
            .iter()
            .filter(|object| object.extension.is_none())
            .map(render_foreign_data_wrapper)
            .collect(),
        foreign_servers: catalog
            .foreign_servers
            .iter()
            .filter(|object| object.extension.is_none())
            .map(render_foreign_server)
            .collect(),
        user_mappings: catalog
            .user_mappings
            .iter()
            .map(render_user_mapping)
            .collect(),
        text_search_parsers: catalog
            .text_search_parsers
            .iter()
            .filter(|object| object.extension.is_none())
            .map(render_text_search_parser)
            .collect(),
        text_search_templates: catalog
            .text_search_templates
            .iter()
            .filter(|object| object.extension.is_none())
            .map(render_text_search_template)
            .collect(),
        text_search_dictionaries: catalog
            .text_search_dictionaries
            .iter()
            .filter(|object| object.extension.is_none())
            .map(render_text_search_dictionary)
            .collect(),
        text_search_configurations: catalog
            .text_search_configurations
            .iter()
            .filter(|object| object.extension.is_none())
            .map(render_text_search_configuration)
            .collect(),
        publications: catalog
            .publications
            .iter()
            .map(render_publication)
            .collect(),
        subscriptions: catalog
            .subscriptions
            .iter()
            .map(render_subscription)
            .collect(),
        roles: catalog.roles.iter().map(render_role).collect(),
        role_database_settings: catalog
            .role_database_settings
            .iter()
            .map(render_role_database_setting)
            .collect(),
        privileges: catalog
            .privileges
            .iter()
            .map(render_object_privilege)
            .collect(),
        default_privileges: catalog
            .default_privileges
            .iter()
            .map(render_default_privilege)
            .collect(),
        security_labels: catalog
            .security_labels
            .iter()
            .map(render_security_label)
            .collect(),
        large_objects: catalog
            .large_objects
            .iter()
            .map(render_large_object)
            .collect(),
        collations: catalog
            .collations
            .iter()
            .filter(|collation| collation.extension.is_none())
            .map(render_collation)
            .collect(),
        extensions: catalog.extensions.iter().map(render_extension).collect(),
    };
    let objects = std::iter::once(presentation::directory_object(
        "database",
        "function.md.j2",
        &data.database.file_name,
        &data.database,
    ))
    .chain(data.cluster_databases.iter().map(|object| {
        presentation::directory_object(
            "cluster-databases",
            "function.md.j2",
            &object.file_name,
            object,
        )
    }))
    .chain(data.tablespaces.iter().map(|object| {
        presentation::directory_object("tablespaces", "function.md.j2", &object.file_name, object)
    }))
    .chain(data.enums.iter().map(|object| {
        presentation::directory_object("enums", "enum.md.j2", &object.file_name, object)
    }))
    .chain(data.composite_types.iter().map(|object| {
        presentation::directory_object(
            "composite-types",
            "function.md.j2",
            &object.file_name,
            object,
        )
    }))
    .chain(data.tables.iter().map(|object| {
        presentation::directory_object("tables", "table.md.j2", &object.file_name, object)
    }))
    .chain(data.domains.iter().map(|object| {
        presentation::directory_object("domains", "function.md.j2", &object.file_name, object)
    }))
    .chain(data.base_types.iter().map(|object| {
        presentation::directory_object("base-types", "function.md.j2", &object.file_name, object)
    }))
    .chain(data.range_types.iter().map(|object| {
        presentation::directory_object("range-types", "function.md.j2", &object.file_name, object)
    }))
    .chain(data.sequences.iter().map(|object| {
        presentation::directory_object("sequences", "function.md.j2", &object.file_name, object)
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
    .chain(data.procedures.iter().map(|object| {
        presentation::directory_object("procedures", "function.md.j2", &object.file_name, object)
    }))
    .chain(data.aggregates.iter().map(|object| {
        presentation::directory_object("aggregates", "function.md.j2", &object.file_name, object)
    }))
    .chain(data.casts.iter().map(|object| {
        presentation::directory_object("casts", "function.md.j2", &object.file_name, object)
    }))
    .chain(data.conversions.iter().map(|object| {
        presentation::directory_object("conversions", "function.md.j2", &object.file_name, object)
    }))
    .chain(data.operators.iter().map(|object| {
        presentation::directory_object("operators", "function.md.j2", &object.file_name, object)
    }))
    .chain(data.operator_families.iter().map(|object| {
        presentation::directory_object(
            "operator-families",
            "function.md.j2",
            &object.file_name,
            object,
        )
    }))
    .chain(data.operator_classes.iter().map(|object| {
        presentation::directory_object(
            "operator-classes",
            "function.md.j2",
            &object.file_name,
            object,
        )
    }))
    .chain(data.access_methods.iter().map(|object| {
        presentation::directory_object(
            "access-methods",
            "function.md.j2",
            &object.file_name,
            object,
        )
    }))
    .chain(data.languages.iter().map(|object| {
        presentation::directory_object("languages", "function.md.j2", &object.file_name, object)
    }))
    .chain(data.transforms.iter().map(|object| {
        presentation::directory_object("transforms", "function.md.j2", &object.file_name, object)
    }))
    .chain(data.rules.iter().map(|object| {
        presentation::directory_object("rules", "function.md.j2", &object.file_name, object)
    }))
    .chain(data.event_triggers.iter().map(|object| {
        presentation::directory_object(
            "event-triggers",
            "function.md.j2",
            &object.file_name,
            object,
        )
    }))
    .chain(data.statistics.iter().map(|object| {
        presentation::directory_object("statistics", "function.md.j2", &object.file_name, object)
    }))
    .chain(data.foreign_data_wrappers.iter().map(|object| {
        presentation::directory_object(
            "foreign-data-wrappers",
            "function.md.j2",
            &object.file_name,
            object,
        )
    }))
    .chain(data.foreign_servers.iter().map(|object| {
        presentation::directory_object(
            "foreign-servers",
            "function.md.j2",
            &object.file_name,
            object,
        )
    }))
    .chain(data.user_mappings.iter().map(|object| {
        presentation::directory_object("user-mappings", "function.md.j2", &object.file_name, object)
    }))
    .chain(data.text_search_parsers.iter().map(|object| {
        presentation::directory_object(
            "text-search-parsers",
            "function.md.j2",
            &object.file_name,
            object,
        )
    }))
    .chain(data.text_search_templates.iter().map(|object| {
        presentation::directory_object(
            "text-search-templates",
            "function.md.j2",
            &object.file_name,
            object,
        )
    }))
    .chain(data.text_search_dictionaries.iter().map(|object| {
        presentation::directory_object(
            "text-search-dictionaries",
            "function.md.j2",
            &object.file_name,
            object,
        )
    }))
    .chain(data.text_search_configurations.iter().map(|object| {
        presentation::directory_object(
            "text-search-configurations",
            "function.md.j2",
            &object.file_name,
            object,
        )
    }))
    .chain(data.publications.iter().map(|object| {
        presentation::directory_object("publications", "function.md.j2", &object.file_name, object)
    }))
    .chain(data.subscriptions.iter().map(|object| {
        presentation::directory_object("subscriptions", "function.md.j2", &object.file_name, object)
    }))
    .chain(data.roles.iter().map(|object| {
        presentation::directory_object("roles", "function.md.j2", &object.file_name, object)
    }))
    .chain(data.role_database_settings.iter().map(|object| {
        presentation::directory_object(
            "role-database-settings",
            "function.md.j2",
            &object.file_name,
            object,
        )
    }))
    .chain(data.privileges.iter().map(|object| {
        presentation::directory_object("privileges", "function.md.j2", &object.file_name, object)
    }))
    .chain(data.default_privileges.iter().map(|object| {
        presentation::directory_object(
            "default-privileges",
            "function.md.j2",
            &object.file_name,
            object,
        )
    }))
    .chain(data.security_labels.iter().map(|object| {
        presentation::directory_object(
            "security-labels",
            "function.md.j2",
            &object.file_name,
            object,
        )
    }))
    .chain(data.large_objects.iter().map(|object| {
        presentation::directory_object("large-objects", "function.md.j2", &object.file_name, object)
    }))
    .chain(data.collations.iter().map(|object| {
        presentation::directory_object("collations", "function.md.j2", &object.file_name, object)
    }))
    .chain(data.extensions.iter().map(|object| {
        presentation::directory_object("extensions", "function.md.j2", &object.file_name, object)
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

fn render_database(database: &Database) -> RenderObject {
    let mut facts = vec![
        RenderFact::new("Owner", inline_code(&database.owner)),
        RenderFact::new("Encoding", inline_code(&database.encoding)),
        RenderFact::new(
            "Locale provider",
            inline_code(match database.locale_provider {
                DatabaseLocaleProvider::Builtin => "builtin",
                DatabaseLocaleProvider::Libc => "libc",
                DatabaseLocaleProvider::Icu => "icu",
            }),
        ),
        RenderFact::new("LC_COLLATE", inline_code(&database.lc_collate)),
        RenderFact::new("LC_CTYPE", inline_code(&database.lc_ctype)),
        RenderFact::new("Tablespace", inline_code(&database.tablespace)),
        RenderFact::new("Template", if database.template { "yes" } else { "no" }),
        RenderFact::new(
            "Allows connections",
            if database.allow_connections {
                "yes"
            } else {
                "no"
            },
        ),
        RenderFact::new("Connection limit", database.connection_limit.to_string()),
    ];
    for (label, value) in [
        ("Locale", database.locale.as_deref()),
        ("ICU rules", database.icu_rules.as_deref()),
        ("Collation version", database.collation_version.as_deref()),
    ] {
        if let Some(value) = value {
            facts.push(RenderFact::new(label, inline_code(value)));
        }
    }
    facts.extend(
        database
            .configuration
            .iter()
            .map(|setting| RenderFact::new("Setting", inline_code(setting))),
    );
    RenderObject {
        qualified_name: inline_code(&database.name),
        file_name: object_file_name("database", &database.name),
        comment: database.comment.as_deref().map(text),
        facts,
        definition: None,
    }
}

fn render_tablespace(tablespace: &Tablespace) -> RenderObject {
    let mut facts = vec![
        RenderFact::new("Owner", inline_code(&tablespace.owner)),
        RenderFact::new(
            "Location",
            if tablespace.location_redacted {
                inline_code("<redacted>")
            } else {
                "-".to_string()
            },
        ),
    ];
    facts.extend(
        tablespace
            .options
            .iter()
            .map(|option| RenderFact::new("Option", inline_code(option))),
    );
    RenderObject {
        qualified_name: inline_code(&tablespace.name),
        file_name: object_file_name("tablespace", &tablespace.name),
        comment: tablespace.comment.as_deref().map(text),
        facts,
        definition: None,
    }
}

fn render_collation(collation: &Collation) -> RenderObject {
    let mut facts = vec![
        RenderFact::new("Owner", inline_code(&collation.owner)),
        RenderFact::new(
            "Provider",
            inline_code(match collation.provider {
                CollationProvider::DatabaseDefault => "database_default",
                CollationProvider::Builtin => "builtin",
                CollationProvider::Libc => "libc",
                CollationProvider::Icu => "icu",
            }),
        ),
        RenderFact::new(
            "Deterministic",
            if collation.deterministic { "yes" } else { "no" },
        ),
    ];
    for (label, value) in [
        ("Encoding", collation.encoding.as_deref()),
        ("Locale", collation.locale.as_deref()),
        ("LC_COLLATE", collation.lc_collate.as_deref()),
        ("LC_CTYPE", collation.lc_ctype.as_deref()),
        ("ICU rules", collation.icu_rules.as_deref()),
        ("Version", collation.version.as_deref()),
    ] {
        if let Some(value) = value {
            facts.push(RenderFact::new(label, inline_code(value)));
        }
    }
    RenderObject {
        qualified_name: inline_code(&format!("{}.{}", collation.namespace, collation.name)),
        file_name: object_file_name(&collation.namespace, &collation.name),
        comment: collation.comment.as_deref().map(text),
        facts,
        definition: None,
    }
}

fn render_extension(extension: &Extension) -> RenderObject {
    let mut facts = vec![
        RenderFact::new("Owner", inline_code(&extension.owner)),
        RenderFact::new("Schema", inline_code(&extension.namespace)),
        RenderFact::new("Version", inline_code(&extension.version)),
        RenderFact::new(
            "Relocatable",
            if extension.relocatable { "yes" } else { "no" },
        ),
    ];
    for table in &extension.configuration {
        let mut value = inline_code(&table.relation);
        if let Some(condition) = &table.condition {
            let _ = write!(value, "; where {}", inline_code(condition));
        }
        facts.push(RenderFact::new("Configuration table", value));
    }
    let mut member_counts = BTreeMap::<&str, usize>::new();
    for member in &extension.members {
        *member_counts.entry(&member.object_type).or_default() += 1;
    }
    if !member_counts.is_empty() {
        let kinds = member_counts
            .into_iter()
            .map(|(kind, count)| format!("{}: {count}", inline_code(kind)))
            .collect::<Vec<_>>()
            .join(", ");
        facts.push(RenderFact::new(
            "Owned objects",
            format!("{} ({kinds})", extension.members.len()),
        ));
    }
    RenderObject {
        qualified_name: inline_code(&extension.name),
        file_name: object_file_name("extensions", &extension.name),
        comment: extension.comment.as_deref().map(text),
        facts,
        definition: None,
    }
}

fn render_cast(cast: &Cast) -> RenderObject {
    let mut facts = vec![
        RenderFact::new(
            "Context",
            inline_code(match cast.context {
                CastContext::Explicit => "explicit",
                CastContext::Assignment => "assignment",
                CastContext::Implicit => "implicit",
            }),
        ),
        RenderFact::new(
            "Method",
            inline_code(match cast.method {
                CastMethod::Function => "function",
                CastMethod::InputOutput => "input_output",
                CastMethod::Binary => "binary",
            }),
        ),
    ];
    if let Some(function) = &cast.function {
        facts.push(RenderFact::new("Function", inline_code(function)));
    }
    let identity = format!("{} AS {}", cast.source_type, cast.target_type);
    RenderObject {
        qualified_name: inline_code(&identity),
        file_name: object_file_name("casts", &identity),
        comment: cast.comment.as_deref().map(text),
        facts,
        definition: None,
    }
}

fn render_conversion(conversion: &Conversion) -> RenderObject {
    RenderObject {
        qualified_name: inline_code(&format!("{}.{}", conversion.namespace, conversion.name)),
        file_name: object_file_name(&conversion.namespace, &conversion.name),
        comment: conversion.comment.as_deref().map(text),
        facts: vec![
            RenderFact::new("Owner", inline_code(&conversion.owner)),
            RenderFact::new("Source encoding", inline_code(&conversion.source_encoding)),
            RenderFact::new("Target encoding", inline_code(&conversion.target_encoding)),
            RenderFact::new("Function", inline_code(&conversion.function)),
            RenderFact::new("Default", if conversion.default { "yes" } else { "no" }),
        ],
        definition: None,
    }
}

fn render_operator(operator: &Operator) -> RenderObject {
    let signature = match &operator.left_type {
        Some(left) => format!("{left}, {}", operator.right_type),
        None => format!("NONE, {}", operator.right_type),
    };
    let mut facts = vec![
        RenderFact::new("Owner", inline_code(&operator.owner)),
        RenderFact::new(
            "Kind",
            inline_code(match operator.kind {
                OperatorKind::Binary => "binary",
                OperatorKind::Prefix => "prefix",
            }),
        ),
        RenderFact::new("Result", inline_code(&operator.result_type)),
        RenderFact::new("Function", inline_code(&operator.function)),
        RenderFact::new("Merge join", if operator.can_merge { "yes" } else { "no" }),
        RenderFact::new("Hash join", if operator.can_hash { "yes" } else { "no" }),
    ];
    for (label, value) in [
        ("Commutator", operator.commutator.as_deref()),
        ("Negator", operator.negator.as_deref()),
        (
            "Restriction estimator",
            operator.restriction_selectivity.as_deref(),
        ),
        ("Join estimator", operator.join_selectivity.as_deref()),
    ] {
        if let Some(value) = value {
            facts.push(RenderFact::new(label, inline_code(value)));
        }
    }
    RenderObject {
        qualified_name: inline_code(&format!(
            "{}.{}({signature})",
            operator.namespace, operator.name
        )),
        file_name: object_file_name(
            &operator.namespace,
            &format!("{}({signature})", operator.name),
        ),
        comment: operator.comment.as_deref().map(text),
        facts,
        definition: None,
    }
}

fn render_operator_family(family: &OperatorFamily) -> RenderObject {
    let mut facts = vec![
        RenderFact::new("Owner", inline_code(&family.owner)),
        RenderFact::new("Access method", inline_code(&family.access_method)),
    ];
    for operator in &family.operators {
        let mut value = format!(
            "strategy {}; {} ({}, {}) via {}",
            operator.strategy,
            inline_code(&operator.operator),
            inline_code(&operator.left_type),
            inline_code(&operator.right_type),
            inline_code(&operator.access_method)
        );
        if operator.purpose == OperatorPurpose::Ordering {
            value.push_str("; ordering");
        }
        if let Some(sort_family) = &operator.sort_family {
            let _ = write!(value, "; sort family {}", inline_code(sort_family));
        }
        facts.push(RenderFact::new("Operator", value));
    }
    for function in &family.functions {
        facts.push(RenderFact::new(
            "Support function",
            format!(
                "number {}; {} ({}, {})",
                function.number,
                inline_code(&function.function),
                inline_code(&function.left_type),
                inline_code(&function.right_type)
            ),
        ));
    }
    RenderObject {
        qualified_name: inline_code(&format!("{}.{}", family.namespace, family.name)),
        file_name: object_file_name(
            &family.namespace,
            &format!("{}.{}", family.access_method, family.name),
        ),
        comment: family.comment.as_deref().map(text),
        facts,
        definition: None,
    }
}

fn render_operator_class(class: &OperatorClass) -> RenderObject {
    let mut facts = vec![
        RenderFact::new("Owner", inline_code(&class.owner)),
        RenderFact::new("Access method", inline_code(&class.access_method)),
        RenderFact::new("Family", inline_code(&class.family)),
        RenderFact::new("Input type", inline_code(&class.input_type)),
        RenderFact::new("Default", if class.default { "yes" } else { "no" }),
    ];
    if let Some(key_type) = &class.key_type {
        facts.push(RenderFact::new("Key type", inline_code(key_type)));
    }
    RenderObject {
        qualified_name: inline_code(&format!("{}.{}", class.namespace, class.name)),
        file_name: object_file_name(
            &class.namespace,
            &format!("{}.{}", class.access_method, class.name),
        ),
        comment: class.comment.as_deref().map(text),
        facts,
        definition: None,
    }
}

fn render_access_method(access_method: &AccessMethod) -> RenderObject {
    RenderObject {
        qualified_name: inline_code(&access_method.name),
        file_name: object_file_name("access-methods", &access_method.name),
        comment: access_method.comment.as_deref().map(text),
        facts: vec![
            RenderFact::new(
                "Kind",
                inline_code(match access_method.kind {
                    AccessMethodKind::Table => "table",
                    AccessMethodKind::Index => "index",
                }),
            ),
            RenderFact::new("Handler", inline_code(&access_method.handler)),
        ],
        definition: None,
    }
}

fn render_language(language: &Language) -> RenderObject {
    let mut facts = vec![
        RenderFact::new("Owner", inline_code(&language.owner)),
        RenderFact::new("Procedural", if language.procedural { "yes" } else { "no" }),
        RenderFact::new("Trusted", if language.trusted { "yes" } else { "no" }),
    ];
    for (label, value) in [
        ("Handler", language.handler.as_deref()),
        ("Inline handler", language.inline_handler.as_deref()),
        ("Validator", language.validator.as_deref()),
    ] {
        if let Some(value) = value {
            facts.push(RenderFact::new(label, inline_code(value)));
        }
    }
    RenderObject {
        qualified_name: inline_code(&language.name),
        file_name: object_file_name("languages", &language.name),
        comment: language.comment.as_deref().map(text),
        facts,
        definition: None,
    }
}

fn render_transform(transform: &Transform) -> RenderObject {
    let mut facts = vec![RenderFact::new(
        "Language",
        inline_code(&transform.language),
    )];
    if let Some(function) = &transform.from_sql {
        facts.push(RenderFact::new("From SQL", inline_code(function)));
    }
    if let Some(function) = &transform.to_sql {
        facts.push(RenderFact::new("To SQL", inline_code(function)));
    }
    let identity = format!("{} FOR {}", transform.data_type, transform.language);
    RenderObject {
        qualified_name: inline_code(&identity),
        file_name: object_file_name("transforms", &identity),
        comment: transform.comment.as_deref().map(text),
        facts,
        definition: None,
    }
}

fn render_rule(rule: &RewriteRule) -> RenderObject {
    RenderObject {
        qualified_name: inline_code(&format!("{}.{}.{}", rule.namespace, rule.target, rule.name)),
        file_name: object_file_name(&rule.namespace, &format!("{}.{}", rule.target, rule.name)),
        comment: rule.comment.as_deref().map(text),
        facts: vec![
            RenderFact::new(
                "Event",
                inline_code(match rule.event {
                    RewriteRuleEvent::Select => "select",
                    RewriteRuleEvent::Update => "update",
                    RewriteRuleEvent::Insert => "insert",
                    RewriteRuleEvent::Delete => "delete",
                }),
            ),
            RenderFact::new("Instead", if rule.instead { "yes" } else { "no" }),
            RenderFact::new("Enabled", inline_code(trigger_enabled_name(rule.enabled))),
        ],
        definition: Some(code_block("sql", &rule.definition)),
    }
}

fn render_event_trigger(trigger: &EventTrigger) -> RenderObject {
    let mut facts = vec![
        RenderFact::new("Owner", inline_code(&trigger.owner)),
        RenderFact::new(
            "Event",
            inline_code(match trigger.event {
                EventTriggerEvent::Login => "login",
                EventTriggerEvent::DdlCommandStart => "DDL command start",
                EventTriggerEvent::DdlCommandEnd => "DDL command end",
                EventTriggerEvent::SqlDrop => "SQL drop",
                EventTriggerEvent::TableRewrite => "table rewrite",
            }),
        ),
        RenderFact::new("Function", inline_code(&trigger.function)),
        RenderFact::new(
            "Enabled",
            inline_code(trigger_enabled_name(trigger.enabled)),
        ),
    ];
    if !trigger.tags.is_empty() {
        facts.push(RenderFact::new(
            "Tags",
            inline_code(&trigger.tags.join(", ")),
        ));
    }
    RenderObject {
        qualified_name: inline_code(&trigger.name),
        file_name: object_file_name("event-trigger", &trigger.name),
        comment: trigger.comment.as_deref().map(text),
        facts,
        definition: Some(code_block("sql", &trigger.definition)),
    }
}

fn render_statistics(statistics: &ExtendedStatistics) -> RenderObject {
    let kinds = statistics
        .kinds
        .iter()
        .map(|kind| match kind {
            StatisticsKind::NdDistinct => "ndistinct",
            StatisticsKind::Dependencies => "dependencies",
            StatisticsKind::MostCommonValues => "mcv",
            StatisticsKind::Expressions => "expressions",
        })
        .collect::<Vec<_>>()
        .join(", ");
    let mut facts = vec![
        RenderFact::new("Owner", inline_code(&statistics.owner)),
        RenderFact::new("Kinds", inline_code(&kinds)),
        RenderFact::new("Statistics target", statistics.target.to_string()),
    ];
    if !statistics.columns.is_empty() {
        facts.push(RenderFact::new(
            "Columns",
            inline_code(&statistics.columns.join(", ")),
        ));
    }
    for expression in &statistics.expressions {
        facts.push(RenderFact::new("Expression", inline_code(expression)));
    }
    RenderObject {
        qualified_name: inline_code(&format!("{}.{}", statistics.namespace, statistics.name)),
        file_name: object_file_name(&statistics.namespace, &statistics.name),
        comment: statistics.comment.as_deref().map(text),
        facts,
        definition: Some(code_block("sql", &statistics.definition)),
    }
}

fn render_foreign_data_wrapper(wrapper: &ForeignDataWrapper) -> RenderObject {
    let mut facts = vec![RenderFact::new("Owner", inline_code(&wrapper.owner))];
    for (label, value) in [
        ("Handler", wrapper.handler.as_deref()),
        ("Validator", wrapper.validator.as_deref()),
    ] {
        if let Some(value) = value {
            facts.push(RenderFact::new(label, inline_code(value)));
        }
    }
    facts.extend(
        wrapper
            .options
            .iter()
            .map(|option| RenderFact::new("Option", inline_code(option))),
    );
    RenderObject {
        qualified_name: inline_code(&wrapper.name),
        file_name: object_file_name("foreign-data-wrappers", &wrapper.name),
        comment: wrapper.comment.as_deref().map(text),
        facts,
        definition: None,
    }
}

fn render_foreign_server(server: &ForeignServer) -> RenderObject {
    let mut facts = vec![
        RenderFact::new("Owner", inline_code(&server.owner)),
        RenderFact::new("Foreign-data wrapper", inline_code(&server.wrapper)),
    ];
    for (label, value) in [
        ("Type", server.server_type.as_deref()),
        ("Version", server.version.as_deref()),
    ] {
        if let Some(value) = value {
            facts.push(RenderFact::new(label, inline_code(value)));
        }
    }
    facts.extend(
        server
            .options
            .iter()
            .map(|option| RenderFact::new("Option", inline_code(option))),
    );
    RenderObject {
        qualified_name: inline_code(&server.name),
        file_name: object_file_name("foreign-servers", &server.name),
        comment: server.comment.as_deref().map(text),
        facts,
        definition: None,
    }
}

fn render_user_mapping(mapping: &UserMapping) -> RenderObject {
    let identity = format!("{} ON {}", mapping.user, mapping.server);
    RenderObject {
        qualified_name: inline_code(&identity),
        file_name: object_file_name(&mapping.server, &mapping.user),
        comment: None,
        facts: mapping
            .options
            .iter()
            .map(|option| RenderFact::new("Option", inline_code(option)))
            .collect(),
        definition: None,
    }
}

fn render_text_search_parser(parser: &TextSearchParser) -> RenderObject {
    RenderObject {
        qualified_name: inline_code(&format!("{}.{}", parser.namespace, parser.name)),
        file_name: object_file_name(&parser.namespace, &parser.name),
        comment: parser.comment.as_deref().map(text),
        facts: vec![
            RenderFact::new("Start function", inline_code(&parser.start_function)),
            RenderFact::new("Token function", inline_code(&parser.token_function)),
            RenderFact::new("End function", inline_code(&parser.end_function)),
            RenderFact::new("Headline function", inline_code(&parser.headline_function)),
            RenderFact::new(
                "Token-types function",
                inline_code(&parser.token_types_function),
            ),
        ],
        definition: None,
    }
}

fn render_text_search_template(template: &TextSearchTemplate) -> RenderObject {
    let mut facts = Vec::new();
    if let Some(function) = &template.init_function {
        facts.push(RenderFact::new("Init function", inline_code(function)));
    }
    facts.push(RenderFact::new(
        "Lexize function",
        inline_code(&template.lexize_function),
    ));
    RenderObject {
        qualified_name: inline_code(&format!("{}.{}", template.namespace, template.name)),
        file_name: object_file_name(&template.namespace, &template.name),
        comment: template.comment.as_deref().map(text),
        facts,
        definition: None,
    }
}

fn render_text_search_dictionary(dictionary: &TextSearchDictionary) -> RenderObject {
    let mut facts = vec![
        RenderFact::new("Owner", inline_code(&dictionary.owner)),
        RenderFact::new("Template", inline_code(&dictionary.template)),
    ];
    if let Some(options) = &dictionary.options {
        facts.push(RenderFact::new("Options", inline_code(options)));
    }
    RenderObject {
        qualified_name: inline_code(&format!("{}.{}", dictionary.namespace, dictionary.name)),
        file_name: object_file_name(&dictionary.namespace, &dictionary.name),
        comment: dictionary.comment.as_deref().map(text),
        facts,
        definition: None,
    }
}

fn render_text_search_configuration(configuration: &TextSearchConfiguration) -> RenderObject {
    let mut facts = vec![
        RenderFact::new("Owner", inline_code(&configuration.owner)),
        RenderFact::new("Parser", inline_code(&configuration.parser)),
    ];
    for mapping in &configuration.mappings {
        facts.push(RenderFact::new(
            "Mapping",
            format!(
                "{}: {}",
                inline_code(&mapping.token_type),
                inline_code(&mapping.dictionaries.join(", "))
            ),
        ));
    }
    RenderObject {
        qualified_name: inline_code(&format!(
            "{}.{}",
            configuration.namespace, configuration.name
        )),
        file_name: object_file_name(&configuration.namespace, &configuration.name),
        comment: configuration.comment.as_deref().map(text),
        facts,
        definition: None,
    }
}

const fn trigger_enabled_name(enabled: TriggerEnabled) -> &'static str {
    match enabled {
        TriggerEnabled::Origin => "origin",
        TriggerEnabled::Disabled => "disabled",
        TriggerEnabled::Replica => "replica",
        TriggerEnabled::Always => "always",
    }
}

fn render_publication(publication: &Publication) -> RenderObject {
    let mut actions = Vec::new();
    if publication.publish_insert {
        actions.push("insert");
    }
    if publication.publish_update {
        actions.push("update");
    }
    if publication.publish_delete {
        actions.push("delete");
    }
    if publication.publish_truncate {
        actions.push("truncate");
    }
    let mut facts = vec![
        RenderFact::new("Owner", inline_code(&publication.owner)),
        RenderFact::new(
            "All tables",
            if publication.all_tables { "yes" } else { "no" },
        ),
        RenderFact::new("Actions", inline_code(&actions.join(", "))),
        RenderFact::new(
            "Generated columns",
            inline_code(match publication.generated_columns {
                PublicationGeneratedColumns::None => "none",
                PublicationGeneratedColumns::Stored => "stored",
            }),
        ),
        RenderFact::new(
            "Publish via partition root",
            if publication.publish_via_partition_root {
                "yes"
            } else {
                "no"
            },
        ),
    ];
    for schema in &publication.schemas {
        facts.push(RenderFact::new("Schema", inline_code(schema)));
    }
    for table in &publication.tables {
        let mut value = inline_code(&format!("{}.{}", table.namespace, table.name));
        if let Some(columns) = &table.columns {
            let _ = write!(value, "; columns {}", inline_code(&columns.join(", ")));
        }
        if let Some(filter) = &table.row_filter {
            let _ = write!(value, "; where {}", inline_code(filter));
        }
        facts.push(RenderFact::new("Table", value));
    }
    RenderObject {
        qualified_name: inline_code(&publication.name),
        file_name: object_file_name("publications", &publication.name),
        comment: publication.comment.as_deref().map(text),
        facts,
        definition: None,
    }
}

fn render_subscription(subscription: &Subscription) -> RenderObject {
    let mut facts = vec![
        RenderFact::new("Owner", inline_code(&subscription.owner)),
        RenderFact::new("Enabled", if subscription.enabled { "yes" } else { "no" }),
        RenderFact::new("Binary", if subscription.binary { "yes" } else { "no" }),
        RenderFact::new(
            "Streaming",
            inline_code(match subscription.streaming {
                SubscriptionStreaming::Off => "off",
                SubscriptionStreaming::On => "on",
                SubscriptionStreaming::Parallel => "parallel",
            }),
        ),
        RenderFact::new(
            "Two phase",
            inline_code(match subscription.two_phase {
                SubscriptionTwoPhase::Disabled => "disabled",
                SubscriptionTwoPhase::Pending => "pending",
                SubscriptionTwoPhase::Enabled => "enabled",
            }),
        ),
        RenderFact::new(
            "Disable on error",
            if subscription.disable_on_error {
                "yes"
            } else {
                "no"
            },
        ),
        RenderFact::new(
            "Password required",
            if subscription.password_required {
                "yes"
            } else {
                "no"
            },
        ),
        RenderFact::new(
            "Run as owner",
            if subscription.run_as_owner {
                "yes"
            } else {
                "no"
            },
        ),
        RenderFact::new("Failover", if subscription.failover { "yes" } else { "no" }),
        RenderFact::new(
            "Synchronous commit",
            inline_code(match subscription.synchronous_commit {
                SynchronousCommit::Off => "off",
                SynchronousCommit::Local => "local",
                SynchronousCommit::RemoteWrite => "remote write",
                SynchronousCommit::On => "on",
                SynchronousCommit::RemoteApply => "remote apply",
            }),
        ),
        RenderFact::new(
            "Publications",
            inline_code(&subscription.publications.join(", ")),
        ),
        RenderFact::new(
            "Origin",
            inline_code(match subscription.origin {
                SubscriptionOrigin::None => "no origin",
                SubscriptionOrigin::Any => "any origin",
            }),
        ),
        RenderFact::new("Connection", inline_code("<redacted>")),
    ];
    if let Some(slot_name) = &subscription.slot_name {
        facts.push(RenderFact::new("Slot", inline_code(slot_name)));
    }
    if let Some(skip_lsn) = &subscription.skip_lsn {
        facts.push(RenderFact::new("Skip LSN", inline_code(skip_lsn)));
    }
    RenderObject {
        qualified_name: inline_code(&subscription.name),
        file_name: object_file_name("subscription", &subscription.name),
        comment: subscription.comment.as_deref().map(text),
        facts,
        definition: None,
    }
}

fn render_role(role: &Role) -> RenderObject {
    let mut facts = vec![
        RenderFact::new("Login", if role.login { "yes" } else { "no" }),
        RenderFact::new("Superuser", if role.superuser { "yes" } else { "no" }),
        RenderFact::new("Inherit", if role.inherit { "yes" } else { "no" }),
        RenderFact::new("Create role", if role.create_role { "yes" } else { "no" }),
        RenderFact::new(
            "Create database",
            if role.create_database { "yes" } else { "no" },
        ),
        RenderFact::new("Replication", if role.replication { "yes" } else { "no" }),
        RenderFact::new(
            "Bypass row-level security",
            if role.bypass_row_level_security {
                "yes"
            } else {
                "no"
            },
        ),
        RenderFact::new("Connection limit", role.connection_limit.to_string()),
        RenderFact::new(
            "Password configured",
            if role.password_configured {
                "yes"
            } else {
                "no"
            },
        ),
    ];
    if let Some(valid_until) = &role.valid_until {
        facts.push(RenderFact::new("Valid until", inline_code(valid_until)));
    }
    facts.extend(
        role.configuration
            .iter()
            .map(|setting| RenderFact::new("Configuration", inline_code(setting))),
    );
    for membership in &role.memberships {
        facts.push(RenderFact::new(
            "Member of",
            format!(
                "{}; grantor {}; admin {}; inherit {}; set {}",
                inline_code(&membership.role),
                inline_code(&membership.grantor),
                if membership.admin { "yes" } else { "no" },
                if membership.inherit { "yes" } else { "no" },
                if membership.set { "yes" } else { "no" },
            ),
        ));
    }
    RenderObject {
        qualified_name: inline_code(&role.name),
        file_name: object_file_name("role", &role.name),
        comment: role.comment.as_deref().map(text),
        facts,
        definition: None,
    }
}

fn render_role_database_setting(setting: &RoleDatabaseSetting) -> RenderObject {
    let identity = format!("{} in {}", setting.role, setting.database);
    RenderObject {
        qualified_name: inline_code(&identity),
        file_name: object_file_name(&setting.database, &setting.role),
        comment: None,
        facts: setting
            .settings
            .iter()
            .map(|value| RenderFact::new("Setting", inline_code(value)))
            .collect(),
        definition: None,
    }
}

fn render_object_privilege(privilege: &ObjectPrivilege) -> RenderObject {
    let object_kind = privilege_object_kind_name(privilege.object_kind);
    let privilege_kind = privilege_kind_name(privilege.privilege);
    let name = format!(
        "{} {} → {} {}",
        object_kind, privilege.object_identity, privilege.grantee, privilege_kind
    );
    let mut facts = vec![
        RenderFact::new("Object type", inline_code(object_kind)),
        RenderFact::new("Object", inline_code(&privilege.object_identity)),
        RenderFact::new("Grantor", inline_code(&privilege.grantor)),
        RenderFact::new("Grantee", inline_code(&privilege.grantee)),
        RenderFact::new("Privilege", inline_code(privilege_kind)),
    ];
    if privilege.grantable {
        facts.push(RenderFact::new("Grant option", "yes"));
    }
    RenderObject {
        qualified_name: inline_code(&name),
        file_name: object_file_name(
            object_kind,
            &format!(
                "{}-{}-{}-{}",
                privilege.object_identity, privilege.grantor, privilege.grantee, privilege_kind
            ),
        ),
        comment: None,
        facts,
        definition: None,
    }
}

const fn privilege_object_kind_name(kind: PrivilegeObjectKind) -> &'static str {
    match kind {
        PrivilegeObjectKind::Database => "database",
        PrivilegeObjectKind::Schema => "schema",
        PrivilegeObjectKind::Table => "table",
        PrivilegeObjectKind::TableColumn => "table column",
        PrivilegeObjectKind::Sequence => "sequence",
        PrivilegeObjectKind::View => "view",
        PrivilegeObjectKind::MaterializedView => "materialized view",
        PrivilegeObjectKind::ForeignTable => "foreign table",
        PrivilegeObjectKind::Function => "function",
        PrivilegeObjectKind::Procedure => "procedure",
        PrivilegeObjectKind::Aggregate => "aggregate",
        PrivilegeObjectKind::Type => "type",
        PrivilegeObjectKind::Domain => "domain",
        PrivilegeObjectKind::Language => "language",
        PrivilegeObjectKind::LargeObject => "large object",
        PrivilegeObjectKind::ForeignDataWrapper => "foreign-data wrapper",
        PrivilegeObjectKind::ForeignServer => "foreign server",
        PrivilegeObjectKind::Parameter => "parameter",
        PrivilegeObjectKind::Tablespace => "tablespace",
    }
}

const fn privilege_kind_name(kind: PrivilegeKind) -> &'static str {
    match kind {
        PrivilegeKind::Select => "SELECT",
        PrivilegeKind::Insert => "INSERT",
        PrivilegeKind::Update => "UPDATE",
        PrivilegeKind::Delete => "DELETE",
        PrivilegeKind::Truncate => "TRUNCATE",
        PrivilegeKind::References => "REFERENCES",
        PrivilegeKind::Trigger => "TRIGGER",
        PrivilegeKind::Maintain => "MAINTAIN",
        PrivilegeKind::Usage => "USAGE",
        PrivilegeKind::Create => "CREATE",
        PrivilegeKind::Connect => "CONNECT",
        PrivilegeKind::Temporary => "TEMPORARY",
        PrivilegeKind::Execute => "EXECUTE",
        PrivilegeKind::Set => "SET",
        PrivilegeKind::AlterSystem => "ALTER SYSTEM",
    }
}

fn render_default_privilege(privilege: &DefaultPrivilege) -> RenderObject {
    let object_kind = match privilege.object_kind {
        DefaultPrivilegeObject::Tables => "tables",
        DefaultPrivilegeObject::Sequences => "sequences",
        DefaultPrivilegeObject::Routines => "routines",
        DefaultPrivilegeObject::Types => "types",
        DefaultPrivilegeObject::Schemas => "schemas",
        DefaultPrivilegeObject::LargeObjects => "large objects",
    };
    let privilege_kind = privilege_kind_name(privilege.privilege);
    let scope = privilege.namespace.as_deref().unwrap_or("database-wide");
    let name = format!(
        "{} / {} / {} → {} {}",
        privilege.owner, scope, object_kind, privilege.grantee, privilege_kind
    );
    let mut facts = vec![
        RenderFact::new("Owner", inline_code(&privilege.owner)),
        RenderFact::new("Scope", inline_code(scope)),
        RenderFact::new("Object family", inline_code(object_kind)),
        RenderFact::new("Grantor", inline_code(&privilege.grantor)),
        RenderFact::new("Grantee", inline_code(&privilege.grantee)),
        RenderFact::new("Privilege", inline_code(privilege_kind)),
    ];
    if privilege.grantable {
        facts.push(RenderFact::new("Grant option", "yes"));
    }
    RenderObject {
        qualified_name: inline_code(&name),
        file_name: object_file_name(
            "default-privilege",
            &format!(
                "{}-{}-{}-{}-{}",
                privilege.owner, scope, object_kind, privilege.grantee, privilege_kind
            ),
        ),
        comment: None,
        facts,
        definition: None,
    }
}

fn render_security_label(label: &SecurityLabel) -> RenderObject {
    let object_kind = security_label_object_kind_name(label.object_kind);
    let name = format!(
        "{} {} / {}",
        object_kind, label.object_identity, label.provider
    );
    RenderObject {
        qualified_name: inline_code(&name),
        file_name: object_file_name(
            object_kind,
            &format!("{}-{}", label.object_identity, label.provider),
        ),
        comment: None,
        facts: vec![
            RenderFact::new("Object type", inline_code(object_kind)),
            RenderFact::new("Object", inline_code(&label.object_identity)),
            RenderFact::new("Provider", inline_code(&label.provider)),
            RenderFact::new("Label", inline_code(&label.label)),
        ],
        definition: None,
    }
}

const fn security_label_object_kind_name(kind: SecurityLabelObjectKind) -> &'static str {
    match kind {
        SecurityLabelObjectKind::Aggregate => "aggregate",
        SecurityLabelObjectKind::Database => "database",
        SecurityLabelObjectKind::Domain => "domain",
        SecurityLabelObjectKind::EventTrigger => "event trigger",
        SecurityLabelObjectKind::ForeignTable => "foreign table",
        SecurityLabelObjectKind::Function => "function",
        SecurityLabelObjectKind::LargeObject => "large object",
        SecurityLabelObjectKind::MaterializedView => "materialized view",
        SecurityLabelObjectKind::Procedure => "procedure",
        SecurityLabelObjectKind::Publication => "publication",
        SecurityLabelObjectKind::Role => "role",
        SecurityLabelObjectKind::Schema => "schema",
        SecurityLabelObjectKind::Sequence => "sequence",
        SecurityLabelObjectKind::Subscription => "subscription",
        SecurityLabelObjectKind::Table => "table",
        SecurityLabelObjectKind::TableColumn => "table column",
        SecurityLabelObjectKind::Tablespace => "tablespace",
        SecurityLabelObjectKind::Type => "type",
        SecurityLabelObjectKind::View => "view",
    }
}

fn render_large_object(large_object: &LargeObject) -> RenderObject {
    RenderObject {
        qualified_name: inline_code(&large_object.oid.to_string()),
        file_name: object_file_name("large-object", &large_object.oid.to_string()),
        comment: large_object.comment.as_deref().map(text),
        facts: vec![
            RenderFact::new("Owner", inline_code(&large_object.owner)),
            RenderFact::new(
                "Contents",
                inline_code(if large_object.contents_omitted {
                    "omitted"
                } else {
                    "included"
                }),
            ),
        ],
        definition: None,
    }
}

fn render_composite_type(composite: &CompositeType) -> RenderObject {
    let mut facts = vec![RenderFact::new("Owner", inline_code(&composite.owner))];
    for attribute in &composite.attributes {
        let mut value = format!(
            "{} {}",
            inline_code(&attribute.name),
            inline_code(&attribute.data_type)
        );
        if let Some(collation) = &attribute.collation {
            let _ = write!(value, "; collation {}", inline_code(collation));
        }
        if let Some(comment) = &attribute.comment {
            let _ = write!(value, "; {}", text(comment));
        }
        facts.push(RenderFact::new("Attribute", value));
    }
    RenderObject {
        qualified_name: inline_code(&format!("{}.{}", composite.namespace, composite.name)),
        file_name: object_file_name(&composite.namespace, &composite.name),
        comment: composite.comment.as_deref().map(text),
        facts,
        definition: Some(code_block("sql", &composite.definition)),
    }
}

fn render_domain(domain: &Domain) -> RenderObject {
    let mut facts = vec![
        RenderFact::new("Base type", inline_code(&domain.base_type)),
        RenderFact::new("Nullable", if domain.not_null { "no" } else { "yes" }),
        RenderFact::new("Owner", inline_code(&domain.owner)),
    ];
    if let Some(collation) = &domain.collation {
        facts.push(RenderFact::new("Collation", inline_code(collation)));
    }
    if let Some(default) = &domain.default {
        facts.push(RenderFact::new("Default", inline_code(default)));
    }
    for constraint in &domain.constraints {
        let comment = constraint
            .comment
            .as_deref()
            .map(|comment| format!("; {}", text(comment)))
            .unwrap_or_default();
        facts.push(RenderFact::new(
            "Constraint",
            format!(
                "{}: {}{}{}",
                inline_code(&constraint.name),
                inline_code(&constraint.definition),
                if constraint.validated {
                    ""
                } else {
                    "; not validated"
                },
                comment,
            ),
        ));
    }
    RenderObject {
        qualified_name: inline_code(&format!("{}.{}", domain.namespace, domain.name)),
        file_name: object_file_name(&domain.namespace, &domain.name),
        comment: domain.comment.as_deref().map(text),
        facts,
        definition: Some(code_block("sql", &domain.definition)),
    }
}

fn render_base_type(base_type: &BaseType) -> RenderObject {
    let mut facts = vec![
        RenderFact::new(
            "Kind",
            inline_code(if base_type.defined { "base" } else { "shell" }),
        ),
        RenderFact::new("Owner", inline_code(&base_type.owner)),
    ];
    if let Some(details) = &base_type.details {
        facts.extend([
            RenderFact::new("Input", inline_code(&details.input)),
            RenderFact::new("Output", inline_code(&details.output)),
            RenderFact::new("Internal length", details.internal_length.to_string()),
            RenderFact::new(
                "Passed by value",
                if details.passed_by_value { "yes" } else { "no" },
            ),
            RenderFact::new("Category", inline_code(&details.category)),
            RenderFact::new("Preferred", if details.preferred { "yes" } else { "no" }),
            RenderFact::new("Delimiter", inline_code(&details.delimiter)),
            RenderFact::new(
                "Alignment",
                inline_code(match details.alignment {
                    TypeAlignment::Char => "char",
                    TypeAlignment::Short => "int2",
                    TypeAlignment::Int => "int4",
                    TypeAlignment::Double => "double",
                }),
            ),
            RenderFact::new(
                "Storage",
                inline_code(match details.storage {
                    TypeStorage::Plain => "plain",
                    TypeStorage::External => "external",
                    TypeStorage::Main => "main",
                    TypeStorage::Extended => "extended",
                }),
            ),
            RenderFact::new("Collatable", if details.collatable { "yes" } else { "no" }),
        ]);
        for (label, value) in [
            ("Receive", details.receive.as_deref()),
            ("Send", details.send.as_deref()),
            (
                "Type modifier input",
                details.type_modifier_input.as_deref(),
            ),
            (
                "Type modifier output",
                details.type_modifier_output.as_deref(),
            ),
            ("Analyze", details.analyze.as_deref()),
            ("Subscript", details.subscript.as_deref()),
            ("Element type", details.element_type.as_deref()),
            ("Default", details.default.as_deref()),
        ] {
            if let Some(value) = value {
                facts.push(RenderFact::new(label, inline_code(value)));
            }
        }
    }
    if let Some(array_type) = &base_type.array_type {
        facts.push(RenderFact::new("Array type", inline_code(array_type)));
    }
    RenderObject {
        qualified_name: inline_code(&format!("{}.{}", base_type.namespace, base_type.name)),
        file_name: object_file_name(&base_type.namespace, &base_type.name),
        comment: base_type.comment.as_deref().map(text),
        facts,
        definition: Some(code_block("sql", &base_type.definition)),
    }
}

fn render_range_type(range_type: &RangeType) -> RenderObject {
    let multirange = format!(
        "{}.{}",
        range_type.multirange.namespace, range_type.multirange.name
    );
    let mut facts = vec![
        RenderFact::new("Kind", inline_code("range")),
        RenderFact::new("Owner", inline_code(&range_type.owner)),
        RenderFact::new("Subtype", inline_code(&range_type.subtype)),
        RenderFact::new(
            "Subtype operator class",
            inline_code(&range_type.subtype_operator_class),
        ),
        RenderFact::new("Multirange", inline_code(&multirange)),
        RenderFact::new(
            "Multirange owner",
            inline_code(&range_type.multirange.owner),
        ),
    ];
    for (label, value) in [
        ("Collation", range_type.collation.as_deref()),
        ("Canonical", range_type.canonical.as_deref()),
        ("Subtype difference", range_type.subtype_diff.as_deref()),
    ] {
        if let Some(value) = value {
            facts.push(RenderFact::new(label, inline_code(value)));
        }
    }
    if let Some(comment) = &range_type.multirange.comment {
        facts.push(RenderFact::new("Multirange comment", text(comment)));
    }
    RenderObject {
        qualified_name: inline_code(&format!("{}.{}", range_type.namespace, range_type.name)),
        file_name: object_file_name(&range_type.namespace, &range_type.name),
        comment: range_type.comment.as_deref().map(text),
        facts,
        definition: Some(code_block("sql", &range_type.definition)),
    }
}

fn render_table(table: &Table) -> RenderTable {
    let mut facts = vec![
        RenderFact::new(
            "Kind",
            inline_code(match table.kind {
                TableKind::Table => "table",
                TableKind::PartitionedTable => "partitioned_table",
                TableKind::Partition => "partition",
                TableKind::ForeignTable => "foreign_table",
            }),
        ),
        RenderFact::new("Owner", inline_code(&table.owner)),
        RenderFact::new(
            "Persistence",
            inline_code(match table.persistence {
                RelationPersistence::Permanent => "permanent",
                RelationPersistence::Unlogged => "unlogged",
            }),
        ),
        RenderFact::new(
            "Replica identity",
            inline_code(match table.replica_identity {
                ReplicaIdentity::Default => "default",
                ReplicaIdentity::Nothing => "nothing",
                ReplicaIdentity::Full => "full",
                ReplicaIdentity::Index => "index",
            }),
        ),
    ];
    if let Some(access_method) = &table.access_method {
        facts.push(RenderFact::new("Access method", inline_code(access_method)));
    }
    if let Some(typed_table) = &table.typed_table {
        facts.push(RenderFact::new("Of type", inline_code(typed_table)));
    }
    facts.extend(
        table
            .options
            .iter()
            .map(|option| RenderFact::new("Option", inline_code(option))),
    );
    if let Some(foreign) = &table.foreign {
        facts.push(RenderFact::new(
            "Foreign server",
            inline_code(&foreign.server),
        ));
        facts.push(RenderFact::new(
            "Foreign-data wrapper",
            inline_code(&foreign.wrapper),
        ));
        facts.extend(
            foreign
                .options
                .iter()
                .map(|option| RenderFact::new("Foreign option", inline_code(option))),
        );
    }
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
        if let Some(comment) = &policy.comment {
            let _ = write!(value, "; {}", text(comment));
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
        .indexes(
            table
                .indexes
                .iter()
                .filter(|index| index.extension.is_none())
                .map(render_index)
                .collect(),
        )
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
    if let Some(identity) = column.identity {
        notes.push(format!(
            "identity {}",
            inline_code(match identity {
                IdentityGeneration::Always => "always",
                IdentityGeneration::ByDefault => "by default",
            })
        ));
    }
    if let Some(collation) = &column.collation {
        notes.push(format!("collation {}", inline_code(collation)));
    }
    if let Some(generated) = &column.generated {
        notes.push(format!(
            "generated {} as {}",
            inline_code(match generated.kind {
                GeneratedColumnKind::Virtual => "virtual",
                GeneratedColumnKind::Stored => "stored",
            }),
            inline_code(&generated.expression)
        ));
    }
    if !column.enum_values.is_empty() {
        notes.push(format!(
            "enum values {}",
            inline_code(&column.enum_values.join(", "))
        ));
    }
    notes.push(format!(
        "storage {}",
        inline_code(match column.storage {
            ColumnStorage::Plain => "plain",
            ColumnStorage::External => "external",
            ColumnStorage::Main => "main",
            ColumnStorage::Extended => "extended",
        })
    ));
    if let Some(compression) = &column.compression {
        notes.push(format!(
            "compression {}",
            inline_code(match compression {
                ColumnCompression::Pglz => "pglz",
                ColumnCompression::Lz4 => "lz4",
            })
        ));
    }
    if column.statistics_target != -1 {
        notes.push(format!("statistics target {}", column.statistics_target));
    }
    notes.extend(
        column
            .options
            .iter()
            .map(|option| format!("option {}", inline_code(option))),
    );
    notes.extend(
        column
            .foreign_options
            .iter()
            .map(|option| format!("foreign option {}", inline_code(option))),
    );
    if !column.locally_defined {
        notes.push("inherited only".to_string());
    } else if column.inheritance_count > 0 {
        notes.push(format!(
            "local plus inherited from {} parent{}",
            column.inheritance_count,
            if column.inheritance_count == 1 {
                ""
            } else {
                "s"
            }
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
    if !constraint.enforced {
        details.push_str("; not enforced");
    }
    if constraint.temporal {
        details.push_str("; temporal");
    }
    if !constraint.locally_defined {
        details.push_str("; inherited");
    }
    if constraint.no_inherit {
        details.push_str("; no inherit");
    }
    if !constraint.exclusion_operators.is_empty() {
        let _ = write!(
            details,
            "; operators {}",
            inline_code(&constraint.exclusion_operators.join(", "))
        );
    }
    if let Some(comment) = &constraint.comment {
        let _ = write!(details, "; {}", text(comment));
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
    let _ = write!(origin, "; owner {}", inline_code(&index.owner));
    if let Some(tablespace) = &index.tablespace {
        let _ = write!(origin, "; tablespace {}", inline_code(tablespace));
    }
    for option in &index.options {
        let _ = write!(origin, "; option {}", inline_code(option));
    }
    if index.partitioned {
        origin.push_str("; partitioned");
    }
    if let Some(parent) = &index.parent_index {
        let _ = write!(origin, "; parent {}", inline_code(parent));
    }
    if let Some(constraint) = &index.constraint {
        let _ = write!(origin, "; constraint {}", inline_code(constraint));
    }
    if let Some(comment) = &index.comment {
        let _ = write!(origin, "; {}", text(comment));
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
        ConstraintKind::NotNull => "not_null",
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
            if !term.operator_class_parameters.is_empty() {
                let _ = write!(
                    rendered,
                    " parameters {}",
                    inline_code(&term.operator_class_parameters.join(", "))
                );
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
    let mut facts = vec![
        RenderFact::new(
            "Kind",
            inline_code(if view.materialized {
                "materialized_view"
            } else {
                "view"
            }),
        ),
        RenderFact::new("Owner", inline_code(&view.owner)),
        RenderFact::new(
            "Persistence",
            inline_code(match view.persistence {
                RelationPersistence::Permanent => "permanent",
                RelationPersistence::Unlogged => "unlogged",
            }),
        ),
    ];
    if view.materialized {
        facts.push(RenderFact::new(
            "Populated",
            if view.populated { "yes" } else { "no" },
        ));
    }
    if let Some(access_method) = &view.access_method {
        facts.push(RenderFact::new("Access method", inline_code(access_method)));
    }
    if let Some(tablespace) = &view.tablespace {
        facts.push(RenderFact::new("Tablespace", inline_code(tablespace)));
    }
    if view.security_barrier {
        facts.push(RenderFact::new("Security barrier", "yes"));
    }
    if view.security_invoker {
        facts.push(RenderFact::new("Security invoker", "yes"));
    }
    if let Some(check_option) = view.check_option {
        facts.push(RenderFact::new(
            "Check option",
            inline_code(match check_option {
                ViewCheckOption::Local => "local",
                ViewCheckOption::Cascaded => "cascaded",
            }),
        ));
    }
    facts.extend(
        view.options
            .iter()
            .map(|option| RenderFact::new("Option", inline_code(option))),
    );
    RenderView::builder()
        .qualified_name(inline_code(&format!("{}.{}", view.namespace, view.name)))
        .file_name(object_file_name(&view.namespace, &view.name))
        .comment(view.comment.as_deref().map(text))
        .facts(facts)
        .columns(view.columns.iter().map(render_column).collect())
        .indexes(
            view.indexes
                .iter()
                .filter(|index| index.extension.is_none())
                .map(render_index)
                .collect(),
        )
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

fn render_sequence(sequence: &Sequence) -> RenderObject {
    let mut facts = vec![
        RenderFact::new("Owner", inline_code(&sequence.owner)),
        RenderFact::new("Type", inline_code(&sequence.data_type)),
        RenderFact::new("Start", sequence.start.to_string()),
        RenderFact::new("Minimum", sequence.minimum.to_string()),
        RenderFact::new("Maximum", sequence.maximum.to_string()),
        RenderFact::new("Increment", sequence.increment.to_string()),
        RenderFact::new("Cache", sequence.cache.to_string()),
        RenderFact::new("Cycle", if sequence.cycle { "yes" } else { "no" }),
        RenderFact::new(
            "Persistence",
            inline_code(match sequence.persistence {
                SequencePersistence::Permanent => "permanent",
                SequencePersistence::Unlogged => "unlogged",
            }),
        ),
    ];
    if let Some(owned_by) = &sequence.owned_by {
        facts.push(RenderFact::new("Owned by", inline_code(owned_by)));
    }
    RenderObject {
        qualified_name: inline_code(&format!("{}.{}", sequence.namespace, sequence.name)),
        file_name: object_file_name(&sequence.namespace, &sequence.name),
        comment: sequence.comment.as_deref().map(text),
        facts,
        definition: Some(code_block("sql", &sequence.definition)),
    }
}

fn render_function(function: &Function) -> RenderObject {
    let mut facts = vec![
        RenderFact::new(
            "Kind",
            inline_code(match function.kind {
                FunctionKind::Ordinary => "ordinary",
                FunctionKind::Window => "window",
            }),
        ),
        RenderFact::new("Arguments", inline_code(&function.arguments)),
        RenderFact::new("Returns", inline_code(&function.return_type)),
        RenderFact::new("Owner", inline_code(&function.owner)),
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
        RenderFact::new(
            "Null input",
            if function.strict { "strict" } else { "called" },
        ),
        RenderFact::new("Leakproof", if function.leakproof { "yes" } else { "no" }),
        RenderFact::new(
            "Returns set",
            if function.returns_set { "yes" } else { "no" },
        ),
        RenderFact::new("Cost", &function.cost),
    ];
    if let Some(rows) = &function.rows {
        facts.push(RenderFact::new("Rows", rows));
    }
    if let Some(support) = &function.support_function {
        facts.push(RenderFact::new("Support function", inline_code(support)));
    }
    facts.extend(
        function
            .configuration
            .iter()
            .map(|setting| RenderFact::new("Setting", inline_code(setting))),
    );
    facts.extend(
        function
            .transforms
            .iter()
            .map(|data_type| RenderFact::new("Transform", inline_code(data_type))),
    );
    RenderObject {
        qualified_name: inline_code(&format!(
            "{}.{}{}",
            function.namespace, function.name, function.signature
        )),
        file_name: object_file_name(
            &function.namespace,
            &format!("{}{}", function.name, function.signature),
        ),
        comment: function.comment.as_deref().map(text),
        facts,
        definition: function
            .definition
            .as_deref()
            .map(|definition| code_block("sql", definition)),
    }
}

fn render_procedure(procedure: &Procedure) -> RenderObject {
    let mut facts = vec![
        RenderFact::new("Arguments", inline_code(&procedure.arguments)),
        RenderFact::new("Owner", inline_code(&procedure.owner)),
        RenderFact::new("Language", inline_code(&procedure.language)),
        RenderFact::new(
            "Security",
            inline_code(if procedure.security_definer {
                "definer"
            } else {
                "invoker"
            }),
        ),
    ];
    facts.extend(
        procedure
            .configuration
            .iter()
            .map(|setting| RenderFact::new("Setting", inline_code(setting))),
    );
    facts.extend(
        procedure
            .transforms
            .iter()
            .map(|data_type| RenderFact::new("Transform", inline_code(data_type))),
    );
    RenderObject {
        qualified_name: inline_code(&format!(
            "{}.{}{}",
            procedure.namespace, procedure.name, procedure.signature
        )),
        file_name: object_file_name(
            &procedure.namespace,
            &format!("{}{}", procedure.name, procedure.signature),
        ),
        comment: procedure.comment.as_deref().map(text),
        facts,
        definition: Some(code_block("sql", &procedure.definition)),
    }
}

fn render_aggregate(aggregate: &Aggregate) -> RenderObject {
    let mut facts = vec![
        RenderFact::new(
            "Kind",
            inline_code(match aggregate.kind {
                AggregateKind::Normal => "normal",
                AggregateKind::OrderedSet => "ordered_set",
                AggregateKind::HypotheticalSet => "hypothetical_set",
            }),
        ),
        RenderFact::new("Arguments", inline_code(&aggregate.arguments)),
        RenderFact::new("Owner", inline_code(&aggregate.owner)),
        RenderFact::new("Returns", inline_code(&aggregate.result_type)),
        RenderFact::new("Direct arguments", aggregate.direct_arguments.to_string()),
        RenderFact::new(
            "Transition function",
            inline_code(&aggregate.transition_function),
        ),
        RenderFact::new("Transition type", inline_code(&aggregate.transition_type)),
        RenderFact::new("Transition space", aggregate.transition_space.to_string()),
        RenderFact::new(
            "Final modify",
            inline_code(aggregate_final_modify_name(aggregate.final_modify)),
        ),
        RenderFact::new(
            "Parallel",
            inline_code(match aggregate.parallel {
                FunctionParallel::Safe => "safe",
                FunctionParallel::Restricted => "restricted",
                FunctionParallel::Unsafe => "unsafe",
            }),
        ),
    ];
    for (label, value) in [
        ("Final function", aggregate.final_function.as_deref()),
        ("Combine function", aggregate.combine_function.as_deref()),
        (
            "Serialization function",
            aggregate.serialization_function.as_deref(),
        ),
        (
            "Deserialization function",
            aggregate.deserialization_function.as_deref(),
        ),
        (
            "Moving transition function",
            aggregate.moving_transition_function.as_deref(),
        ),
        (
            "Moving inverse function",
            aggregate.moving_inverse_function.as_deref(),
        ),
        (
            "Moving final function",
            aggregate.moving_final_function.as_deref(),
        ),
        ("Sort operator", aggregate.sort_operator.as_deref()),
        (
            "Moving transition type",
            aggregate.moving_transition_type.as_deref(),
        ),
        ("Initial condition", aggregate.initial_condition.as_deref()),
        (
            "Moving initial condition",
            aggregate.moving_initial_condition.as_deref(),
        ),
    ] {
        if let Some(value) = value {
            facts.push(RenderFact::new(label, inline_code(value)));
        }
    }
    if aggregate.final_extra_arguments {
        facts.push(RenderFact::new("Final extra arguments", "yes"));
    }
    if aggregate.moving_final_extra_arguments {
        facts.push(RenderFact::new("Moving final extra arguments", "yes"));
    }
    if aggregate.moving_transition_type.is_some() {
        facts.push(RenderFact::new(
            "Moving transition space",
            aggregate.moving_transition_space.to_string(),
        ));
        facts.push(RenderFact::new(
            "Moving final modify",
            inline_code(aggregate_final_modify_name(aggregate.moving_final_modify)),
        ));
    }
    RenderObject {
        qualified_name: inline_code(&format!(
            "{}.{}{}",
            aggregate.namespace, aggregate.name, aggregate.signature
        )),
        file_name: object_file_name(
            &aggregate.namespace,
            &format!("{}{}", aggregate.name, aggregate.signature),
        ),
        comment: aggregate.comment.as_deref().map(text),
        facts,
        definition: None,
    }
}

const fn aggregate_final_modify_name(value: AggregateFinalModify) -> &'static str {
    match value {
        AggregateFinalModify::ReadOnly => "read_only",
        AggregateFinalModify::Shareable => "shareable",
        AggregateFinalModify::ReadWrite => "read_write",
    }
}
