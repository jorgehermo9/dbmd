use std::str::FromStr;

use dbmd_backend_clickhouse::{
    introspect, render_source, template_files, ClickHouseSource, ConstraintKind, RefreshSchedule,
    ResourceOperation, TableKind, TtlAction, TtlDestination, ViewSqlSecurity,
};
use dbmd_core::SourceId;
use dbmd_render::{OutputLayout, RenderContext, RenderOptions, RenderedArtifact, Renderer};
use dbmd_test_support::ClickHouseServer;

#[test]
fn introspects_the_clickhouse_schema_surface_deterministically() {
    let server = ClickHouseServer::start_with_settings(
        include_str!("fixtures/schema_surface.sql"),
        &[
            ("allow_experimental_codecs", "1"),
            ("allow_experimental_window_view", "1"),
            ("allow_experimental_analyzer", "0"),
        ],
    )
    .expect("ClickHouse test container should start");
    let source = ClickHouseSource::new(
        SourceId::from_str("analytics").expect("test source ID should be valid"),
        server.endpoint(),
    )
    .with_database("analytics");

    let first = introspect(&source).expect("ClickHouse introspection should succeed");
    let second = introspect(&source).expect("repeated introspection should succeed");

    assert_eq!(first, second);
    assert_eq!(first.catalog().databases.len(), 1);
    assert_eq!(
        first.catalog().databases[0].uuid,
        "10000000-0000-0000-0000-000000000001"
    );
    assert_eq!(
        first.catalog().tables.len(),
        20,
        "unexpected table set: {:?}",
        first
            .catalog()
            .tables
            .iter()
            .map(|table| table.name.as_str())
            .collect::<Vec<_>>()
    );
    assert!(first
        .catalog()
        .tables
        .iter()
        .any(|table| table.name == "country_names" && table.kind == TableKind::Dictionary));
    let events = first
        .catalog()
        .tables
        .iter()
        .find(|table| table.name == "events")
        .expect("events table should be present");
    assert_eq!(events.kind, TableKind::Table);
    assert_eq!(events.uuid, "20000000-0000-0000-0000-000000000001");
    assert_eq!(events.engine, "ReplacingMergeTree");
    assert_eq!(events.partition_key, "toYYYYMM(occurred_at)");
    assert_eq!(events.primary_key, "tenant_id, event_id");
    assert_eq!(events.sorting_key, "tenant_id, event_id, occurred_at");
    assert!(events
        .data_skipping_indexes
        .iter()
        .any(|index| index.name == "payload_text" && index.index_type == "text"));
    assert!(
        events
            .data_skipping_indexes
            .iter()
            .any(|index| index.name == "auto_minmax_index_occurred_at"
                && index.implicit == Some(true))
    );
    assert_eq!(events.projections.len(), 2);
    let projection_index = events
        .projections
        .iter()
        .find(|projection| projection.name == "by_time")
        .and_then(|projection| projection.index.as_ref())
        .expect("projection-index role should be retained");
    assert_eq!(projection_index.expression, "occurred_at");
    assert_eq!(projection_index.index_type, "basic");
    assert_eq!(events.constraints.len(), 2);
    assert!(events.constraints.iter().any(|constraint| {
        constraint.name == "positive_tenant" && constraint.kind == ConstraintKind::Assume
    }));
    assert!(events.columns.iter().any(
        |column| column.name == "expires_at" && column.default_kind.as_str() == "materialized"
    ));
    assert!(events
        .columns
        .iter()
        .any(|column| column.name == "vector" && column.data_type == "QBit(Float32, 8)"));
    assert!(events.columns.iter().any(|column| {
        column.name == "occurred_at" && column.statistics.as_deref() == Some("minmax(auto)")
    }));
    assert!(events
        .engine_full
        .contains("add_minmax_index_for_temporal_columns = 1"));
    assert_eq!(events.engine_arguments, ["version", "deleted"]);
    assert_eq!(events.settings["index_granularity"], "4096");
    assert!(matches!(
        events.ttl_rules[0].action,
        TtlAction::Delete { predicate: None }
    ));
    let retention = first
        .catalog()
        .tables
        .iter()
        .find(|table| table.name == "retention_matrix")
        .expect("multi-action retention table should be present");
    assert_eq!(retention.target, None);
    assert_eq!(retention.ttl_rules.len(), 4);
    assert!(matches!(
        retention.ttl_rules[0].action,
        TtlAction::Move {
            destination: TtlDestination::Disk,
            ..
        }
    ));
    assert!(matches!(
        retention.ttl_rules[1].action,
        TtlAction::Move {
            destination: TtlDestination::Volume,
            ..
        }
    ));
    assert!(matches!(
        retention.ttl_rules[2].action,
        TtlAction::Recompress { .. }
    ));
    assert!(matches!(
        retention.ttl_rules[3].action,
        TtlAction::Delete {
            predicate: Some(ref predicate)
        } if predicate == "deleted = 1"
    ));
    assert_eq!(
        retention
            .columns
            .iter()
            .find(|column| column.name == "expires_at")
            .and_then(|column| column.ttl.as_deref()),
        Some("expires_at + toIntervalDay(1)")
    );
    let rollup = first
        .catalog()
        .tables
        .iter()
        .find(|table| table.name == "retention_rollup")
        .expect("TTL rollup table should be present");
    assert!(matches!(
        rollup.ttl_rules[0].action,
        TtlAction::GroupBy {
            ref keys,
            ref assignments,
        } if keys == "tenant_id" && assignments == &["amount = sum(amount)"]
    ));
    let refresh_base = first
        .catalog()
        .tables
        .iter()
        .find(|table| table.name == "refresh_base")
        .and_then(|table| table.refresh.as_ref())
        .expect("EVERY refresh contract should be present");
    assert!(matches!(
        refresh_base.schedule,
        RefreshSchedule::Every { ref interval } if interval == "1 HOUR"
    ));
    assert_eq!(refresh_base.offset.as_deref(), Some("5 MINUTE"));
    assert_eq!(refresh_base.randomize_for.as_deref(), Some("1 MINUTE"));
    assert_eq!(refresh_base.settings["refresh_retries"], "5");
    assert!(refresh_base.append);
    let refresh_dependent = first
        .catalog()
        .tables
        .iter()
        .find(|table| table.name == "refresh_dependent")
        .and_then(|table| table.refresh.as_ref())
        .expect("AFTER refresh contract should be present");
    assert!(matches!(
        refresh_dependent.schedule,
        RefreshSchedule::After { ref interval } if interval == "2 HOUR"
    ));
    assert_eq!(refresh_dependent.dependencies.len(), 1);
    assert_eq!(refresh_dependent.dependencies[0].table, "refresh_base");
    assert!(!refresh_dependent.append);
    let modern_storage = first
        .catalog()
        .tables
        .iter()
        .find(|table| table.name == "modern_storage")
        .expect("26.6 modern storage table should be present");
    assert_eq!(modern_storage.engine, "CoalescingMergeTree");
    assert_eq!(
        modern_storage.settings["map_serialization_version"],
        "'with_buckets'"
    );
    assert_eq!(modern_storage.settings["max_buckets_in_map"], "64");
    assert_eq!(
        modern_storage
            .columns
            .iter()
            .find(|column| column.name == "measurement")
            .and_then(|column| column.compression_codec.as_deref()),
        Some("CODEC(ALP, ZSTD(1))")
    );
    assert!(modern_storage
        .columns
        .iter()
        .any(|column| column.data_type == "Map(String, String)"));
    let s3_archive = first
        .catalog()
        .tables
        .iter()
        .find(|table| table.name == "s3_archive")
        .expect("S3 storage-class table should be present");
    assert_eq!(s3_archive.engine, "S3");
    assert_eq!(s3_archive.engine_arguments.len(), 2);
    assert_eq!(
        s3_archive.engine_parameters["storage_class_name"],
        "'INTELLIGENT_TIERING'"
    );
    let materialized_view = first
        .catalog()
        .tables
        .iter()
        .find(|table| table.name == "event_counts_mv")
        .expect("materialized view should be present");
    assert_eq!(materialized_view.kind, TableKind::MaterializedView);
    assert_eq!(
        materialized_view.target.as_deref(),
        Some("analytics.event_counts")
    );
    let active_view = first
        .catalog()
        .tables
        .iter()
        .find(|table| table.name == "active_events")
        .expect("secured ordinary view should be present");
    assert_eq!(active_view.definer.as_deref(), Some("default"));
    assert_eq!(active_view.sql_security, Some(ViewSqlSecurity::Invoker));
    let parameterized_view = first
        .catalog()
        .tables
        .iter()
        .find(|table| table.name == "events_by_tenant")
        .expect("parameterized view should be present");
    assert_eq!(parameterized_view.parameters.len(), 1);
    assert_eq!(parameterized_view.parameters[0].name, "requested_tenant");
    assert_eq!(parameterized_view.parameters[0].data_type, "UInt32");
    assert_eq!(first.catalog().functions.len(), 1);
    assert_eq!(first.catalog().functions[0].origin, "SQLUserDefined");
    assert_eq!(first.catalog().functions[0].syntax, None);
    let dictionary = first
        .catalog()
        .tables
        .iter()
        .find(|table| table.name == "country_names")
        .and_then(|table| table.dictionary.as_ref())
        .expect("dictionary metadata should be attached");
    assert_eq!(dictionary.layout, "HASHED");
    assert_eq!(dictionary.keys[0].name, "country_id");
    assert!(dictionary.keys[0].object_id);
    let parent = dictionary
        .attributes
        .iter()
        .find(|field| field.name == "parent_id")
        .expect("hierarchical dictionary field should be present");
    assert_eq!(parent.default_expression.as_deref(), Some("0"));
    assert!(parent.hierarchical);
    let country_name = dictionary
        .attributes
        .iter()
        .find(|field| field.name == "country_name")
        .expect("injective dictionary field should be present");
    assert_eq!(
        country_name.default_expression.as_deref(),
        Some("'unknown'")
    );
    assert!(country_name.injective);
    assert_eq!(
        dictionary
            .attributes
            .iter()
            .find(|field| field.name == "normalized_name")
            .and_then(|field| field.expression.as_deref()),
        Some("lowerUTF8(country_name)")
    );
    assert_eq!(dictionary.lifetime_min_seconds, 30);
    assert_eq!(dictionary.lifetime_max_seconds, 60);
    assert_eq!(dictionary.settings["max_threads_for_updates"], "4");
    let range_dictionary = first
        .catalog()
        .tables
        .iter()
        .find(|table| table.name == "country_rates")
        .and_then(|table| table.dictionary.as_ref())
        .expect("range dictionary metadata should be attached");
    assert_eq!(range_dictionary.layout, "RANGE_HASHED");
    assert_eq!(range_dictionary.range_min.as_deref(), Some("valid_from"));
    assert_eq!(range_dictionary.range_max.as_deref(), Some("valid_to"));
    let targeted_window = first
        .catalog()
        .tables
        .iter()
        .find(|table| table.name == "windowed_events")
        .expect("targeted window view should be present");
    assert_eq!(targeted_window.kind, TableKind::WindowView);
    assert_eq!(
        targeted_window.target.as_deref(),
        Some("analytics.window_event_counts")
    );
    let targeted_window_contract = targeted_window
        .window
        .as_ref()
        .expect("window execution contract should be typed");
    assert_eq!(
        targeted_window_contract.inner_engine.as_deref(),
        Some("AggregatingMergeTree ORDER BY tuple()")
    );
    assert_eq!(
        targeted_window_contract.watermark.as_deref(),
        Some("STRICTLY_ASCENDING")
    );
    let owned_window = first
        .catalog()
        .tables
        .iter()
        .find(|table| table.name == "windowed_events_owned")
        .and_then(|table| table.window.as_ref())
        .expect("owned window view should be present");
    assert!(owned_window
        .storage_engine
        .as_deref()
        .is_some_and(|engine| engine.starts_with("MergeTree ORDER BY window_end")));
    assert_eq!(owned_window.watermark.as_deref(), Some("ASCENDING"));
    assert_eq!(
        owned_window.allowed_lateness.as_deref(),
        Some("toIntervalSecond('2')")
    );
    let remote_accounts = first
        .catalog()
        .tables
        .iter()
        .find(|table| table.name == "remote_accounts")
        .expect("external MySQL table should be present");
    assert!(remote_accounts.engine_full.contains("[HIDDEN]"));
    let service_user = first
        .catalog()
        .users
        .iter()
        .find(|user| user.name == "analytics_service")
        .expect("fixture user should be present");
    assert_eq!(service_user.authentication_types, ["sha256_password"]);
    assert_eq!(service_user.default_database.as_deref(), Some("analytics"));
    assert_eq!(first.catalog().roles[0].name, "analytics_reader");
    assert!(first
        .catalog()
        .grants
        .iter()
        .any(|grant| grant.role.as_deref() == Some("analytics_reader")
            && grant.access_type == "SELECT"
            && grant.grant_option));
    assert!(first
        .catalog()
        .role_grants
        .iter()
        .any(|grant| grant.user.as_deref() == Some("analytics_service")
            && grant.granted_role == "analytics_reader"
            && grant.default
            && grant.admin_option));
    assert_eq!(first.catalog().row_policies[0].short_name, "tenant_events");
    assert_eq!(
        first.catalog().quotas[0].limits[0].max_queries_per_normalized_hash,
        Some(7)
    );
    assert_eq!(
        first.catalog().settings_profiles[0].elements[0]
            .setting_name
            .as_deref(),
        Some("max_threads")
    );
    assert_eq!(first.catalog().named_collections[0].entries[0].key, "host");
    assert_eq!(
        first.catalog().named_collections[0].entries[0].overridable,
        Some(true)
    );
    assert_eq!(
        first.catalog().named_collections[0].entries[1].overridable,
        Some(false)
    );
    assert_eq!(first.catalog().resources[0].unit, "CPUNanosecond");
    assert_eq!(
        first.catalog().resources[0].operations,
        [
            ResourceOperation::MasterThread,
            ResourceOperation::WorkerThread
        ]
    );
    assert_eq!(
        first.catalog().workloads[1].parent.as_deref(),
        Some("analytics_all")
    );
    assert_eq!(
        first.catalog().workloads[1].settings[0].name,
        "max_concurrent_threads"
    );
    assert_eq!(
        first.catalog().workloads[1].settings[0].resource.as_deref(),
        Some("analytics_cpu")
    );
    let serialized = serde_json::to_string(&first)
        .expect("ClickHouse catalog should serialize for safety check");
    assert!(!serialized.contains("dbmd-password-sentinel"));
    assert!(!serialized.contains("dbmd-collection-secret-sentinel"));
    assert!(!serialized.contains("dbmd-dictionary-secret-sentinel"));
    assert!(!serialized.contains("dbmd-engine-secret-sentinel"));
    insta::assert_yaml_snapshot!("clickhouse_schema_surface", first);

    let context = RenderContext::new(vec![render_source(
        first.id(),
        first.display_name(),
        first.catalog(),
        false,
    )]);
    let renderer =
        Renderer::embedded(template_files()).expect("ClickHouse embedded templates should compile");
    let RenderedArtifact::SingleFile(markdown) = renderer
        .render(&context)
        .expect("ClickHouse presentation should render")
    else {
        panic!("default ClickHouse rendering should produce one file");
    };
    let markdown = String::from_utf8(markdown).expect("rendered Markdown should be UTF-8");
    assert!(markdown.contains("ReplacingMergeTree"));
    assert!(markdown.contains("payload_tokens"));
    assert!(markdown.contains("analytics_normalize"));
    assert!(markdown.contains("QBit(Float32, 8)"));
    assert!(markdown.contains("payload_text"));
    assert!(markdown.contains("move to volume 'default'"));
    assert!(markdown.contains("group by tenant_id set amount = sum(amount)"));
    assert!(markdown.contains("TTL `expires_at + toIntervalDay(1)`"));
    assert!(markdown.contains("**Refresh:** `every 1 HOUR`"));
    assert!(markdown.contains("**Refresh offset:** `5 MINUTE`"));
    assert!(markdown.contains("**Refresh mode:** `append`"));
    assert!(markdown.contains("**Refresh depends on:** `analytics.refresh_base`"));
    assert!(markdown.contains("CoalescingMergeTree"));
    assert!(markdown.contains("codec `CODEC(ALP, ZSTD(1))`"));
    assert!(markdown.contains("`map_serialization_version` = `'with_buckets'`"));
    assert!(
        markdown.contains("**Engine parameter:** `storage_class_name` = `'INTELLIGENT_TIERING'`")
    );
    assert!(markdown.contains("index occurred_at type basic"));
    assert!(markdown.contains("**SQL security:** `invoker`"));
    assert!(markdown.contains("`requested_tenant` `UInt32`"));
    assert!(markdown.contains("parent_id UInt64 DEFAULT 0 HIERARCHICAL"));
    assert!(markdown.contains("country_name String DEFAULT 'unknown' INJECTIVE"));
    assert!(markdown.contains("**Dictionary range:** MIN `valid_from` MAX `valid_to`"));
    assert!(markdown.contains("**Dictionary setting:** `max_threads_for_updates` = `4`"));
    assert!(markdown.contains("**Window inner engine:** `AggregatingMergeTree ORDER BY tuple()`"));
    assert!(markdown.contains("**Watermark:** `STRICTLY_ASCENDING`"));
    assert!(markdown.contains("**Allowed lateness:** `toIntervalSecond('2')`"));
    assert!(markdown.contains("analytics_service"));
    assert!(markdown.contains("queries_per_normalized_hash=7"));
    assert!(markdown.contains("analytics_remote"));
    assert!(markdown.contains("**Entry:** `host`; overridable"));
    assert!(markdown.contains("**Entry:** `password`; not overridable"));
    assert!(markdown.contains("**Operation:** `master thread`"));
    assert!(markdown.contains("`max_concurrent_threads` = `8` for `analytics_cpu`"));
    assert!(!markdown.contains("dbmd-password-sentinel"));
    assert!(!markdown.contains("dbmd-collection-secret-sentinel"));
    assert!(!markdown.contains("dbmd-dictionary-secret-sentinel"));
    assert!(!markdown.contains("dbmd-engine-secret-sentinel"));
    insta::assert_snapshot!("clickhouse_markdown", markdown);
    assert_directory_render(&renderer, &context, "tables/analytics.events.md");
    assert_directory_render(
        &renderer,
        &context,
        "access-and-workloads/user.analytics_service.md",
    );
}

fn assert_directory_render(renderer: &Renderer, context: &RenderContext, object_path: &str) {
    let RenderedArtifact::Directory(files) = renderer
        .render_with_options(
            context,
            RenderOptions {
                layout: OutputLayout::Directory,
                ..RenderOptions::default()
            },
        )
        .expect("ClickHouse directory profile should render")
    else {
        panic!("directory options should produce a directory artifact");
    };
    let index_path = "index.md"
        .parse()
        .expect("static artifact path should parse");
    let index = String::from_utf8(files[&index_path].clone()).expect("index should be UTF-8");
    assert!(index.contains("tables/analytics.events.md"));
    assert!(files.keys().any(|path| path.as_str() == object_path));
}
