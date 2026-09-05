# `infrastructure.label_c`

**Kind:** `base`

**Owner:** `dbmd`

**Input:** `infrastructure.label_c_in`

**Output:** `infrastructure.label_c_out`

**Internal length:** 4

**Passed by value:** yes

**Category:** `U`

**Preferred:** no

**Delimiter:** `,`

**Alignment:** `int4`

**Storage:** `plain`

**Collatable:** no

**Array type:** `infrastructure._label_c`

```sql
CREATE TYPE "infrastructure"."label_c" (
    INPUT = infrastructure.label_c_in,
    OUTPUT = infrastructure.label_c_out,
    INTERNALLENGTH = 4,
    PASSEDBYVALUE,
    ALIGNMENT = int4,
    STORAGE = plain,
    CATEGORY = 'U',
    ARRAY_TYPE = infrastructure._label_c
);
```
