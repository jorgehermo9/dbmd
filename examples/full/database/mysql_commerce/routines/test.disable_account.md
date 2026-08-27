# `test.disable_account`

**Kind:** procedure

**Data access:** modifies SQL data

**Deterministic:** no

**Security:** definer

**Definer:** `root@localhost`

**SQL mode:** `ONLY_FULL_GROUP_BY,STRICT_TRANS_TABLES,NO_ZERO_IN_DATE,NO_ZERO_DATE,ERROR_FOR_DIVISION_BY_ZERO,NO_ENGINE_SUBSTITUTION`

**Character set client:** `latin1`

**Connection collation:** `latin1_swedish_ci`

**Database collation:** `utf8mb4_0900_ai_ci`

**Parameters:** `in target_id bigint unsigned`

**External language:** `SQL`

```sql
CREATE DEFINER=`root`@`localhost` PROCEDURE `disable_account`(IN target_id BIGINT UNSIGNED)
    MODIFIES SQL DATA
UPDATE accounts SET status = 'disabled' WHERE account_id = target_id
```
