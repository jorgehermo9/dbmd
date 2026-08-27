# `test.accounts_updated`

**`before` `update`** on `test.accounts`.

**Orientation:** for each row

**Order:** 1

**Definer:** `root@localhost`

**SQL mode:** `STRICT_TRANS_TABLES,ERROR_FOR_DIVISION_BY_ZERO,NO_AUTO_CREATE_USER,NO_ENGINE_SUBSTITUTION`

**Character set client:** `utf8mb4`

**Connection collation:** `utf8mb4_uca1400_ai_ci`

**Database collation:** `utf8mb4_uca1400_ai_ci`

**Update columns:** `email, status`

```sql
CREATE DEFINER=`root`@`localhost` TRIGGER accounts_updated
BEFORE UPDATE OF status, email ON accounts
FOR EACH ROW SET NEW.status = COALESCE(NEW.status, OLD.status)
```