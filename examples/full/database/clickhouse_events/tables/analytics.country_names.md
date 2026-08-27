# `analytics.country_names`

## Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `country_id` | `UInt64` | no | - | precision 64 base 2; scale 0 |
| `parent_id` | `UInt64` | no | - | precision 64 base 2; scale 0 |
| `country_name` | `String` | no | - |  |
| `normalized_name` | `String` | no | - |  |


## ClickHouse

**Kind:** `dictionary`

**UUID:** `20000000-0000-0000-0000-000000000006`

**Loads after:** `analytics.country_source`

**Dictionary layout:** `HASHED`

**Dictionary keys:** `country_id UInt64 IS_OBJECT_ID`

**Dictionary attributes:** `parent_id UInt64 DEFAULT 0 HIERARCHICAL, country_name String DEFAULT 'unknown' INJECTIVE, normalized_name String EXPRESSION lowerUTF8(country_name)`

**Dictionary source:** ``

**Dictionary lifetime:** 30..60 seconds

**Dictionary setting:** `max_threads_for_updates` = `4`

```sql
CREATE DICTIONARY analytics.country_names (`country_id` UInt64 IS_OBJECT_ID, `parent_id` UInt64 DEFAULT 0 HIERARCHICAL, `country_name` String DEFAULT 'unknown' INJECTIVE, `normalized_name` String EXPRESSION lowerUTF8(country_name)) PRIMARY KEY country_id SOURCE(CLICKHOUSE(HOST 'localhost' PORT 9000 USER 'default' PASSWORD '[HIDDEN]' DB 'analytics' TABLE 'country_source')) LIFETIME(MIN 30 MAX 60) LAYOUT(HASHED()) SETTINGS(max_threads_for_updates = 4)
```
