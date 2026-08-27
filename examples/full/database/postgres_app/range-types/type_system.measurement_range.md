# `type_system.measurement_range`

One continuous measurement interval

**Kind:** `range`

**Owner:** `dbmd`

**Subtype:** `double precision`

**Subtype operator class:** `pg_catalog.float8_ops`

**Multirange:** `type_system.measurement_ranges`

**Multirange owner:** `dbmd`

**Subtype difference:** `pg_catalog.float8mi`

**Multirange comment:** Disjoint measurement intervals

```sql
CREATE TYPE "type_system"."measurement_range" AS RANGE (
    SUBTYPE = double precision,
    SUBTYPE_OPCLASS = pg_catalog.float8_ops,
    SUBTYPE_DIFF = pg_catalog.float8mi,
    MULTIRANGE_TYPE_NAME = "type_system"."measurement_ranges"
);
```
