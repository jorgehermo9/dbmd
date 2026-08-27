# Database: `ClickHouse events`

Source: `clickhouse_events`

Backend: `clickhouse`

## Databases

| Name | Comment |
|---|---|
| `analytics` | Engine `Atomic`; UUID `10000000-0000-0000-0000-000000000001`; Analytical application data |


## Tables

- [`analytics.country_names`](tables/analytics.country_names.md)
- [`analytics.country_rates`](tables/analytics.country_rates.md)
- [`analytics.country_source`](tables/analytics.country_source.md)
- [`analytics.event_counts`](tables/analytics.event_counts.md)
- [`analytics.events`](tables/analytics.events.md)
- [`analytics.modern_storage`](tables/analytics.modern_storage.md)
- [`analytics.refresh_rollups`](tables/analytics.refresh_rollups.md)
- [`analytics.refresh_snapshots`](tables/analytics.refresh_snapshots.md)
- [`analytics.remote_accounts`](tables/analytics.remote_accounts.md)
- [`analytics.retention_matrix`](tables/analytics.retention_matrix.md)
- [`analytics.retention_rollup`](tables/analytics.retention_rollup.md)
- [`analytics.s3_archive`](tables/analytics.s3_archive.md)
- [`analytics.window_event_counts`](tables/analytics.window_event_counts.md)


## Views

- [`analytics.active_events`](views/analytics.active_events.md)
- [`analytics.event_counts_mv`](views/analytics.event_counts_mv.md)
- [`analytics.events_by_tenant`](views/analytics.events_by_tenant.md)
- [`analytics.refresh_base`](views/analytics.refresh_base.md)
- [`analytics.refresh_dependent`](views/analytics.refresh_dependent.md)
- [`analytics.windowed_events`](views/analytics.windowed_events.md)
- [`analytics.windowed_events_owned`](views/analytics.windowed_events_owned.md)


## Functions

- [`analytics_normalize`](functions/global.analytics_normalize.md)


## Access and workload objects

- [`analytics_service`](access-and-workloads/user.analytics_service.md)
- [`analytics_reader`](access-and-workloads/role.analytics_reader.md)
- [`tenant_events ON analytics.events`](access-and-workloads/row-policy.tenant_events%20ON%20analytics%2Eevents.md)
- [`analytics_quota`](access-and-workloads/quota.analytics_quota.md)
- [`analytics_profile`](access-and-workloads/settings-profile.analytics_profile.md)
- [`analytics_remote`](access-and-workloads/named-collection.analytics_remote.md)
- [`analytics_cpu`](access-and-workloads/resource.analytics_cpu.md)
- [`analytics_all`](access-and-workloads/workload.analytics_all.md)
- [`analytics_interactive`](access-and-workloads/workload.analytics_interactive.md)


