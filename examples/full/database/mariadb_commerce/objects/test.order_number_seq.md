# `test.order_number_seq`

**Kind:** `sequence`

**Type:** `bigint`

**Start:** `1000`

**Minimum:** `1`

**Maximum:** `9223372036854775806`

**Increment:** `10`

**Cycle:** no

**Cache:** `20`

**Engine:** `InnoDB`

```sql
CREATE SEQUENCE `order_number_seq` start with 1000 minvalue 1 maxvalue 9223372036854775806 increment by 10 cache 20 nocycle ENGINE=InnoDB
```
