# `main.account_directory.account_directory_insert`

**INSTEAD OF INSERT** on `main.account_directory`.

```sql
CREATE TRIGGER account_directory_insert
INSTEAD OF INSERT ON account_directory
BEGIN
    INSERT INTO accounts (id, tenant_id, organization_slug, email)
    VALUES (NEW.account_id, 1, 'default', NEW.email);
END
```