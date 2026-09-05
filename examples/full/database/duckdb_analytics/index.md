# Database: `DuckDB analytics`

Source: `duckdb_analytics`

Backend: `duckdb`

## Schemas

| Name | Details |
|---|---|
| `warehouse.analytics` | `duckdb` catalog; read-only; catalog tags `storage_version=v1.0.0+` |


## Tables

- [`warehouse.analytics.accounts`](tables/warehouse%2Eanalytics.accounts.md)
- [`warehouse.analytics.tenants`](tables/warehouse%2Eanalytics.tenants.md)


## Views

- [`warehouse.analytics.active_accounts`](views/warehouse%2Eanalytics.active_accounts.md)


## Types, Sequences, Functions, and Extensions

- [`warehouse.analytics.account_pair`](objects/warehouse%2Eanalytics.account_pair.md)
- [`warehouse.analytics.account_status`](objects/warehouse%2Eanalytics.account_status.md)
- [`warehouse.analytics.positive_integer`](objects/warehouse%2Eanalytics.positive_integer.md)
- [`warehouse.analytics.reference_value`](objects/warehouse%2Eanalytics.reference_value.md)
- [`warehouse.analytics.account_id_seq`](objects/warehouse%2Eanalytics.account_id_seq.md)
- [`warehouse.analytics.accounts_for_tenant`](objects/warehouse%2Eanalytics.accounts_for_tenant.md)
- [`warehouse.analytics.normalize_email`](objects/warehouse%2Eanalytics.normalize_email.md)
- [`core_functions`](objects/extensions.core_functions.md)


