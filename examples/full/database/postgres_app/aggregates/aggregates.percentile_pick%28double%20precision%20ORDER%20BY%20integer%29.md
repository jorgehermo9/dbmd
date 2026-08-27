# `aggregates.percentile_pick(double precision ORDER BY integer)`

Example ordered-set aggregate

**Kind:** `ordered_set`

**Arguments:** `double precision ORDER BY integer`

**Owner:** `dbmd`

**Returns:** `integer`

**Direct arguments:** 1

**Transition function:** `aggregates.collect_integer(integer[],integer)`

**Transition type:** `integer[]`

**Transition space:** 0

**Final modify:** `read_write`

**Parallel:** `safe`

**Final function:** `aggregates.pick_integer(integer[],double precision)`

**Initial condition:** `{}`

