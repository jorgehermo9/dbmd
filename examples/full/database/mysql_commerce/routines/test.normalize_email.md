# `test.normalize_email`

**Kind:** function

**Data access:** no SQL

**Deterministic:** yes

**Security:** definer

**Definer:** `root@localhost`

**SQL mode:** `ONLY_FULL_GROUP_BY,STRICT_TRANS_TABLES,NO_ZERO_IN_DATE,NO_ZERO_DATE,ERROR_FOR_DIVISION_BY_ZERO,NO_ENGINE_SUBSTITUTION`

**Character set client:** `latin1`

**Connection collation:** `latin1_swedish_ci`

**Database collation:** `utf8mb4_0900_ai_ci`

**Parameters:** `return varchar(255), in value varchar(255)`

**External language:** `SQL`

```sql
CREATE DEFINER=`root`@`localhost` FUNCTION `normalize_email`(value VARCHAR(255)) RETURNS varchar(255) CHARSET utf8mb4
    NO SQL
    DETERMINISTIC
RETURN lower(value)
```
