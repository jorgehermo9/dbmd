# `advanced.orders_dependencies`

Cross-column order distribution

**Owner:** `dbmd`

**Kinds:** `ndistinct, dependencies, mcv`

**Statistics target:** 500

**Columns:** `customer_id, region`

```sql
CREATE STATISTICS advanced.orders_dependencies ON customer_id, region FROM advanced.orders
```
