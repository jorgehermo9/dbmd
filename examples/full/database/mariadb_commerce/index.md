# Database: `MariaDB commerce`

Source: `mariadb_commerce`

Backend: `mariadb`

## Schemas

| Name | Details |
|---|---|
| `test` | Default character set `utf8mb4`; collation `utf8mb4_uca1400_ai_ci`. Commerce schema fixture |


## Tables

- [`test.accounts`](tables/test.accounts.md)
- [`test.discarded_events`](tables/test.discarded_events.md)
- [`test.monthly_metrics`](tables/test.monthly_metrics.md)
- [`test.tenants`](tables/test.tenants.md)
- [`test.tenant_audits`](tables/test.tenant_audits.md)
- [`test.tenant_embeddings`](tables/test.tenant_embeddings.md)


## Views

- [`test.active_accounts`](views/test.active_accounts.md)


## Triggers

- [`test.accounts_changed`](triggers/test.accounts_changed.md)
- [`test.accounts_updated`](triggers/test.accounts_updated.md)
- [`test.accounts_update_marker`](triggers/test.accounts_update_marker.md)


## Routines, Sequences, and Events

- [`test.disable_account`](objects/test.disable_account.md)
- [`test.next_account_id`](objects/test.next_account_id.md)
- [`test.normalize_email`](objects/test.normalize_email.md)
- [`test.descending_order_seq`](objects/test.descending_order_seq.md)
- [`test.order_number_seq`](objects/test.order_number_seq.md)
- [`test.archive_accounts_once`](objects/test.archive_accounts_once.md)
- [`test.purge_disabled_accounts`](objects/test.purge_disabled_accounts.md)
- [`analytics_remote`](objects/server.analytics_remote.md)
- [`Aria (storage engine)`](objects/plugin.Aria%20%28storage%20engine%29.md)
- [`associative_array (data type)`](objects/plugin.associative_array%20%28data%20type%29.md)
- [`binlog (daemon)`](objects/plugin.binlog%20%28daemon%29.md)
- [`BLACKHOLE (storage engine)`](objects/plugin.BLACKHOLE%20%28storage%20engine%29.md)
- [`caching_sha2_password (authentication)`](objects/plugin.caching_sha2_password%20%28authentication%29.md)
- [`CLIENT_STATISTICS (information schema)`](objects/plugin.CLIENT_STATISTICS%20%28information%20schema%29.md)
- [`CSV (storage engine)`](objects/plugin.CSV%20%28storage%20engine%29.md)
- [`FEEDBACK (information schema)`](objects/plugin.FEEDBACK%20%28information%20schema%29.md)
- [`GEOMETRY_COLUMNS (information schema)`](objects/plugin.GEOMETRY_COLUMNS%20%28information%20schema%29.md)
- [`INDEX_STATISTICS (information schema)`](objects/plugin.INDEX_STATISTICS%20%28information%20schema%29.md)
- [`inet4 (data type)`](objects/plugin.inet4%20%28data%20type%29.md)
- [`inet6 (data type)`](objects/plugin.inet6%20%28data%20type%29.md)
- [`inet6_aton (native function)`](objects/plugin.inet6_aton%20%28native%20function%29.md)
- [`inet6_ntoa (native function)`](objects/plugin.inet6_ntoa%20%28native%20function%29.md)
- [`inet_aton (native function)`](objects/plugin.inet_aton%20%28native%20function%29.md)
- [`inet_ntoa (native function)`](objects/plugin.inet_ntoa%20%28native%20function%29.md)
- [`InnoDB (storage engine)`](objects/plugin.InnoDB%20%28storage%20engine%29.md)
- [`INNODB_BUFFER_PAGE (information schema)`](objects/plugin.INNODB_BUFFER_PAGE%20%28information%20schema%29.md)
- [`INNODB_BUFFER_PAGE_LRU (information schema)`](objects/plugin.INNODB_BUFFER_PAGE_LRU%20%28information%20schema%29.md)
- [`INNODB_BUFFER_POOL_STATS (information schema)`](objects/plugin.INNODB_BUFFER_POOL_STATS%20%28information%20schema%29.md)
- [`INNODB_CMP (information schema)`](objects/plugin.INNODB_CMP%20%28information%20schema%29.md)
- [`INNODB_CMPMEM (information schema)`](objects/plugin.INNODB_CMPMEM%20%28information%20schema%29.md)
- [`INNODB_CMPMEM_RESET (information schema)`](objects/plugin.INNODB_CMPMEM_RESET%20%28information%20schema%29.md)
- [`INNODB_CMP_PER_INDEX (information schema)`](objects/plugin.INNODB_CMP_PER_INDEX%20%28information%20schema%29.md)
- [`INNODB_CMP_PER_INDEX_RESET (information schema)`](objects/plugin.INNODB_CMP_PER_INDEX_RESET%20%28information%20schema%29.md)
- [`INNODB_CMP_RESET (information schema)`](objects/plugin.INNODB_CMP_RESET%20%28information%20schema%29.md)
- [`INNODB_FT_BEING_DELETED (information schema)`](objects/plugin.INNODB_FT_BEING_DELETED%20%28information%20schema%29.md)
- [`INNODB_FT_CONFIG (information schema)`](objects/plugin.INNODB_FT_CONFIG%20%28information%20schema%29.md)
- [`INNODB_FT_DEFAULT_STOPWORD (information schema)`](objects/plugin.INNODB_FT_DEFAULT_STOPWORD%20%28information%20schema%29.md)
- [`INNODB_FT_DELETED (information schema)`](objects/plugin.INNODB_FT_DELETED%20%28information%20schema%29.md)
- [`INNODB_FT_INDEX_CACHE (information schema)`](objects/plugin.INNODB_FT_INDEX_CACHE%20%28information%20schema%29.md)
- [`INNODB_FT_INDEX_TABLE (information schema)`](objects/plugin.INNODB_FT_INDEX_TABLE%20%28information%20schema%29.md)
- [`INNODB_LOCKS (information schema)`](objects/plugin.INNODB_LOCKS%20%28information%20schema%29.md)
- [`INNODB_LOCK_WAITS (information schema)`](objects/plugin.INNODB_LOCK_WAITS%20%28information%20schema%29.md)
- [`INNODB_METRICS (information schema)`](objects/plugin.INNODB_METRICS%20%28information%20schema%29.md)
- [`INNODB_SYS_COLUMNS (information schema)`](objects/plugin.INNODB_SYS_COLUMNS%20%28information%20schema%29.md)
- [`INNODB_SYS_FIELDS (information schema)`](objects/plugin.INNODB_SYS_FIELDS%20%28information%20schema%29.md)
- [`INNODB_SYS_FOREIGN (information schema)`](objects/plugin.INNODB_SYS_FOREIGN%20%28information%20schema%29.md)
- [`INNODB_SYS_FOREIGN_COLS (information schema)`](objects/plugin.INNODB_SYS_FOREIGN_COLS%20%28information%20schema%29.md)
- [`INNODB_SYS_INDEXES (information schema)`](objects/plugin.INNODB_SYS_INDEXES%20%28information%20schema%29.md)
- [`INNODB_SYS_TABLES (information schema)`](objects/plugin.INNODB_SYS_TABLES%20%28information%20schema%29.md)
- [`INNODB_SYS_TABLESPACES (information schema)`](objects/plugin.INNODB_SYS_TABLESPACES%20%28information%20schema%29.md)
- [`INNODB_SYS_TABLESTATS (information schema)`](objects/plugin.INNODB_SYS_TABLESTATS%20%28information%20schema%29.md)
- [`INNODB_SYS_VIRTUAL (information schema)`](objects/plugin.INNODB_SYS_VIRTUAL%20%28information%20schema%29.md)
- [`INNODB_TABLESPACES_ENCRYPTION (information schema)`](objects/plugin.INNODB_TABLESPACES_ENCRYPTION%20%28information%20schema%29.md)
- [`INNODB_TRX (information schema)`](objects/plugin.INNODB_TRX%20%28information%20schema%29.md)
- [`is_ipv4 (native function)`](objects/plugin.is_ipv4%20%28native%20function%29.md)
- [`is_ipv4_compat (native function)`](objects/plugin.is_ipv4_compat%20%28native%20function%29.md)
- [`is_ipv4_mapped (native function)`](objects/plugin.is_ipv4_mapped%20%28native%20function%29.md)
- [`is_ipv6 (native function)`](objects/plugin.is_ipv6%20%28native%20function%29.md)
- [`MEMORY (storage engine)`](objects/plugin.MEMORY%20%28storage%20engine%29.md)
- [`mhnsw (daemon)`](objects/plugin.mhnsw%20%28daemon%29.md)
- [`MRG_MyISAM (storage engine)`](objects/plugin.MRG_MyISAM%20%28storage%20engine%29.md)
- [`MyISAM (storage engine)`](objects/plugin.MyISAM%20%28storage%20engine%29.md)
- [`mysql_native_password (authentication)`](objects/plugin.mysql_native_password%20%28authentication%29.md)
- [`mysql_old_password (authentication)`](objects/plugin.mysql_old_password%20%28authentication%29.md)
- [`online_alter_log (daemon)`](objects/plugin.online_alter_log%20%28daemon%29.md)
- [`partition (storage engine)`](objects/plugin.partition%20%28storage%20engine%29.md)
- [`PERFORMANCE_SCHEMA (storage engine)`](objects/plugin.PERFORMANCE_SCHEMA%20%28storage%20engine%29.md)
- [`SEQUENCE (storage engine)`](objects/plugin.SEQUENCE%20%28storage%20engine%29.md)
- [`SPATIAL_REF_SYS (information schema)`](objects/plugin.SPATIAL_REF_SYS%20%28information%20schema%29.md)
- [`SQL_SEQUENCE (storage engine)`](objects/plugin.SQL_SEQUENCE%20%28storage%20engine%29.md)
- [`sys_guid (native function)`](objects/plugin.sys_guid%20%28native%20function%29.md)
- [`sys_refcursor (data type)`](objects/plugin.sys_refcursor%20%28data%20type%29.md)
- [`TABLE_STATISTICS (information schema)`](objects/plugin.TABLE_STATISTICS%20%28information%20schema%29.md)
- [`THREAD_POOL_GROUPS (information schema)`](objects/plugin.THREAD_POOL_GROUPS%20%28information%20schema%29.md)
- [`THREAD_POOL_QUEUES (information schema)`](objects/plugin.THREAD_POOL_QUEUES%20%28information%20schema%29.md)
- [`THREAD_POOL_STATS (information schema)`](objects/plugin.THREAD_POOL_STATS%20%28information%20schema%29.md)
- [`THREAD_POOL_WAITS (information schema)`](objects/plugin.THREAD_POOL_WAITS%20%28information%20schema%29.md)
- [`unix_socket (authentication)`](objects/plugin.unix_socket%20%28authentication%29.md)
- [`USER_STATISTICS (information schema)`](objects/plugin.USER_STATISTICS%20%28information%20schema%29.md)
- [`user_variables (information schema)`](objects/plugin.user_variables%20%28information%20schema%29.md)
- [`uuid (data type)`](objects/plugin.uuid%20%28data%20type%29.md)
- [`uuid (native function)`](objects/plugin.uuid%20%28native%20function%29.md)
- [`uuid_v4 (native function)`](objects/plugin.uuid_v4%20%28native%20function%29.md)
- [`uuid_v7 (native function)`](objects/plugin.uuid_v7%20%28native%20function%29.md)
- [`wsrep (replication)`](objects/plugin.wsrep%20%28replication%29.md)
- [`WSREP_MEMBERSHIP (information schema)`](objects/plugin.WSREP_MEMBERSHIP%20%28information%20schema%29.md)
- [`wsrep_provider (replication)`](objects/plugin.wsrep_provider%20%28replication%29.md)
- [`WSREP_STATUS (information schema)`](objects/plugin.WSREP_STATUS%20%28information%20schema%29.md)
- [`xmltype (data type)`](objects/plugin.xmltype%20%28data%20type%29.md)
- [`test.analytics_tools`](objects/test.analytics_tools.md)
- [`analytics_reader@`](objects/account.analytics_reader%40.md)
- [`analytics_service@localhost`](objects/account.analytics_service%40localhost.md)
- [`healthcheck@127.0.0.1`](objects/account.healthcheck%40127%2E0%2E0%2E1.md)
- [`healthcheck@::1`](objects/account.healthcheck%40%3A%3A1.md)
- [`healthcheck@localhost`](objects/account.healthcheck%40localhost.md)
- [`mariadb.sys@localhost`](objects/account.mariadb%2Esys%40localhost.md)
- [`proxy_target@localhost`](objects/account.proxy_target%40localhost.md)
- [`root@%`](objects/account.root%40%25.md)
- [`root@localhost`](objects/account.root%40localhost.md)


