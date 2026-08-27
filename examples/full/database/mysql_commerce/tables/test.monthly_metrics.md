# `test.monthly_metrics`

## Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `occurred_on` | `date` | no | - |  |
| `metric` | `varchar(64)` | no | - |  |
| `value` | `bigint` | no | - |  |


## Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| `PRIMARY` | `primary_key` | `occurred_on, metric` | - |


## Indexes

| Name | Terms | Unique | Origin / details | Predicate |
|---|---|---|---|---|
| `PRIMARY` | `occurred_on, metric` | yes | BTREE | - |


## MySQL

**Engine:** `InnoDB`

**Row format:** `Dynamic`

**Collation:** `utf8mb4_0900_ai_ci`

**Create options:** `partitioned`

**Subpartition:** `p2025_h0`: method=`HASH`, expression=``month(`occurred_on`)``, boundary=`2026`, position=1; nodegroup=`default`

**Subpartition:** `p2025_h1`: method=`HASH`, expression=``month(`occurred_on`)``, boundary=`2026`, position=2; nodegroup=`default`

**Subpartition:** `pmax_h0`: method=`HASH`, expression=``month(`occurred_on`)``, boundary=`MAXVALUE`, position=1; nodegroup=`default`

**Subpartition:** `pmax_h1`: method=`HASH`, expression=``month(`occurred_on`)``, boundary=`MAXVALUE`, position=2; nodegroup=`default`

```sql
CREATE TABLE `monthly_metrics` (
  `occurred_on` date NOT NULL,
  `metric` varchar(64) NOT NULL,
  `value` bigint NOT NULL,
  PRIMARY KEY (`occurred_on`,`metric`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci
/*!50100 PARTITION BY RANGE (year(`occurred_on`))
SUBPARTITION BY HASH (month(`occurred_on`))
(PARTITION p2025 VALUES LESS THAN (2026)
 (SUBPARTITION p2025_h0 ENGINE = InnoDB,
  SUBPARTITION p2025_h1 ENGINE = InnoDB),
 PARTITION pmax VALUES LESS THAN MAXVALUE
 (SUBPARTITION pmax_h0 ENGINE = InnoDB,
  SUBPARTITION pmax_h1 ENGINE = InnoDB)) */
```
