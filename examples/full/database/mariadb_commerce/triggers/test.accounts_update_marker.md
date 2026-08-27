# `test.accounts_update_marker`

**`before` `update`** on `test.accounts`.

**Orientation:** for each row

**Order:** 2

**Definer:** `root@localhost`

**SQL mode:** `STRICT_TRANS_TABLES,ERROR_FOR_DIVISION_BY_ZERO,NO_AUTO_CREATE_USER,NO_ENGINE_SUBSTITUTION`

**Character set client:** `utf8mb4`

**Connection collation:** `utf8mb4_uca1400_ai_ci`

**Database collation:** `utf8mb4_uca1400_ai_ci`

```sql
CREATE DEFINER=`root`@`localhost` TRIGGER accounts_update_marker
BEFORE UPDATE ON accounts
FOR EACH ROW
SET @dbmd_mariadb_last_account = NEW.account_id
```