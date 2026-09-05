# `warehouse.analytics.tenants`

Application tenants

## Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `tenant_id` | `BIGINT` | no | - |  |
| `name` | `VARCHAR` | no | - |  |


## Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| `tenants_tenant_id_pkey` | `primary key` | `tenant_id` | `PRIMARY KEY(tenant_id)` |
| `tenants_name_not_null` | `not null` | `name` | `NOT NULL` |
| `tenants_name_key` | `unique` | `name` | `UNIQUE("name")` |
| `tenants_tenant_id_not_null` | `not null` | `tenant_id` | `NOT NULL` |


## DuckDB

**Temporary:** no

```sql
CREATE TABLE analytics.tenants(tenant_id BIGINT PRIMARY KEY, "name" VARCHAR NOT NULL UNIQUE);
```
