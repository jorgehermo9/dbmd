# `audit.partitioned_events`

## Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `id` | `bigint` | no | - | storage `plain` |
| `occurred_on` | `date` | no | - | storage `plain` |


## Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| `partitioned_events_id_not_null` | `not_null` | `id` | `NOT NULL id` |
| `partitioned_events_occurred_on_not_null` | `not_null` | `occurred_on` | `NOT NULL occurred_on` |


## PostgreSQL

**Kind:** `partitioned_table`

**Owner:** `dbmd`

**Persistence:** `permanent`

**Replica identity:** `default`

**Partition key:** `RANGE (occurred_on)`

