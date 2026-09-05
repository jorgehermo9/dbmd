# `test.disable_account`

**Kind:** procedure

**Data access:** modifies SQL data

**Deterministic:** no

**Security:** definer

**Definer:** `root@localhost`

**SQL mode:** `STRICT_TRANS_TABLES,ERROR_FOR_DIVISION_BY_ZERO,NO_AUTO_CREATE_USER,NO_ENGINE_SUBSTITUTION`

**Character set client:** `utf8mb4`

**Connection collation:** `utf8mb4_uca1400_ai_ci`

**Database collation:** `utf8mb4_uca1400_ai_ci`

**Parameters:** `in target_id bigint(20) unsigned`

```sql
CREATE DEFINER=`root`@`localhost` PROCEDURE `disable_account`(IN target_id BIGINT UNSIGNED)
    MODIFIES SQL DATA
UPDATE accounts SET status = 'disabled' WHERE account_id = target_id
```
