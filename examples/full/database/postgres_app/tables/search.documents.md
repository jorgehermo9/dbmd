# `search.documents`

## Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `id` | `bigint` | no | - | storage `plain` |
| `tenant_id` | `bigint` | no | - | storage `plain` |
| `title` | `text` | no | - | collation `pg_catalog."C"`; storage `extended` |
| `body` | `text` | no | - | collation `pg_catalog."default"`; storage `extended` |
| `published` | `boolean` | no | `false` | storage `plain` |
| `active_window` | `int4range` | yes | - | storage `extended` |


## Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| `documents_active_window_exclude` | `exclusion` | `active_window` | `EXCLUDE USING gist (active_window WITH &&)`; no inherit; operators `pg_catalog.&&(pg_catalog.anyrange,pg_catalog.anyrange)` |
| `documents_body_not_null` | `not_null` | `body` | `NOT NULL body` |
| `documents_id_not_null` | `not_null` | `id` | `NOT NULL id` |
| `documents_published_not_null` | `not_null` | `published` | `NOT NULL published` |
| `documents_tenant_id_not_null` | `not_null` | `tenant_id` | `NOT NULL tenant_id` |
| `documents_title_check` | `check` | `title` | `CHECK (title <> ''::text) NOT VALID`; not validated; Rejects empty titles |
| `documents_title_not_null` | `not_null` | `title` | `NOT NULL title` |
| `documents_title_unique` | `unique` | `tenant_id, title` | `UNIQUE NULLS NOT DISTINCT (tenant_id, title) DEFERRABLE INITIALLY DEFERRED`; no inherit |


## Indexes

| Name | Terms | Unique | Origin / details | Predicate |
|---|---|---|---|---|
| `documents_active_window_exclude` | `active_window` ascending opclass `pg_catalog.range_ops` | no | postgres `gist`; owner `dbmd`; constraint `documents_active_window_exclude` | - |
| `documents_brin_idx` | `id` ascending opclass `pg_catalog.int8_bloom_ops` parameters `n_distinct_per_range=32, false_positive_rate=0.05` | no | postgres `brin`; owner `dbmd` | - |
| `documents_cluster_idx` | `id` ascending opclass `pg_catalog.int8_ops` nulls `last` | no | postgres `btree`; clustered; owner `dbmd` | - |
| `documents_lookup_idx` | `tenant_id` ascending opclass `pg_catalog.int8_ops` nulls `last`, `lower(title)` descending collate `pg_catalog."C"` opclass `pg_catalog.text_ops` nulls `first` | yes | postgres `btree`; include `body`; nulls not distinct; owner `dbmd`; option `fillfactor=75`; Published-document lookup | `published` |
| `documents_replica_idx` | `id` ascending opclass `pg_catalog.int8_ops` nulls `last` | yes | postgres `btree`; replica identity; owner `dbmd` | - |
| `documents_title_unique` | `tenant_id` ascending opclass `pg_catalog.int8_ops` nulls `last`, `title` ascending collate `pg_catalog."C"` opclass `pg_catalog.text_ops` nulls `last` | yes | postgres `btree`; nulls not distinct; owner `dbmd`; constraint `documents_title_unique` | - |


## PostgreSQL

**Kind:** `table`

**Owner:** `dbmd`

**Persistence:** `permanent`

**Replica identity:** `index`

**Access method:** `heap`

