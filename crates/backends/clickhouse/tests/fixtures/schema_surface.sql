CREATE DATABASE analytics COMMENT 'Analytical application data';

CREATE TABLE analytics.events
(
    tenant_id UInt32 COMMENT 'Owning tenant',
    event_id UUID,
    occurred_at DateTime64(3, 'UTC') CODEC(DoubleDelta, ZSTD),
    event_type LowCardinality(String) DEFAULT 'unknown',
    payload String CODEC(ZSTD(3)),
    expires_at DateTime MATERIALIZED toDateTime(occurred_at) + INTERVAL 30 DAY,
    version UInt64,
    deleted UInt8 DEFAULT 0,
    INDEX payload_tokens lower(payload) TYPE tokenbf_v1(1024, 3, 0) GRANULARITY 4,
    PROJECTION by_event_type
    (
        SELECT event_type, count()
        GROUP BY event_type
    ),
    CONSTRAINT valid_deleted CHECK deleted IN (0, 1)
)
ENGINE = ReplacingMergeTree(version, deleted)
PARTITION BY toYYYYMM(occurred_at)
PRIMARY KEY (tenant_id, event_id)
ORDER BY (tenant_id, event_id, occurred_at)
TTL toDateTime(occurred_at) + INTERVAL 365 DAY DELETE
SETTINGS index_granularity = 4096, deduplicate_merge_projection_mode = 'drop'
COMMENT 'Immutable analytical events';

CREATE TABLE analytics.event_counts
(
    event_type LowCardinality(String),
    total AggregateFunction(count)
)
ENGINE = AggregatingMergeTree
ORDER BY event_type;

CREATE MATERIALIZED VIEW analytics.event_counts_mv
TO analytics.event_counts
AS SELECT event_type, countState() AS total
FROM analytics.events
GROUP BY event_type;

CREATE VIEW analytics.active_events
COMMENT 'Non-deleted events'
AS SELECT tenant_id, event_id, occurred_at, event_type
FROM analytics.events
WHERE deleted = 0;

CREATE FUNCTION analytics_normalize AS value -> lowerUTF8(value);

CREATE TABLE analytics.country_source
(
    country_id UInt64,
    country_name String
)
ENGINE = MergeTree
ORDER BY country_id;

CREATE DICTIONARY analytics.country_names
(
    country_id UInt64,
    country_name String
)
PRIMARY KEY country_id
SOURCE(CLICKHOUSE(HOST 'localhost' PORT 9000 USER 'default' DB 'analytics' TABLE 'country_source'))
LIFETIME(MIN 0 MAX 0)
LAYOUT(HASHED());
