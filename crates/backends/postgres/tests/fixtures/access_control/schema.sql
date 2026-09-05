CREATE ROLE dbmd_acl_owner NOLOGIN;
CREATE ROLE dbmd_acl_reader NOLOGIN;

CREATE SCHEMA secure AUTHORIZATION dbmd_acl_owner;
CREATE TABLE secure.events (
    id bigint PRIMARY KEY,
    payload text
);
ALTER TABLE secure.events OWNER TO dbmd_acl_owner;
CREATE VIEW secure.event_view AS SELECT id, payload FROM secure.events;
ALTER VIEW secure.event_view OWNER TO dbmd_acl_owner;
CREATE MATERIALIZED VIEW secure.event_rollup AS
SELECT count(*) AS event_count FROM secure.events WITH NO DATA;
ALTER MATERIALIZED VIEW secure.event_rollup OWNER TO dbmd_acl_owner;
CREATE SEQUENCE secure.event_sequence;
ALTER SEQUENCE secure.event_sequence OWNER TO dbmd_acl_owner;
CREATE DOMAIN secure.event_code AS text CHECK (VALUE <> '');
ALTER DOMAIN secure.event_code OWNER TO dbmd_acl_owner;
CREATE TYPE secure.event_state AS ENUM ('pending', 'complete');
ALTER TYPE secure.event_state OWNER TO dbmd_acl_owner;

CREATE FUNCTION secure.event_count() RETURNS bigint
LANGUAGE sql
RETURN (SELECT count(*) FROM secure.events);
ALTER FUNCTION secure.event_count() OWNER TO dbmd_acl_owner;
CREATE PROCEDURE secure.clear_events()
LANGUAGE sql
AS 'DELETE FROM secure.events';
ALTER PROCEDURE secure.clear_events() OWNER TO dbmd_acl_owner;
CREATE AGGREGATE secure.total_int(integer) (
    SFUNC = int4pl,
    STYPE = integer,
    INITCOND = '0'
);
ALTER AGGREGATE secure.total_int(integer) OWNER TO dbmd_acl_owner;

CREATE EXTENSION postgres_fdw;
CREATE SERVER secure_server FOREIGN DATA WRAPPER postgres_fdw
OPTIONS (host '127.0.0.1', dbname 'postgres');
ALTER SERVER secure_server OWNER TO dbmd_acl_owner;
CREATE FOREIGN TABLE secure.remote_events (
    id bigint,
    payload text
) SERVER secure_server OPTIONS (table_name 'events');
ALTER FOREIGN TABLE secure.remote_events OWNER TO dbmd_acl_owner;

REVOKE ALL ON SCHEMA secure FROM PUBLIC;
GRANT USAGE ON SCHEMA secure TO dbmd_acl_reader WITH GRANT OPTION;
GRANT SELECT, INSERT, UPDATE, DELETE, TRUNCATE, REFERENCES, TRIGGER, MAINTAIN
    ON TABLE secure.events TO dbmd_acl_reader;
GRANT UPDATE (payload) ON TABLE secure.events TO dbmd_acl_reader WITH GRANT OPTION;
GRANT SELECT ON TABLE secure.event_view, secure.event_rollup, secure.remote_events
    TO dbmd_acl_reader;
GRANT SELECT, UPDATE, USAGE ON SEQUENCE secure.event_sequence TO dbmd_acl_reader;
GRANT EXECUTE ON FUNCTION secure.event_count() TO dbmd_acl_reader;
GRANT EXECUTE ON PROCEDURE secure.clear_events() TO dbmd_acl_reader;
GRANT EXECUTE ON FUNCTION secure.total_int(integer) TO dbmd_acl_reader;
GRANT USAGE ON DOMAIN secure.event_code TO dbmd_acl_reader;
GRANT USAGE ON TYPE secure.event_state TO dbmd_acl_reader;
GRANT USAGE ON LANGUAGE plpgsql TO dbmd_acl_reader;
GRANT USAGE ON FOREIGN DATA WRAPPER postgres_fdw TO dbmd_acl_reader;
GRANT USAGE ON FOREIGN SERVER secure_server TO dbmd_acl_reader;
GRANT SET ON PARAMETER work_mem TO dbmd_acl_reader;
GRANT ALTER SYSTEM ON PARAMETER statement_timeout TO dbmd_acl_reader;
GRANT CREATE ON TABLESPACE pg_default TO dbmd_acl_reader;

DO $fixture$
BEGIN
    EXECUTE format(
        'GRANT CONNECT, CREATE, TEMPORARY ON DATABASE %I TO dbmd_acl_reader',
        current_database()
    );
    EXECUTE format(
        'ALTER ROLE dbmd_acl_reader IN DATABASE %I SET lock_timeout = %L',
        current_database(),
        '3s'
    );
END
$fixture$;

SELECT pg_catalog.lo_create(424242);
SELECT pg_catalog.lo_put(
    424242,
    0,
    pg_catalog.convert_to('large-object-secret', 'UTF8')
);
ALTER LARGE OBJECT 424242 OWNER TO dbmd_acl_owner;
GRANT SELECT, UPDATE ON LARGE OBJECT 424242 TO dbmd_acl_reader;
COMMENT ON LARGE OBJECT 424242 IS 'Fixture document payload';

ALTER DEFAULT PRIVILEGES FOR ROLE dbmd_acl_owner IN SCHEMA secure
    GRANT SELECT ON TABLES TO dbmd_acl_reader WITH GRANT OPTION;
ALTER DEFAULT PRIVILEGES FOR ROLE dbmd_acl_owner IN SCHEMA secure
    GRANT USAGE ON SEQUENCES TO dbmd_acl_reader;
ALTER DEFAULT PRIVILEGES FOR ROLE dbmd_acl_owner IN SCHEMA secure
    GRANT EXECUTE ON ROUTINES TO dbmd_acl_reader;
ALTER DEFAULT PRIVILEGES FOR ROLE dbmd_acl_owner IN SCHEMA secure
    GRANT USAGE ON TYPES TO dbmd_acl_reader;
ALTER DEFAULT PRIVILEGES FOR ROLE dbmd_acl_owner
    GRANT USAGE ON SCHEMAS TO dbmd_acl_reader;
ALTER DEFAULT PRIVILEGES FOR ROLE dbmd_acl_owner
    GRANT SELECT ON LARGE OBJECTS TO dbmd_acl_reader;
