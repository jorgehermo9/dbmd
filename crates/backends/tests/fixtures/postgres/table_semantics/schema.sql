CREATE SCHEMA tenancy;

CREATE TABLE tenancy.base_events (
    tenant_id bigint NOT NULL,
    payload jsonb NOT NULL
);

CREATE TABLE tenancy.special_events (
    category text NOT NULL
) INHERITS (tenancy.base_events);

CREATE TABLE tenancy.events (
    tenant_id bigint NOT NULL,
    created_at date NOT NULL,
    payload jsonb NOT NULL
) PARTITION BY RANGE (created_at);

CREATE TABLE tenancy.events_2025
PARTITION OF tenancy.events
FOR VALUES FROM ('2025-01-01') TO ('2026-01-01');

ALTER TABLE tenancy.base_events ENABLE ROW LEVEL SECURITY;
ALTER TABLE tenancy.base_events FORCE ROW LEVEL SECURITY;

CREATE POLICY tenant_events ON tenancy.base_events
    AS RESTRICTIVE
    FOR SELECT
    TO PUBLIC
    USING (tenant_id = current_setting('app.tenant_id')::bigint);
