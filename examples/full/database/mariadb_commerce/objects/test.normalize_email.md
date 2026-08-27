# `test.normalize_email`

**Kind:** function

**Data access:** no SQL

**Deterministic:** yes

**Security:** definer

**Definer:** `root@localhost`

**SQL mode:** `STRICT_TRANS_TABLES,ERROR_FOR_DIVISION_BY_ZERO,NO_AUTO_CREATE_USER,NO_ENGINE_SUBSTITUTION`

**Character set client:** `utf8mb4`

**Connection collation:** `utf8mb4_uca1400_ai_ci`

**Database collation:** `utf8mb4_uca1400_ai_ci`

**Parameters:** `return varchar(255), in value varchar(255) default 'fallback@example.invalid'`

```sql
CREATE DEFINER=`root`@`localhost` FUNCTION `normalize_email`(value VARCHAR(255) DEFAULT 'fallback@example.invalid') RETURNS varchar(255) CHARSET utf8mb4 COLLATE utf8mb4_uca1400_ai_ci
    NO SQL
    DETERMINISTIC
RETURN lower(value)
```
