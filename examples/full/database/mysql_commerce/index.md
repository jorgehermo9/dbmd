# Database: `MySQL commerce`

Source: `mysql_commerce`

Backend: `mysql`

## Schemas

| Name | Details |
|---|---|
| `test` | Default character set `utf8mb4`; collation `utf8mb4_0900_ai_ci`; encryption no; read-only no. |


## Tables

- [`test.accounts`](tables/test.accounts.md)
- [`test.generated_primary_key`](tables/test.generated_primary_key.md)
- [`test.inline_memberships`](tables/test.inline_memberships.md)
- [`test.memory_lookup`](tables/test.memory_lookup.md)
- [`test.monthly_metrics`](tables/test.monthly_metrics.md)
- [`test.tenants`](tables/test.tenants.md)


## Views

- [`test.active_accounts`](views/test.active_accounts.md)
- [`test.tenant_documents`](views/test.tenant_documents.md)


## Triggers

- [`test.accounts_updated`](triggers/test.accounts_updated.md)
- [`test.accounts_update_marker`](triggers/test.accounts_update_marker.md)


## Routines

- [`test.disable_account`](routines/test.disable_account.md)
- [`test.next_account_id`](routines/test.next_account_id.md)
- [`test.normalize_email`](routines/test.normalize_email.md)


## Events

- [`test.archive_accounts_once`](events/test.archive_accounts_once.md)
- [`test.purge_disabled_accounts`](events/test.purge_disabled_accounts.md)


