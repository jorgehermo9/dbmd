CREATE EXTENSION vector WITH SCHEMA public;
CREATE EXTENSION dbmd_fixture WITH SCHEMA public;

COMMENT ON EXTENSION vector IS 'Vector similarity search support';

CREATE SCHEMA app;

CREATE TABLE app.items (
    id bigint PRIMARY KEY,
    embedding vector(3) NOT NULL
);

CREATE INDEX items_embedding_hnsw_idx
    ON app.items
    USING hnsw (embedding vector_l2_ops)
    WITH (m = 8, ef_construction = 32);

CREATE INDEX items_embedding_ivfflat_idx
    ON app.items
    USING ivfflat (embedding vector_cosine_ops)
    WITH (lists = 10);
