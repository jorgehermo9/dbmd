# `test.archive_accounts_once`

**Definer:** `root@localhost`

**Type:** one time

**Status:** disabled

**Time zone:** `SYSTEM`

**On completion:** preserve

**Schedule:** `AT 2031-01-01 00:00:00`

**SQL mode:** `ONLY_FULL_GROUP_BY,STRICT_TRANS_TABLES,NO_ZERO_IN_DATE,NO_ZERO_DATE,ERROR_FOR_DIVISION_BY_ZERO,NO_ENGINE_SUBSTITUTION`

**Originator server ID:** 1

**Character set client:** `latin1`

**Connection collation:** `latin1_swedish_ci`

**Database collation:** `utf8mb4_0900_ai_ci`

```sql
CREATE DEFINER=`root`@`localhost` EVENT `archive_accounts_once` ON SCHEDULE AT '2031-01-01 00:00:00' ON COMPLETION PRESERVE DISABLE DO SET @dbmd_archive_requested = 1
```
