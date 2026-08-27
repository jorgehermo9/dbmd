# `automation.invoice_number`

Stable invoice number allocator

**Owner:** `dbmd`

**Type:** `bigint`

**Start:** 1000

**Minimum:** 1000

**Maximum:** 999999

**Increment:** 5

**Cache:** 20

**Cycle:** yes

**Persistence:** `unlogged`

**Owned by:** `automation.invoices.id`

```sql
CREATE UNLOGGED SEQUENCE "automation"."invoice_number" AS bigint INCREMENT BY 5 MINVALUE 1000 MAXVALUE 999999 START WITH 1000 CACHE 20 CYCLE OWNED BY automation.invoices.id;
```
