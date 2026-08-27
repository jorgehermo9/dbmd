# `test.purge_disabled_accounts`

Remove old disabled accounts

**Kind:** `event`

**Status:** disabled

**Schedule:** recurring

**Completion:** preserve

**Definer:** `root@localhost`

**Time zone:** `SYSTEM`

**SQL mode:** `STRICT_TRANS_TABLES,ERROR_FOR_DIVISION_BY_ZERO,NO_AUTO_CREATE_USER,NO_ENGINE_SUBSTITUTION`

**Originator:** 1

**Character set client:** `utf8mb4`

**Connection collation:** `utf8mb4_uca1400_ai_ci`

**Database collation:** `utf8mb4_uca1400_ai_ci`

**Interval:** `1` day

**Starts:** `2030-01-01 00:00:00`

```sql
CREATE DEFINER=`root`@`localhost` EVENT `purge_disabled_accounts` ON SCHEDULE EVERY 1 DAY STARTS '2030-01-01 00:00:00' ON COMPLETION PRESERVE DISABLE COMMENT 'Remove old disabled accounts' DO DELETE FROM accounts WHERE status = 'disabled' AND row_end < CURRENT_TIMESTAMP - INTERVAL 365 DAY
```
