# `test.analytics_tools`

Analytics package

**Kind:** `package`

**Security:** invoker

**Definer:** `root@localhost`

**SQL mode:** `STRICT_TRANS_TABLES,ERROR_FOR_DIVISION_BY_ZERO,NO_AUTO_CREATE_USER,NO_ENGINE_SUBSTITUTION`

**Body:** yes

```sql
CREATE DEFINER=`root`@`localhost` PACKAGE `analytics_tools`    SQL SECURITY INVOKER
    COMMENT 'Analytics package'
 PROCEDURE refresh_cache(IN tenant BIGINT);
FUNCTION normalize(value VARCHAR(255)) RETURNS VARCHAR(255);
END

CREATE DEFINER=`root`@`localhost` PACKAGE BODY `analytics_tools` PROCEDURE refresh_cache(IN tenant BIGINT)
BEGIN
    SELECT tenant;
END;
FUNCTION normalize(value VARCHAR(255)) RETURNS VARCHAR(255)
RETURN lower(value);
END
```
