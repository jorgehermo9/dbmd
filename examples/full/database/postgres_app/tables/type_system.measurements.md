# `type_system.measurements`

## Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `token` | `type_system.scalar_token` | no | - | storage `plain` |
| `accepted` | `type_system.measurement_range` | no | - | storage `extended` |
| `historical` | `type_system.measurement_ranges` | no | - | storage `extended` |


## Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| `measurements_accepted_not_null` | `not_null` | `accepted` | `NOT NULL accepted` |
| `measurements_historical_not_null` | `not_null` | `historical` | `NOT NULL historical` |
| `measurements_token_not_null` | `not_null` | `token` | `NOT NULL token` |


## PostgreSQL

**Kind:** `table`

**Owner:** `dbmd`

**Persistence:** `permanent`

**Replica identity:** `default`

**Access method:** `heap`

