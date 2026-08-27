# `test.purge_disabled_accounts`

Remove old disabled accounts

**Definer:** `root@localhost`

**Type:** recurring

**Status:** disabled

**Time zone:** `SYSTEM`

**On completion:** preserve

**Schedule:** `EVERY 1 DAY STARTS 2030-01-01 00:00:00`

**SQL mode:** `ONLY_FULL_GROUP_BY,STRICT_TRANS_TABLES,NO_ZERO_IN_DATE,NO_ZERO_DATE,ERROR_FOR_DIVISION_BY_ZERO,NO_ENGINE_SUBSTITUTION`

**Originator server ID:** 1

**Character set client:** `latin1`

**Connection collation:** `latin1_swedish_ci`

**Database collation:** `utf8mb4_0900_ai_ci`

```sql
CREATE DEFINER=`root`@`localhost` EVENT `purge_disabled_accounts` ON SCHEDULE EVERY 1 DAY STARTS '2030-01-01 00:00:00' ON COMPLETION PRESERVE DISABLE COMMENT 'Remove old disabled accounts' DO DELETE FROM accounts WHERE status = 'disabled' AND updated_at < CURRENT_TIMESTAMP - INTERVAL 365 DAY
```
