# `test.next_account_id`

**Kind:** procedure

**Data access:** no SQL

**Deterministic:** no

**Security:** definer

**Definer:** `root@localhost`

**SQL mode:** `ONLY_FULL_GROUP_BY,STRICT_TRANS_TABLES,NO_ZERO_IN_DATE,NO_ZERO_DATE,ERROR_FOR_DIVISION_BY_ZERO,NO_ENGINE_SUBSTITUTION`

**Character set client:** `latin1`

**Connection collation:** `latin1_swedish_ci`

**Database collation:** `utf8mb4_0900_ai_ci`

**Parameters:** `in current_id bigint unsigned, out next_id bigint unsigned`

**External language:** `SQL`

```sql
CREATE DEFINER=`root`@`localhost` PROCEDURE `next_account_id`(IN current_id BIGINT UNSIGNED, OUT next_id BIGINT UNSIGNED)
    NO SQL
SET next_id = current_id + 1
```
