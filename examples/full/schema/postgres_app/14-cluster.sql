CREATE ROLE dbmd_cluster_reader NOLOGIN;
COMMENT ON ROLE dbmd_cluster_reader IS 'Cluster fixture read capability';

CREATE ROLE dbmd_cluster_app
LOGIN
PASSWORD 'cluster-secret'
CONNECTION LIMIT 5
VALID UNTIL '2030-01-01 00:00:00+00';
ALTER ROLE dbmd_cluster_app SET statement_timeout = '5s';
DO $fixture$
BEGIN
    EXECUTE pg_catalog.format(
        'ALTER ROLE dbmd_cluster_app IN DATABASE %I SET lock_timeout = %L',
        pg_catalog.current_database(),
        '2s'
    );
END
$fixture$;
COMMENT ON ROLE dbmd_cluster_app IS 'Cluster fixture login role';

GRANT dbmd_cluster_reader TO dbmd_cluster_app
WITH ADMIN OPTION, INHERIT FALSE, SET TRUE;

ALTER TABLESPACE pg_default SET (random_page_cost = 1.1);
COMMENT ON TABLESPACE pg_default IS 'Cluster fixture default tablespace';
GRANT CREATE ON TABLESPACE pg_default TO dbmd_cluster_reader;
