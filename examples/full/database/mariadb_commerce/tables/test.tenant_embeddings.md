# `test.tenant_embeddings`

Bitemporal tenant embeddings

## Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `tenant_id` | `bigint(20) unsigned` | no | - |  |
| `valid_from` | `date` | no | - |  |
| `valid_to` | `date` | no | - |  |
| `embedding` | `vector(5)` | no | - |  |
| `row_start` | `timestamp(6)` | no | - | `STORED GENERATED`; stored generated as `ROW START`; system-time period start |
| `row_end` | `timestamp(6)` | no | - | `STORED GENERATED`; stored generated as `ROW END`; system-time period end |


## Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| `tenant_validity_uq` | `unique` | `tenant_id, row_end, valid_to, valid_from` | period `validity` without overlaps |
| `validity` | `check` | `` | `` `valid_from` < `valid_to` ``; declared at table level |


## Indexes

| Name | Terms | Unique | Origin / details | Predicate |
|---|---|---|---|---|
| `embedding_vector_idx` | `embedding` | no | VECTOR; M=8; distance=cosine | - |
| `tenant_validity_uq` | `tenant_id, row_end, valid_to, valid_from` | yes | BTREE; period validity without overlaps | - |


## MariaDB

**Engine:** `InnoDB`

**Row format:** `Dynamic`

**Collation:** `utf8mb4_uca1400_ai_ci`

**System versioning:** enabled

**System-time period:** `row_start, row_end`

**Application-time period:** `validity`: `valid_from` to `valid_to`

```sql
CREATE TABLE `tenant_embeddings` (
  `tenant_id` bigint(20) unsigned NOT NULL,
  `valid_from` date NOT NULL,
  `valid_to` date NOT NULL,
  `embedding` vector(5) NOT NULL,
  `row_start` timestamp(6) GENERATED ALWAYS AS ROW START,
  `row_end` timestamp(6) GENERATED ALWAYS AS ROW END,
  PERIOD FOR `validity` (`valid_from`, `valid_to`),
  UNIQUE KEY `tenant_validity_uq` (`tenant_id`,`row_end`,`validity` WITHOUT OVERLAPS),
  VECTOR KEY `embedding_vector_idx` (`embedding`) `M`=8 `DISTANCE`=cosine,
  PERIOD FOR SYSTEM_TIME (`row_start`, `row_end`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_uca1400_ai_ci COMMENT='Bitemporal tenant embeddings' WITH SYSTEM VERSIONING
```
