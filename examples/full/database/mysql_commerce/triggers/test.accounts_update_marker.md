# `test.accounts_update_marker`

**`before` `update`** on `test.accounts`.

**Orientation:** for each row

**Order:** 2

**Definer:** `root@localhost`

**SQL mode:** `ONLY_FULL_GROUP_BY,STRICT_TRANS_TABLES,NO_ZERO_IN_DATE,NO_ZERO_DATE,ERROR_FOR_DIVISION_BY_ZERO,NO_ENGINE_SUBSTITUTION`

**Character set client:** `latin1`

**Connection collation:** `latin1_swedish_ci`

**Database collation:** `utf8mb4_0900_ai_ci`

```sql
CREATE DEFINER=`root`@`localhost` TRIGGER `accounts_update_marker` BEFORE UPDATE ON `accounts` FOR EACH ROW SET @dbmd_last_account = NEW.account_id
```