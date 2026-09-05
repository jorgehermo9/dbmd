# `type_system.scalar_token`

Integer-backed application token

**Kind:** `base`

**Owner:** `dbmd`

**Input:** `type_system.scalar_token_in`

**Output:** `type_system.scalar_token_out`

**Internal length:** 4

**Passed by value:** yes

**Category:** `N`

**Preferred:** yes

**Delimiter:** `,`

**Alignment:** `int4`

**Storage:** `plain`

**Collatable:** no

**Default:** `0`

**Array type:** `type_system._scalar_token`

```sql
CREATE TYPE "type_system"."scalar_token" (
    INPUT = type_system.scalar_token_in,
    OUTPUT = type_system.scalar_token_out,
    INTERNALLENGTH = 4,
    PASSEDBYVALUE,
    ALIGNMENT = int4,
    STORAGE = plain,
    CATEGORY = 'N',
    PREFERRED = true,
    DEFAULT = '0',
    ARRAY_TYPE = type_system._scalar_token
);
```
