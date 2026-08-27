# `test.tenant_documents`

**Kind:** `json_relational_duality`

**Check option:** none

**Updatable:** yes

**Security:** definer

**Definer:** `root@localhost`

**JSON column:** `data`

**Root table:** `test.tenants`

**Status:** `valid`

**Operations:** `insert=false, update=false, delete=false, read_only=true`

**Mapped table:** `#0 test.tenants parent=None relationship=- where=- permissions=false/false/false read_only=true root=true`

**JSON field:** `_id -> #0 test.tenants.tenant_id, permissions=false/false/false read_only=true root=true`

**JSON field:** `name -> #0 test.tenants.name, permissions=false/false/false read_only=true root=true`

| Column | Type | Nullable |
|---|---|---|


```sql
CREATE ALGORITHM=UNDEFINED DEFINER=`root`@`localhost` SQL SECURITY DEFINER JSON RELATIONAL DUALITY VIEW `tenant_documents` AS select json_duality_object('_id':`tenants`.`tenant_id`,'name':`tenants`.`name`) AS `JSON_DUALITY_OBJECT('_id':tenant_id, 'name':name)` from `tenants`
```