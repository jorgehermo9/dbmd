# `test.archive_accounts_once`

**Kind:** `event`

**Status:** disabled

**Schedule:** one time

**Completion:** preserve

**Definer:** `root@localhost`

**Time zone:** `SYSTEM`

**SQL mode:** `STRICT_TRANS_TABLES,ERROR_FOR_DIVISION_BY_ZERO,NO_AUTO_CREATE_USER,NO_ENGINE_SUBSTITUTION`

**Originator:** 1

**Character set client:** `utf8mb4`

**Connection collation:** `utf8mb4_uca1400_ai_ci`

**Database collation:** `utf8mb4_uca1400_ai_ci`

**Execute at:** `2031-01-01 00:00:00`

```sql
CREATE DEFINER=`root`@`localhost` EVENT `archive_accounts_once` ON SCHEDULE AT '2031-01-01 00:00:00' ON COMPLETION PRESERVE DISABLE DO SET @dbmd_mariadb_archive_requested = 1
```
