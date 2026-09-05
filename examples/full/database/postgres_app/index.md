# Database: `PostgreSQL application`

Source: `postgres_app`

Backend: `postgres`

## Database

- [`dbmd`](database/database.dbmd.md)

## Namespaces

| Name | Owner | Comment |
|---|---|---|
| `advanced` | `dbmd` | - |
| `aggregates` | `dbmd` | - |
| `audit` | `dbmd` | - |
| `automation` | `dbmd` | - |
| `billing` | `dbmd` | - |
| `catalog` | `dbmd` | - |
| `infrastructure` | `dbmd` | - |
| `public` | `pg_database_owner` | standard public schema |
| `routines` | `dbmd` | - |
| `search` | `dbmd` | - |
| `secure` | `dbmd_acl_owner` | - |
| `storage` | `dbmd` | - |
| `temporal` | `dbmd` | - |
| `tenancy` | `dbmd` | - |
| `type_system` | `dbmd` | - |


## Enum Types

- [`catalog.account_state`](enums/catalog.account_state.md)
- [`infrastructure.label_a`](enums/infrastructure.label_a.md)
- [`infrastructure.label_b`](enums/infrastructure.label_b.md)
- [`secure.event_state`](enums/secure.event_state.md)


## Composite Types

- [`storage.device_row`](composite-types/storage.device_row.md)


## Domains

- [`secure.event_code`](domains/secure.event_code.md)


## Base and Shell Types

- [`infrastructure.label_c`](base-types/infrastructure.label_c.md)
- [`type_system.pending_value`](base-types/type_system.pending_value.md)
- [`type_system.scalar_token`](base-types/type_system.scalar_token.md)


## Range and Multirange Types

- [`type_system.measurement_range`](range-types/type_system.measurement_range.md)


## Sequences

- [`audit.accounts_id_seq`](sequences/audit.accounts_id_seq.md)
- [`automation.invoice_number`](sequences/automation.invoice_number.md)
- [`catalog.accounts_id_seq`](sequences/catalog.accounts_id_seq.md)
- [`secure.event_sequence`](sequences/secure.event_sequence.md)


## Tables

- [`advanced.deleted_orders`](tables/advanced.deleted_orders.md)
- [`advanced.orders`](tables/advanced.orders.md)
- [`audit.account_limits`](tables/audit.account_limits.md)
- [`audit.accounts`](tables/audit.accounts.md)
- [`audit.partitioned_events`](tables/audit.partitioned_events.md)
- [`audit.partitioned_events_2026`](tables/audit.partitioned_events_2026.md)
- [`automation.invoices`](tables/automation.invoices.md)
- [`billing.accounts`](tables/billing.accounts.md)
- [`billing.invoices`](tables/billing.invoices.md)
- [`catalog.accounts`](tables/catalog.accounts.md)
- [`search.documents`](tables/search.documents.md)
- [`secure.events`](tables/secure.events.md)
- [`secure.remote_events`](tables/secure.remote_events.md)
- [`storage.event_payloads`](tables/storage.event_payloads.md)
- [`storage.remote_events`](tables/storage.remote_events.md)
- [`storage.typed_devices`](tables/storage.typed_devices.md)
- [`temporal.accounts`](tables/temporal.accounts.md)
- [`temporal.plan_assignments`](tables/temporal.plan_assignments.md)
- [`temporal.plan_versions`](tables/temporal.plan_versions.md)
- [`tenancy.base_events`](tables/tenancy.base_events.md)
- [`tenancy.events`](tables/tenancy.events.md)
- [`tenancy.events_2025`](tables/tenancy.events_2025.md)
- [`tenancy.special_events`](tables/tenancy.special_events.md)
- [`type_system.measurements`](tables/type_system.measurements.md)


## Views

- [`audit.account_emails`](views/audit.account_emails.md)
- [`catalog.active_accounts`](views/catalog.active_accounts.md)
- [`secure.event_rollup`](views/secure.event_rollup.md)
- [`secure.event_view`](views/secure.event_view.md)


## Triggers

- [`audit.account_emails.account_emails_write`](triggers/audit.account_emails%2Eaccount_emails_write.md)
- [`audit.accounts.accounts_balance_constraint`](triggers/audit.accounts%2Eaccounts_balance_constraint.md)
- [`audit.accounts.accounts_transition`](triggers/audit.accounts%2Eaccounts_transition.md)
- [`audit.accounts.accounts_truncate`](triggers/audit.accounts%2Eaccounts_truncate.md)
- [`audit.accounts.zz_accounts_change`](triggers/audit.accounts%2Ezz_accounts_change.md)
- [`audit.partitioned_events.partitioned_events_change`](triggers/audit.partitioned_events%2Epartitioned_events_change.md)
- [`audit.partitioned_events_2026.partitioned_events_change`](triggers/audit.partitioned_events_2026%2Epartitioned_events_change.md)


## Functions

- [`advanced.capture_schema_change()`](functions/advanced.capture_schema_change%28%29.md)
- [`aggregates.collect_integer(state integer[], value integer)`](functions/aggregates.collect_integer%28state%20integer%5B%5D%2C%20value%20integer%29.md)
- [`aggregates.hypothetical_position(state integer[], hypothetical integer)`](functions/aggregates.hypothetical_position%28state%20integer%5B%5D%2C%20hypothetical%20integer%29.md)
- [`aggregates.pick_integer(state integer[], fraction double precision)`](functions/aggregates.pick_integer%28state%20integer%5B%5D%2C%20fraction%20double%20precision%29.md)
- [`aggregates.total_combine(left_state bigint, right_state bigint)`](functions/aggregates.total_combine%28left_state%20bigint%2C%20right_state%20bigint%29.md)
- [`aggregates.total_final(state bigint)`](functions/aggregates.total_final%28state%20bigint%29.md)
- [`aggregates.total_inverse(state bigint, value integer)`](functions/aggregates.total_inverse%28state%20bigint%2C%20value%20integer%29.md)
- [`aggregates.total_step(state bigint, value integer)`](functions/aggregates.total_step%28state%20bigint%2C%20value%20integer%29.md)
- [`audit.capture_row_change()`](functions/audit.capture_row_change%28%29.md)
- [`audit.capture_statement_change()`](functions/audit.capture_statement_change%28%29.md)
- [`infrastructure.fixture_btree_handler(internal)`](functions/infrastructure.fixture_btree_handler%28internal%29.md)
- [`infrastructure.label_a_to_b(infrastructure.label_a)`](functions/infrastructure.label_a_to_b%28infrastructure%2Elabel_a%29.md)
- [`infrastructure.label_c_in(cstring)`](functions/infrastructure.label_c_in%28cstring%29.md)
- [`infrastructure.label_c_out(infrastructure.label_c)`](functions/infrastructure.label_c_out%28infrastructure%2Elabel_c%29.md)
- [`infrastructure.nonzero(integer)`](functions/infrastructure.nonzero%28integer%29.md)
- [`infrastructure.same_integer(integer, integer)`](functions/infrastructure.same_integer%28integer%2C%20integer%29.md)
- [`routines.range_values(first_value integer, last_value integer)`](functions/routines.range_values%28first_value%20integer%2C%20last_value%20integer%29.md)
- [`routines.row_number_clone()`](functions/routines.row_number_clone%28%29.md)
- [`routines.starts_with(value text, prefix text)`](functions/routines.starts_with%28value%20text%2C%20prefix%20text%29.md)
- [`secure.event_count()`](functions/secure.event_count%28%29.md)
- [`type_system.scalar_token_in(cstring)`](functions/type_system.scalar_token_in%28cstring%29.md)
- [`type_system.scalar_token_out(type_system.scalar_token)`](functions/type_system.scalar_token_out%28type_system%2Escalar_token%29.md)


## Procedures

- [`infrastructure.accept_integer(IN value integer)`](procedures/infrastructure.accept_integer%28IN%20value%20integer%29.md)
- [`secure.clear_events()`](procedures/secure.clear_events%28%29.md)


## Aggregates

- [`aggregates.hypothetical_position(integer ORDER BY integer)`](aggregates/aggregates.hypothetical_position%28integer%20ORDER%20BY%20integer%29.md)
- [`aggregates.integer_total(integer)`](aggregates/aggregates.integer_total%28integer%29.md)
- [`aggregates.percentile_pick(double precision ORDER BY integer)`](aggregates/aggregates.percentile_pick%28double%20precision%20ORDER%20BY%20integer%29.md)
- [`secure.total_int(integer)`](aggregates/secure.total_int%28integer%29.md)


## Casts

- [`infrastructure.label_a AS infrastructure.label_b`](casts/casts.infrastructure%2Elabel_a%20AS%20infrastructure%2Elabel_b.md)
- [`infrastructure.label_b AS infrastructure.label_c`](casts/casts.infrastructure%2Elabel_b%20AS%20infrastructure%2Elabel_c.md)
- [`infrastructure.label_c AS integer`](casts/casts.infrastructure%2Elabel_c%20AS%20integer.md)


## Encoding Conversions

- [`infrastructure.utf8_to_latin1`](conversions/infrastructure.utf8_to_latin1.md)


## Operators

- [`infrastructure.!!(NONE, integer)`](operators/infrastructure.%21%21%28NONE%2C%20integer%29.md)
- [`infrastructure.===(integer, integer)`](operators/infrastructure.%3D%3D%3D%28integer%2C%20integer%29.md)


## Operator Families

- [`infrastructure.integer_family`](operator-families/infrastructure.btree%2Einteger_family.md)


## Operator Classes

- [`infrastructure.integer_class`](operator-classes/infrastructure.btree%2Einteger_class.md)


## Access Methods

- [`fixture_btree`](access-methods/access-methods.fixture_btree.md)


## Procedural Languages

- [`fixture_pl`](languages/languages.fixture_pl.md)


## Transforms

- [`integer FOR fixture_pl`](transforms/transforms.integer%20FOR%20fixture_pl.md)


## Rewrite Rules

- [`advanced.orders.archive_order_delete`](rules/advanced.orders%2Earchive_order_delete.md)


## Event Triggers

- [`capture_schema_change`](event-triggers/event-trigger.capture_schema_change.md)


## Extended Statistics

- [`advanced.orders_dependencies`](statistics/advanced.orders_dependencies.md)
- [`advanced.orders_expression`](statistics/advanced.orders_expression.md)


## Foreign-Data Wrappers

- [`fixture_wrapper`](foreign-data-wrappers/foreign-data-wrappers.fixture_wrapper.md)


## Foreign Servers

- [`fixture_server`](foreign-servers/foreign-servers.fixture_server.md)
- [`secure_server`](foreign-servers/foreign-servers.secure_server.md)


## User Mappings

- [`PUBLIC ON fixture_server`](user-mappings/fixture_server.PUBLIC.md)


## Text Search Parsers

- [`advanced.default_parser`](text-search-parsers/advanced.default_parser.md)


## Text Search Templates

- [`advanced.simple_template`](text-search-templates/advanced.simple_template.md)


## Text Search Dictionaries

- [`advanced.simple_dictionary`](text-search-dictionaries/advanced.simple_dictionary.md)


## Text Search Configurations

- [`advanced.search_configuration`](text-search-configurations/advanced.search_configuration.md)


## Publications

- [`advanced_publication`](publications/publications.advanced_publication.md)
- [`all_tables`](publications/publications.all_tables.md)
- [`temporal_changes`](publications/publications.temporal_changes.md)
- [`temporal_schema`](publications/publications.temporal_schema.md)


## Subscriptions

- [`advanced_subscription`](subscriptions/subscription.advanced_subscription.md)


## Object Privileges

- [`aggregate secure.total_int(integer) → PUBLIC EXECUTE`](privileges/aggregate.secure%2Etotal_int%28integer%29-dbmd_acl_owner-PUBLIC-EXECUTE.md)
- [`aggregate secure.total_int(integer) → dbmd_acl_owner EXECUTE`](privileges/aggregate.secure%2Etotal_int%28integer%29-dbmd_acl_owner-dbmd_acl_owner-EXECUTE.md)
- [`aggregate secure.total_int(integer) → dbmd_acl_reader EXECUTE`](privileges/aggregate.secure%2Etotal_int%28integer%29-dbmd_acl_owner-dbmd_acl_reader-EXECUTE.md)
- [`database dbmd → PUBLIC CONNECT`](privileges/database.dbmd-dbmd-PUBLIC-CONNECT.md)
- [`database dbmd → PUBLIC TEMPORARY`](privileges/database.dbmd-dbmd-PUBLIC-TEMPORARY.md)
- [`database dbmd → dbmd CONNECT`](privileges/database.dbmd-dbmd-dbmd-CONNECT.md)
- [`database dbmd → dbmd CREATE`](privileges/database.dbmd-dbmd-dbmd-CREATE.md)
- [`database dbmd → dbmd TEMPORARY`](privileges/database.dbmd-dbmd-dbmd-TEMPORARY.md)
- [`database dbmd → dbmd_acl_reader CONNECT`](privileges/database.dbmd-dbmd-dbmd_acl_reader-CONNECT.md)
- [`database dbmd → dbmd_acl_reader CREATE`](privileges/database.dbmd-dbmd-dbmd_acl_reader-CREATE.md)
- [`database dbmd → dbmd_acl_reader TEMPORARY`](privileges/database.dbmd-dbmd-dbmd_acl_reader-TEMPORARY.md)
- [`foreign table secure.remote_events → dbmd_acl_owner DELETE`](privileges/foreign%20table.secure%2Eremote_events-dbmd_acl_owner-dbmd_acl_owner-DELETE.md)
- [`foreign table secure.remote_events → dbmd_acl_owner INSERT`](privileges/foreign%20table.secure%2Eremote_events-dbmd_acl_owner-dbmd_acl_owner-INSERT.md)
- [`foreign table secure.remote_events → dbmd_acl_owner MAINTAIN`](privileges/foreign%20table.secure%2Eremote_events-dbmd_acl_owner-dbmd_acl_owner-MAINTAIN.md)
- [`foreign table secure.remote_events → dbmd_acl_owner REFERENCES`](privileges/foreign%20table.secure%2Eremote_events-dbmd_acl_owner-dbmd_acl_owner-REFERENCES.md)
- [`foreign table secure.remote_events → dbmd_acl_owner SELECT`](privileges/foreign%20table.secure%2Eremote_events-dbmd_acl_owner-dbmd_acl_owner-SELECT.md)
- [`foreign table secure.remote_events → dbmd_acl_owner TRIGGER`](privileges/foreign%20table.secure%2Eremote_events-dbmd_acl_owner-dbmd_acl_owner-TRIGGER.md)
- [`foreign table secure.remote_events → dbmd_acl_owner TRUNCATE`](privileges/foreign%20table.secure%2Eremote_events-dbmd_acl_owner-dbmd_acl_owner-TRUNCATE.md)
- [`foreign table secure.remote_events → dbmd_acl_owner UPDATE`](privileges/foreign%20table.secure%2Eremote_events-dbmd_acl_owner-dbmd_acl_owner-UPDATE.md)
- [`foreign table secure.remote_events → dbmd_acl_reader SELECT`](privileges/foreign%20table.secure%2Eremote_events-dbmd_acl_owner-dbmd_acl_reader-SELECT.md)
- [`foreign-data wrapper postgres_fdw → dbmd USAGE`](privileges/foreign-data%20wrapper.postgres_fdw-dbmd-dbmd-USAGE.md)
- [`foreign-data wrapper postgres_fdw → dbmd_acl_reader USAGE`](privileges/foreign-data%20wrapper.postgres_fdw-dbmd-dbmd_acl_reader-USAGE.md)
- [`function secure.event_count() → PUBLIC EXECUTE`](privileges/function.secure%2Eevent_count%28%29-dbmd_acl_owner-PUBLIC-EXECUTE.md)
- [`function secure.event_count() → dbmd_acl_owner EXECUTE`](privileges/function.secure%2Eevent_count%28%29-dbmd_acl_owner-dbmd_acl_owner-EXECUTE.md)
- [`function secure.event_count() → dbmd_acl_reader EXECUTE`](privileges/function.secure%2Eevent_count%28%29-dbmd_acl_owner-dbmd_acl_reader-EXECUTE.md)
- [`language plpgsql → PUBLIC USAGE`](privileges/language.plpgsql-dbmd-PUBLIC-USAGE.md)
- [`language plpgsql → dbmd USAGE`](privileges/language.plpgsql-dbmd-dbmd-USAGE.md)
- [`language plpgsql → dbmd_acl_reader USAGE`](privileges/language.plpgsql-dbmd-dbmd_acl_reader-USAGE.md)
- [`large object 424242 → dbmd_acl_owner SELECT`](privileges/large%20object.424242-dbmd_acl_owner-dbmd_acl_owner-SELECT.md)
- [`large object 424242 → dbmd_acl_owner UPDATE`](privileges/large%20object.424242-dbmd_acl_owner-dbmd_acl_owner-UPDATE.md)
- [`large object 424242 → dbmd_acl_reader SELECT`](privileges/large%20object.424242-dbmd_acl_owner-dbmd_acl_reader-SELECT.md)
- [`large object 424242 → dbmd_acl_reader UPDATE`](privileges/large%20object.424242-dbmd_acl_owner-dbmd_acl_reader-UPDATE.md)
- [`materialized view secure.event_rollup → dbmd_acl_owner DELETE`](privileges/materialized%20view.secure%2Eevent_rollup-dbmd_acl_owner-dbmd_acl_owner-DELETE.md)
- [`materialized view secure.event_rollup → dbmd_acl_owner INSERT`](privileges/materialized%20view.secure%2Eevent_rollup-dbmd_acl_owner-dbmd_acl_owner-INSERT.md)
- [`materialized view secure.event_rollup → dbmd_acl_owner MAINTAIN`](privileges/materialized%20view.secure%2Eevent_rollup-dbmd_acl_owner-dbmd_acl_owner-MAINTAIN.md)
- [`materialized view secure.event_rollup → dbmd_acl_owner REFERENCES`](privileges/materialized%20view.secure%2Eevent_rollup-dbmd_acl_owner-dbmd_acl_owner-REFERENCES.md)
- [`materialized view secure.event_rollup → dbmd_acl_owner SELECT`](privileges/materialized%20view.secure%2Eevent_rollup-dbmd_acl_owner-dbmd_acl_owner-SELECT.md)
- [`materialized view secure.event_rollup → dbmd_acl_owner TRIGGER`](privileges/materialized%20view.secure%2Eevent_rollup-dbmd_acl_owner-dbmd_acl_owner-TRIGGER.md)
- [`materialized view secure.event_rollup → dbmd_acl_owner TRUNCATE`](privileges/materialized%20view.secure%2Eevent_rollup-dbmd_acl_owner-dbmd_acl_owner-TRUNCATE.md)
- [`materialized view secure.event_rollup → dbmd_acl_owner UPDATE`](privileges/materialized%20view.secure%2Eevent_rollup-dbmd_acl_owner-dbmd_acl_owner-UPDATE.md)
- [`materialized view secure.event_rollup → dbmd_acl_reader SELECT`](privileges/materialized%20view.secure%2Eevent_rollup-dbmd_acl_owner-dbmd_acl_reader-SELECT.md)
- [`parameter statement_timeout → dbmd ALTER SYSTEM`](privileges/parameter.statement_timeout-dbmd-dbmd-ALTER%20SYSTEM.md)
- [`parameter statement_timeout → dbmd SET`](privileges/parameter.statement_timeout-dbmd-dbmd-SET.md)
- [`parameter statement_timeout → dbmd_acl_reader ALTER SYSTEM`](privileges/parameter.statement_timeout-dbmd-dbmd_acl_reader-ALTER%20SYSTEM.md)
- [`parameter work_mem → dbmd ALTER SYSTEM`](privileges/parameter.work_mem-dbmd-dbmd-ALTER%20SYSTEM.md)
- [`parameter work_mem → dbmd SET`](privileges/parameter.work_mem-dbmd-dbmd-SET.md)
- [`parameter work_mem → dbmd_acl_reader SET`](privileges/parameter.work_mem-dbmd-dbmd_acl_reader-SET.md)
- [`procedure secure.clear_events() → PUBLIC EXECUTE`](privileges/procedure.secure%2Eclear_events%28%29-dbmd_acl_owner-PUBLIC-EXECUTE.md)
- [`procedure secure.clear_events() → dbmd_acl_owner EXECUTE`](privileges/procedure.secure%2Eclear_events%28%29-dbmd_acl_owner-dbmd_acl_owner-EXECUTE.md)
- [`procedure secure.clear_events() → dbmd_acl_reader EXECUTE`](privileges/procedure.secure%2Eclear_events%28%29-dbmd_acl_owner-dbmd_acl_reader-EXECUTE.md)
- [`schema public → PUBLIC USAGE`](privileges/schema.public-pg_database_owner-PUBLIC-USAGE.md)
- [`schema public → pg_database_owner CREATE`](privileges/schema.public-pg_database_owner-pg_database_owner-CREATE.md)
- [`schema public → pg_database_owner USAGE`](privileges/schema.public-pg_database_owner-pg_database_owner-USAGE.md)
- [`schema secure → dbmd_acl_owner CREATE`](privileges/schema.secure-dbmd_acl_owner-dbmd_acl_owner-CREATE.md)
- [`schema secure → dbmd_acl_owner USAGE`](privileges/schema.secure-dbmd_acl_owner-dbmd_acl_owner-USAGE.md)
- [`schema secure → dbmd_acl_reader USAGE`](privileges/schema.secure-dbmd_acl_owner-dbmd_acl_reader-USAGE.md)
- [`sequence secure.event_sequence → dbmd_acl_owner SELECT`](privileges/sequence.secure%2Eevent_sequence-dbmd_acl_owner-dbmd_acl_owner-SELECT.md)
- [`sequence secure.event_sequence → dbmd_acl_owner UPDATE`](privileges/sequence.secure%2Eevent_sequence-dbmd_acl_owner-dbmd_acl_owner-UPDATE.md)
- [`sequence secure.event_sequence → dbmd_acl_owner USAGE`](privileges/sequence.secure%2Eevent_sequence-dbmd_acl_owner-dbmd_acl_owner-USAGE.md)
- [`sequence secure.event_sequence → dbmd_acl_reader SELECT`](privileges/sequence.secure%2Eevent_sequence-dbmd_acl_owner-dbmd_acl_reader-SELECT.md)
- [`sequence secure.event_sequence → dbmd_acl_reader UPDATE`](privileges/sequence.secure%2Eevent_sequence-dbmd_acl_owner-dbmd_acl_reader-UPDATE.md)
- [`sequence secure.event_sequence → dbmd_acl_reader USAGE`](privileges/sequence.secure%2Eevent_sequence-dbmd_acl_owner-dbmd_acl_reader-USAGE.md)
- [`foreign server secure_server → dbmd_acl_owner USAGE`](privileges/foreign%20server.secure_server-dbmd_acl_owner-dbmd_acl_owner-USAGE.md)
- [`foreign server secure_server → dbmd_acl_reader USAGE`](privileges/foreign%20server.secure_server-dbmd_acl_owner-dbmd_acl_reader-USAGE.md)
- [`table secure.events → dbmd_acl_owner DELETE`](privileges/table.secure%2Eevents-dbmd_acl_owner-dbmd_acl_owner-DELETE.md)
- [`table secure.events → dbmd_acl_owner INSERT`](privileges/table.secure%2Eevents-dbmd_acl_owner-dbmd_acl_owner-INSERT.md)
- [`table secure.events → dbmd_acl_owner MAINTAIN`](privileges/table.secure%2Eevents-dbmd_acl_owner-dbmd_acl_owner-MAINTAIN.md)
- [`table secure.events → dbmd_acl_owner REFERENCES`](privileges/table.secure%2Eevents-dbmd_acl_owner-dbmd_acl_owner-REFERENCES.md)
- [`table secure.events → dbmd_acl_owner SELECT`](privileges/table.secure%2Eevents-dbmd_acl_owner-dbmd_acl_owner-SELECT.md)
- [`table secure.events → dbmd_acl_owner TRIGGER`](privileges/table.secure%2Eevents-dbmd_acl_owner-dbmd_acl_owner-TRIGGER.md)
- [`table secure.events → dbmd_acl_owner TRUNCATE`](privileges/table.secure%2Eevents-dbmd_acl_owner-dbmd_acl_owner-TRUNCATE.md)
- [`table secure.events → dbmd_acl_owner UPDATE`](privileges/table.secure%2Eevents-dbmd_acl_owner-dbmd_acl_owner-UPDATE.md)
- [`table secure.events → dbmd_acl_reader DELETE`](privileges/table.secure%2Eevents-dbmd_acl_owner-dbmd_acl_reader-DELETE.md)
- [`table secure.events → dbmd_acl_reader INSERT`](privileges/table.secure%2Eevents-dbmd_acl_owner-dbmd_acl_reader-INSERT.md)
- [`table secure.events → dbmd_acl_reader MAINTAIN`](privileges/table.secure%2Eevents-dbmd_acl_owner-dbmd_acl_reader-MAINTAIN.md)
- [`table secure.events → dbmd_acl_reader REFERENCES`](privileges/table.secure%2Eevents-dbmd_acl_owner-dbmd_acl_reader-REFERENCES.md)
- [`table secure.events → dbmd_acl_reader SELECT`](privileges/table.secure%2Eevents-dbmd_acl_owner-dbmd_acl_reader-SELECT.md)
- [`table secure.events → dbmd_acl_reader TRIGGER`](privileges/table.secure%2Eevents-dbmd_acl_owner-dbmd_acl_reader-TRIGGER.md)
- [`table secure.events → dbmd_acl_reader TRUNCATE`](privileges/table.secure%2Eevents-dbmd_acl_owner-dbmd_acl_reader-TRUNCATE.md)
- [`table secure.events → dbmd_acl_reader UPDATE`](privileges/table.secure%2Eevents-dbmd_acl_owner-dbmd_acl_reader-UPDATE.md)
- [`table column secure.events.payload → dbmd_acl_reader UPDATE`](privileges/table%20column.secure%2Eevents%2Epayload-dbmd_acl_owner-dbmd_acl_reader-UPDATE.md)
- [`type secure.event_code → PUBLIC USAGE`](privileges/type.secure%2Eevent_code-dbmd_acl_owner-PUBLIC-USAGE.md)
- [`type secure.event_code → dbmd_acl_owner USAGE`](privileges/type.secure%2Eevent_code-dbmd_acl_owner-dbmd_acl_owner-USAGE.md)
- [`type secure.event_code → dbmd_acl_reader USAGE`](privileges/type.secure%2Eevent_code-dbmd_acl_owner-dbmd_acl_reader-USAGE.md)
- [`type secure.event_state → PUBLIC USAGE`](privileges/type.secure%2Eevent_state-dbmd_acl_owner-PUBLIC-USAGE.md)
- [`type secure.event_state → dbmd_acl_owner USAGE`](privileges/type.secure%2Eevent_state-dbmd_acl_owner-dbmd_acl_owner-USAGE.md)
- [`type secure.event_state → dbmd_acl_reader USAGE`](privileges/type.secure%2Eevent_state-dbmd_acl_owner-dbmd_acl_reader-USAGE.md)
- [`view secure.event_view → dbmd_acl_owner DELETE`](privileges/view.secure%2Eevent_view-dbmd_acl_owner-dbmd_acl_owner-DELETE.md)
- [`view secure.event_view → dbmd_acl_owner INSERT`](privileges/view.secure%2Eevent_view-dbmd_acl_owner-dbmd_acl_owner-INSERT.md)
- [`view secure.event_view → dbmd_acl_owner MAINTAIN`](privileges/view.secure%2Eevent_view-dbmd_acl_owner-dbmd_acl_owner-MAINTAIN.md)
- [`view secure.event_view → dbmd_acl_owner REFERENCES`](privileges/view.secure%2Eevent_view-dbmd_acl_owner-dbmd_acl_owner-REFERENCES.md)
- [`view secure.event_view → dbmd_acl_owner SELECT`](privileges/view.secure%2Eevent_view-dbmd_acl_owner-dbmd_acl_owner-SELECT.md)
- [`view secure.event_view → dbmd_acl_owner TRIGGER`](privileges/view.secure%2Eevent_view-dbmd_acl_owner-dbmd_acl_owner-TRIGGER.md)
- [`view secure.event_view → dbmd_acl_owner TRUNCATE`](privileges/view.secure%2Eevent_view-dbmd_acl_owner-dbmd_acl_owner-TRUNCATE.md)
- [`view secure.event_view → dbmd_acl_owner UPDATE`](privileges/view.secure%2Eevent_view-dbmd_acl_owner-dbmd_acl_owner-UPDATE.md)
- [`view secure.event_view → dbmd_acl_reader SELECT`](privileges/view.secure%2Eevent_view-dbmd_acl_owner-dbmd_acl_reader-SELECT.md)


## Default Privileges

- [`dbmd_acl_owner / secure / sequences → dbmd_acl_reader USAGE`](default-privileges/default-privilege.dbmd_acl_owner-secure-sequences-dbmd_acl_reader-USAGE.md)
- [`dbmd_acl_owner / secure / types → dbmd_acl_reader USAGE`](default-privileges/default-privilege.dbmd_acl_owner-secure-types-dbmd_acl_reader-USAGE.md)
- [`dbmd_acl_owner / secure / routines → dbmd_acl_reader EXECUTE`](default-privileges/default-privilege.dbmd_acl_owner-secure-routines-dbmd_acl_reader-EXECUTE.md)
- [`dbmd_acl_owner / secure / tables → dbmd_acl_reader SELECT`](default-privileges/default-privilege.dbmd_acl_owner-secure-tables-dbmd_acl_reader-SELECT.md)
- [`dbmd_acl_owner / database-wide / large objects → dbmd_acl_owner SELECT`](default-privileges/default-privilege.dbmd_acl_owner-database-wide-large%20objects-dbmd_acl_owner-SELECT.md)
- [`dbmd_acl_owner / database-wide / large objects → dbmd_acl_owner UPDATE`](default-privileges/default-privilege.dbmd_acl_owner-database-wide-large%20objects-dbmd_acl_owner-UPDATE.md)
- [`dbmd_acl_owner / database-wide / large objects → dbmd_acl_reader SELECT`](default-privileges/default-privilege.dbmd_acl_owner-database-wide-large%20objects-dbmd_acl_reader-SELECT.md)
- [`dbmd_acl_owner / database-wide / schemas → dbmd_acl_owner CREATE`](default-privileges/default-privilege.dbmd_acl_owner-database-wide-schemas-dbmd_acl_owner-CREATE.md)
- [`dbmd_acl_owner / database-wide / schemas → dbmd_acl_owner USAGE`](default-privileges/default-privilege.dbmd_acl_owner-database-wide-schemas-dbmd_acl_owner-USAGE.md)
- [`dbmd_acl_owner / database-wide / schemas → dbmd_acl_reader USAGE`](default-privileges/default-privilege.dbmd_acl_owner-database-wide-schemas-dbmd_acl_reader-USAGE.md)


## Large Objects

- [`424242`](large-objects/large-object.424242.md)


## Collations

- [`temporal.unicode_fast`](collations/temporal.unicode_fast.md)


## Extensions

- [`btree_gist`](extensions/extensions.btree_gist.md)
- [`plpgsql`](extensions/extensions.plpgsql.md)
- [`postgres_fdw`](extensions/extensions.postgres_fdw.md)


