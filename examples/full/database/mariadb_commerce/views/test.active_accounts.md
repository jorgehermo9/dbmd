# `test.active_accounts`

**Check option:** cascaded

**Updatable:** yes

**Security:** invoker

**Algorithm:** merge

**Definer:** `root@localhost`

| Column | Type | Nullable |
|---|---|---|


```sql
CREATE ALGORITHM=MERGE DEFINER=`root`@`localhost` SQL SECURITY INVOKER VIEW `active_accounts` AS select `accounts`.`tenant_id` AS `tenant_id`,`accounts`.`account_id` AS `account_id`,`accounts`.`email` AS `email` from `accounts` where `accounts`.`status` = 'active' WITH CASCADED CHECK OPTION
```