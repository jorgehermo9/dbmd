SET allow_experimental_codecs = 1;
SET allow_experimental_window_view = 1;
SET allow_experimental_analyzer = 0;

CREATE DATABASE analytics UUID '10000000-0000-0000-0000-000000000001'
ENGINE = Atomic
COMMENT 'Analytical application data';

CREATE TABLE analytics.events UUID '20000000-0000-0000-0000-000000000001'
(
    tenant_id UInt32 COMMENT 'Owning tenant',
    event_id UUID,
    occurred_at DateTime64(3, 'UTC') CODEC(DoubleDelta, ZSTD),
    event_type LowCardinality(String) DEFAULT 'unknown',
    payload String CODEC(ZSTD(3)),
    vector QBit(Float32, 8),
    expires_at DateTime MATERIALIZED toDateTime(occurred_at) + INTERVAL 30 DAY,
    version UInt64,
    deleted UInt8 DEFAULT 0,
    INDEX payload_tokens lower(payload) TYPE tokenbf_v1(1024, 3, 0) GRANULARITY 4,
    INDEX payload_text payload TYPE text(tokenizer = 'splitByNonAlpha') GRANULARITY 128,
    PROJECTION by_event_type
    (
        SELECT event_type, count()
        GROUP BY event_type
    ),
    PROJECTION by_time INDEX occurred_at TYPE basic,
    CONSTRAINT valid_deleted CHECK deleted IN (0, 1),
    CONSTRAINT positive_tenant ASSUME tenant_id > 0
)
ENGINE = ReplacingMergeTree(version, deleted)
PARTITION BY toYYYYMM(occurred_at)
PRIMARY KEY (tenant_id, event_id)
ORDER BY (tenant_id, event_id, occurred_at)
TTL toDateTime(occurred_at) + INTERVAL 365 DAY DELETE
SETTINGS index_granularity = 4096,
         deduplicate_merge_projection_mode = 'drop',
         auto_statistics_types = 'minmax',
         add_minmax_index_for_temporal_columns = 1
COMMENT 'Immutable analytical events';

CREATE TABLE analytics.event_counts UUID '20000000-0000-0000-0000-000000000002'
(
    event_type LowCardinality(String),
    total AggregateFunction(count)
)
ENGINE = AggregatingMergeTree
ORDER BY event_type;

CREATE MATERIALIZED VIEW analytics.event_counts_mv UUID '20000000-0000-0000-0000-000000000003'
TO analytics.event_counts
AS SELECT event_type, countState() AS total
FROM analytics.events
GROUP BY event_type;

CREATE VIEW analytics.active_events UUID '20000000-0000-0000-0000-000000000004'
DEFINER = CURRENT_USER SQL SECURITY INVOKER
COMMENT 'Non-deleted events'
AS SELECT tenant_id, event_id, occurred_at, event_type
FROM analytics.events
WHERE deleted = 0;

CREATE VIEW analytics.events_by_tenant UUID '20000000-0000-0000-0000-000000000017'
AS SELECT tenant_id, event_id, occurred_at, event_type
FROM analytics.events
WHERE tenant_id = {requested_tenant:UInt32};

CREATE FUNCTION analytics_normalize AS value -> lowerUTF8(value);

CREATE TABLE analytics.country_source UUID '20000000-0000-0000-0000-000000000005'
(
    country_id UInt64,
    parent_id UInt64,
    country_name String,
    valid_from Date,
    valid_to Date,
    rate Float64
)
ENGINE = MergeTree
ORDER BY country_id;

CREATE DICTIONARY analytics.country_names UUID '20000000-0000-0000-0000-000000000006'
(
    country_id UInt64 IS_OBJECT_ID,
    parent_id UInt64 DEFAULT 0 HIERARCHICAL,
    country_name String DEFAULT 'unknown' INJECTIVE,
    normalized_name String EXPRESSION lowerUTF8(country_name)
)
PRIMARY KEY country_id
SOURCE(CLICKHOUSE(HOST 'localhost' PORT 9000 USER 'default' PASSWORD 'dbmd-dictionary-secret-sentinel' DB 'analytics' TABLE 'country_source'))
LIFETIME(MIN 30 MAX 60)
LAYOUT(HASHED())
SETTINGS(max_threads_for_updates = 4);

CREATE DICTIONARY analytics.country_rates UUID '20000000-0000-0000-0000-000000000016'
(
    country_id UInt64,
    valid_from Date,
    valid_to Date,
    rate Float64 DEFAULT 0
)
PRIMARY KEY country_id
SOURCE(CLICKHOUSE(HOST 'localhost' PORT 9000 USER 'default' PASSWORD 'dbmd-dictionary-secret-sentinel' DB 'analytics' TABLE 'country_source'))
LIFETIME(0)
LAYOUT(RANGE_HASHED())
RANGE(MIN valid_from MAX valid_to);

CREATE TABLE analytics.remote_accounts UUID '20000000-0000-0000-0000-000000000007'
(
    account_id UInt64,
    email String
)
ENGINE = MySQL(
    '127.0.0.1:3306',
    'remote_app',
    'accounts',
    'remote_reader',
    'dbmd-engine-secret-sentinel'
);

CREATE TABLE analytics.retention_matrix UUID '20000000-0000-0000-0000-000000000008'
(
    event_id UInt64,
    occurred_at DateTime,
    expires_at DateTime TTL expires_at + INTERVAL 1 DAY,
    deleted UInt8,
    payload String
)
ENGINE = MergeTree
ORDER BY event_id
TTL occurred_at + INTERVAL 7 DAY TO DISK 'default',
    occurred_at + INTERVAL 10 DAY TO VOLUME 'default',
    occurred_at + INTERVAL 14 DAY RECOMPRESS CODEC(ZSTD(9)),
    occurred_at + INTERVAL 30 DAY DELETE WHERE deleted = 1
SETTINGS index_granularity = 4096;

CREATE TABLE analytics.retention_rollup UUID '20000000-0000-0000-0000-000000000009'
(
    tenant_id UInt64,
    occurred_at DateTime,
    amount UInt64
)
ENGINE = MergeTree
ORDER BY tenant_id
TTL occurred_at + INTERVAL 30 DAY GROUP BY tenant_id SET amount = sum(amount);

CREATE TABLE analytics.window_event_counts UUID '20000000-0000-0000-0000-000000000018'
(
    total UInt64,
    window_end DateTime
)
ENGINE = MergeTree
ORDER BY window_end;

CREATE WINDOW VIEW analytics.windowed_events UUID '20000000-0000-0000-0000-000000000019'
TO analytics.window_event_counts
INNER ENGINE = AggregatingMergeTree ORDER BY tuple()
WATERMARK=STRICTLY_ASCENDING
AS SELECT count(event_id) AS total, tumbleEnd(window_id) AS window_end
FROM analytics.retention_matrix
GROUP BY tumble(occurred_at, INTERVAL '5' SECOND) AS window_id;

CREATE WINDOW VIEW analytics.windowed_events_owned UUID '20000000-0000-0000-0000-000000000020'
INNER ENGINE = AggregatingMergeTree ORDER BY tuple()
ENGINE = MergeTree ORDER BY window_end
WATERMARK=ASCENDING
ALLOWED_LATENESS=INTERVAL '2' SECOND
AS SELECT count(event_id) AS total, tumbleEnd(window_id) AS window_end
FROM analytics.retention_matrix
GROUP BY tumble(occurred_at, INTERVAL '5' SECOND) AS window_id;

CREATE TABLE analytics.refresh_snapshots UUID '20000000-0000-0000-0000-000000000010'
(
    tenant_id UInt32,
    captured_at DateTime
)
ENGINE = MergeTree
ORDER BY tenant_id;

CREATE TABLE analytics.refresh_rollups UUID '20000000-0000-0000-0000-000000000011'
(
    tenant_id UInt32,
    total UInt64
)
ENGINE = MergeTree
ORDER BY tenant_id;

CREATE MATERIALIZED VIEW analytics.refresh_base UUID '20000000-0000-0000-0000-000000000012'
REFRESH EVERY 1 HOUR OFFSET 5 MINUTE RANDOMIZE FOR 1 MINUTE
SETTINGS refresh_retries = 5
APPEND TO analytics.refresh_snapshots
EMPTY
AS SELECT tenant_id, now() AS captured_at
FROM analytics.events;

CREATE MATERIALIZED VIEW analytics.refresh_dependent UUID '20000000-0000-0000-0000-000000000013'
REFRESH AFTER 2 HOUR DEPENDS ON analytics.refresh_base
SETTINGS refresh_retries = 3
TO analytics.refresh_rollups
EMPTY
AS SELECT tenant_id, count() AS total
FROM analytics.events
GROUP BY tenant_id;

CREATE TABLE analytics.modern_storage UUID '20000000-0000-0000-0000-000000000014'
(
    id UInt64,
    measurement Float64 CODEC(ALP, ZSTD),
    attributes Map(String, String)
)
ENGINE = CoalescingMergeTree
ORDER BY id
SETTINGS map_serialization_version = 'with_buckets',
         map_serialization_version_for_zero_level_parts = 'basic',
         map_buckets_strategy = 'linear',
         map_buckets_coefficient = 0.5,
         map_buckets_min_avg_size = 0,
         max_buckets_in_map = 64;

CREATE TABLE analytics.s3_archive UUID '20000000-0000-0000-0000-000000000015'
(
    payload String
)
ENGINE = S3(
    's3://dbmd-audit/archive.parquet',
    storage_class_name = 'INTELLIGENT_TIERING'
);

CREATE ROLE analytics_reader;

CREATE USER analytics_service
IDENTIFIED WITH sha256_password BY 'dbmd-password-sentinel'
HOST LOCAL
DEFAULT ROLE analytics_reader
DEFAULT DATABASE analytics;

GRANT SELECT ON analytics.* TO analytics_reader WITH GRANT OPTION;
GRANT analytics_reader TO analytics_service WITH ADMIN OPTION;

CREATE ROW POLICY tenant_events ON analytics.events
FOR SELECT USING tenant_id > 0
TO analytics_reader;

CREATE QUOTA analytics_quota
KEYED BY user_name
FOR INTERVAL 1 HOUR MAX queries = 100,
                        query_selects = 90,
                        query_inserts = 10,
                        errors = 5,
                        result_rows = 1000,
                        result_bytes = 2000,
                        read_rows = 3000,
                        read_bytes = 4000,
                        written_bytes = 5000,
                        execution_time = 60,
                        failed_sequential_authentications = 3,
                        queries_per_normalized_hash = 7
TO analytics_reader;

CREATE SETTINGS PROFILE analytics_profile
SETTINGS max_threads = 4 MIN 1 MAX 8 WRITABLE
TO analytics_reader;

CREATE NAMED COLLECTION analytics_remote AS
host = 'localhost' OVERRIDABLE,
password = 'dbmd-collection-secret-sentinel' NOT OVERRIDABLE;

CREATE RESOURCE analytics_cpu (MASTER THREAD, WORKER THREAD);
CREATE WORKLOAD analytics_all;
CREATE WORKLOAD analytics_interactive IN analytics_all
SETTINGS max_concurrent_threads = 8 FOR analytics_cpu;
