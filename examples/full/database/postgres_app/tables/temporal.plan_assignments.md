# `temporal.plan_assignments`

## Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `plan_id` | `bigint` | no | - | storage `plain` |
| `valid_at` | `daterange` | no | - | storage `extended` |


## Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| `assignments_plan_period` | `foreign_key` | `plan_id, valid_at` | `FOREIGN KEY (plan_id, PERIOD valid_at) REFERENCES temporal.plan_versions(plan_id, PERIOD valid_at) NOT ENFORCED`; not validated; not enforced; temporal; no inherit |
| `plan_assignments_plan_id_not_null` | `not_null` | `plan_id` | `NOT NULL plan_id` |
| `plan_assignments_valid_at_not_null` | `not_null` | `valid_at` | `NOT NULL valid_at` |


## PostgreSQL

**Kind:** `table`

**Owner:** `dbmd`

**Persistence:** `permanent`

**Replica identity:** `default`

**Access method:** `heap`

