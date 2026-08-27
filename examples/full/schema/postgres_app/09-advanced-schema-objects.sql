CREATE SCHEMA advanced;

CREATE TABLE advanced.orders (
    id bigint PRIMARY KEY,
    customer_id bigint NOT NULL,
    region text NOT NULL,
    amount numeric(12, 2) NOT NULL
);

CREATE TABLE advanced.deleted_orders (
    id bigint,
    deleted_at timestamptz NOT NULL DEFAULT current_timestamp
);

CREATE RULE archive_order_delete AS
ON DELETE TO advanced.orders
DO ALSO INSERT INTO advanced.deleted_orders (id) VALUES (OLD.id);

ALTER TABLE advanced.orders ENABLE REPLICA RULE archive_order_delete;
COMMENT ON RULE archive_order_delete ON advanced.orders IS
    'Archives replicated deletes';

CREATE FUNCTION advanced.capture_schema_change()
RETURNS event_trigger
LANGUAGE plpgsql
AS $$
BEGIN
    NULL;
END;
$$;

CREATE EVENT TRIGGER capture_schema_change
ON ddl_command_end
WHEN TAG IN ('CREATE TABLE', 'ALTER TABLE')
EXECUTE FUNCTION advanced.capture_schema_change();

ALTER EVENT TRIGGER capture_schema_change ENABLE ALWAYS;
COMMENT ON EVENT TRIGGER capture_schema_change IS
    'Captures selected schema changes';

CREATE STATISTICS advanced.orders_dependencies
    (ndistinct, dependencies, mcv)
ON customer_id, region
FROM advanced.orders;

ALTER STATISTICS advanced.orders_dependencies SET STATISTICS 500;
COMMENT ON STATISTICS advanced.orders_dependencies IS
    'Cross-column order distribution';

CREATE STATISTICS advanced.orders_expression
ON (lower(region))
FROM advanced.orders;

CREATE TEXT SEARCH PARSER advanced.default_parser (
    START = pg_catalog.prsd_start,
    GETTOKEN = pg_catalog.prsd_nexttoken,
    END = pg_catalog.prsd_end,
    HEADLINE = pg_catalog.prsd_headline,
    LEXTYPES = pg_catalog.prsd_lextype
);
COMMENT ON TEXT SEARCH PARSER advanced.default_parser IS
    'Fixture parser backed by PostgreSQL defaults';

CREATE TEXT SEARCH TEMPLATE advanced.simple_template (
    INIT = pg_catalog.dsimple_init,
    LEXIZE = pg_catalog.dsimple_lexize
);
COMMENT ON TEXT SEARCH TEMPLATE advanced.simple_template IS
    'Fixture simple dictionary template';

CREATE TEXT SEARCH DICTIONARY advanced.simple_dictionary (
    TEMPLATE = advanced.simple_template,
    STOPWORDS = english
);
COMMENT ON TEXT SEARCH DICTIONARY advanced.simple_dictionary IS
    'Fixture stop-word dictionary';

CREATE TEXT SEARCH CONFIGURATION advanced.search_configuration (
    PARSER = advanced.default_parser
);
ALTER TEXT SEARCH CONFIGURATION advanced.search_configuration
ADD MAPPING FOR asciiword
WITH advanced.simple_dictionary, pg_catalog.english_stem;
COMMENT ON TEXT SEARCH CONFIGURATION advanced.search_configuration IS
    'Fixture search pipeline';

CREATE PUBLICATION advanced_publication FOR TABLE advanced.orders;

CREATE SUBSCRIPTION advanced_subscription
CONNECTION 'host=127.0.0.1 dbname=publisher user=replicator password=subscription-secret'
PUBLICATION advanced_publication
WITH (
    connect = false,
    create_slot = false,
    enabled = false,
    slot_name = NONE,
    streaming = parallel,
    binary = true,
    two_phase = true,
    disable_on_error = true,
    password_required = false,
    run_as_owner = true,
    failover = true,
    synchronous_commit = 'remote_apply',
    origin = 'none'
);
COMMENT ON SUBSCRIPTION advanced_subscription IS
    'Disconnected fixture subscription';

ALTER SUBSCRIPTION advanced_subscription SKIP (lsn = '0/16B6C50');
