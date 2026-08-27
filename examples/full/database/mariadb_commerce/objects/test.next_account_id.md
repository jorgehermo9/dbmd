# `test.next_account_id`

**Kind:** procedure

**Data access:** no SQL

**Deterministic:** no

**Security:** definer

**Definer:** `root@localhost`

**SQL mode:** `STRICT_TRANS_TABLES,ERROR_FOR_DIVISION_BY_ZERO,NO_AUTO_CREATE_USER,NO_ENGINE_SUBSTITUTION`

**Character set client:** `utf8mb4`

**Connection collation:** `utf8mb4_uca1400_ai_ci`

**Database collation:** `utf8mb4_uca1400_ai_ci`

**Parameters:** `in current_id bigint(20) unsigned, out next_id bigint(20) unsigned`

```sql
CREATE DEFINER=`root`@`localhost` PROCEDURE `next_account_id`(IN current_id BIGINT UNSIGNED, OUT next_id BIGINT UNSIGNED)
    NO SQL
SET next_id = current_id + 1
```
