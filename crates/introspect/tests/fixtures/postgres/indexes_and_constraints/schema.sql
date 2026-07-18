CREATE SCHEMA search;

CREATE TABLE search.documents (
    id bigint NOT NULL,
    tenant_id bigint NOT NULL,
    title text COLLATE "C" NOT NULL,
    body text NOT NULL,
    published boolean NOT NULL DEFAULT false
);

ALTER TABLE search.documents
    ADD CONSTRAINT documents_title_check
    CHECK (title <> '') NOT VALID;

ALTER TABLE search.documents
    ADD CONSTRAINT documents_title_unique
    UNIQUE NULLS NOT DISTINCT (tenant_id, title)
    DEFERRABLE INITIALLY DEFERRED;

CREATE UNIQUE INDEX documents_lookup_idx
    ON search.documents USING btree (
        tenant_id int8_ops ASC NULLS LAST,
        lower(title) COLLATE "C" text_ops DESC NULLS FIRST
    )
    INCLUDE (body)
    NULLS NOT DISTINCT
    WHERE published;

CREATE INDEX documents_cluster_idx
    ON search.documents USING btree (id);

CLUSTER search.documents USING documents_cluster_idx;

CREATE UNIQUE INDEX documents_replica_idx
    ON search.documents USING btree (id);

ALTER TABLE search.documents
    REPLICA IDENTITY USING INDEX documents_replica_idx;
