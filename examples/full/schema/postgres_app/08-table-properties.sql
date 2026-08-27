CREATE SCHEMA storage;

CREATE TYPE storage.device_row AS (
    device_id bigint,
    payload text
);

CREATE TABLE storage.typed_devices OF storage.device_row;

CREATE UNLOGGED TABLE storage.event_payloads (
    event_id bigint PRIMARY KEY,
    payload text
) WITH (fillfactor = 70);

ALTER TABLE storage.event_payloads REPLICA IDENTITY FULL;
ALTER TABLE storage.event_payloads ALTER COLUMN payload SET STORAGE EXTERNAL;
ALTER TABLE storage.event_payloads ALTER COLUMN payload SET COMPRESSION lz4;
ALTER TABLE storage.event_payloads ALTER COLUMN payload SET STATISTICS 777;
ALTER TABLE storage.event_payloads ALTER COLUMN payload SET (n_distinct = -0.5);

CREATE FOREIGN DATA WRAPPER fixture_wrapper
OPTIONS (api_token 'wrapper-secret', endpoint 'catalog.example');

COMMENT ON FOREIGN DATA WRAPPER fixture_wrapper IS
    'Fixture foreign-data wrapper';

CREATE SERVER fixture_server
TYPE 'catalog'
VERSION '1.0'
FOREIGN DATA WRAPPER fixture_wrapper
OPTIONS (host 'catalog.example', password 'server-secret');

COMMENT ON SERVER fixture_server IS 'Fixture foreign server';

CREATE USER MAPPING FOR PUBLIC
SERVER fixture_server
OPTIONS (user 'catalog_reader', password 'mapping-secret');

CREATE FOREIGN TABLE storage.remote_events (
    event_id bigint OPTIONS (remote_name 'external_id'),
    payload text
)
SERVER fixture_server
OPTIONS (schema_name 'remote', table_name 'events');
