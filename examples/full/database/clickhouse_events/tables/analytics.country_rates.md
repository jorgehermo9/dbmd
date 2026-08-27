# `analytics.country_rates`

## Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `country_id` | `UInt64` | no | - | precision 64 base 2; scale 0 |
| `valid_from` | `Date` | no | - | datetime precision 0 |
| `valid_to` | `Date` | no | - | datetime precision 0 |
| `rate` | `Float64` | no | - |  |


## ClickHouse

**Kind:** `dictionary`

**UUID:** `20000000-0000-0000-0000-000000000016`

**Loads after:** `analytics.country_source`

**Dictionary layout:** `RANGE_HASHED`

**Dictionary keys:** `country_id UInt64`

**Dictionary attributes:** `rate Float64 DEFAULT 0`

**Dictionary source:** ``

**Dictionary lifetime:** 0..0 seconds

**Dictionary range:** MIN `valid_from` MAX `valid_to`

```sql
CREATE DICTIONARY analytics.country_rates (`country_id` UInt64, `valid_from` Date, `valid_to` Date, `rate` Float64 DEFAULT 0) PRIMARY KEY country_id SOURCE(CLICKHOUSE(HOST 'localhost' PORT 9000 USER 'default' PASSWORD '[HIDDEN]' DB 'analytics' TABLE 'country_source')) LIFETIME(MIN 0 MAX 0) LAYOUT(RANGE_HASHED()) RANGE(MIN valid_from MAX valid_to)
```
