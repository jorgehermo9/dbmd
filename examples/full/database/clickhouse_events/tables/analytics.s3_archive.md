# `analytics.s3_archive`

## Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `payload` | `String` | no | - |  |


## ClickHouse

**Kind:** `table`

**UUID:** `20000000-0000-0000-0000-000000000015`

**Engine:** `S3('s3://dbmd-audit/archive.parquet', 'Parquet', storage_class_name = 'INTELLIGENT_TIERING')`

**Engine argument:** 1: `'s3://dbmd-audit/archive.parquet'`

**Engine argument:** 2: `'Parquet'`

**Engine parameter:** `storage_class_name` = `'INTELLIGENT_TIERING'`

```sql
CREATE TABLE analytics.s3_archive (`payload` String) ENGINE = S3('s3://dbmd-audit/archive.parquet', 'Parquet', storage_class_name = 'INTELLIGENT_TIERING')
```
