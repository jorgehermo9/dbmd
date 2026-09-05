# `temporal.plan_versions`

## Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `plan_id` | `bigint` | no | - | storage `plain` |
| `valid_at` | `daterange` | no | - | storage `extended` |


## Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| `plan_versions_identity` | `unique` | `plan_id, valid_at` | `UNIQUE (plan_id, valid_at WITHOUT OVERLAPS)`; temporal; no inherit; operators `pg_catalog.=(bigint,bigint), pg_catalog.&&(pg_catalog.anyrange,pg_catalog.anyrange)` |
| `plan_versions_plan_id_not_null` | `not_null` | `plan_id` | `NOT NULL plan_id` |
| `plan_versions_valid_at_not_null` | `not_null` | `valid_at` | `NOT NULL valid_at` |


## Indexes

| Name | Terms | Unique | Origin / details | Predicate |
|---|---|---|---|---|
| `plan_versions_identity` | `plan_id` ascending opclass `public.gist_int8_ops`, `valid_at` ascending opclass `pg_catalog.range_ops` | yes | postgres `gist`; owner `dbmd`; constraint `assignments_plan_period` | - |


## PostgreSQL

**Kind:** `table`

**Owner:** `dbmd`

**Persistence:** `permanent`

**Replica identity:** `default`

**Access method:** `heap`

