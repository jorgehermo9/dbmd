# `audit.partitioned_events_2026`

## Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `id` | `bigint` | no | - | storage `plain`; inherited only |
| `occurred_on` | `date` | no | - | storage `plain`; inherited only |


## Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| `partitioned_events_id_not_null` | `not_null` | `id` | `NOT NULL id`; inherited |
| `partitioned_events_occurred_on_not_null` | `not_null` | `occurred_on` | `NOT NULL occurred_on`; inherited |


## PostgreSQL

**Kind:** `partition`

**Owner:** `dbmd`

**Persistence:** `permanent`

**Replica identity:** `default`

**Access method:** `heap`

**Inherits:** `audit.partitioned_events`

**Partition parent:** `audit.partitioned_events`

**Partition bound:** `FOR VALUES FROM ('2026-01-01') TO ('2027-01-01')`

