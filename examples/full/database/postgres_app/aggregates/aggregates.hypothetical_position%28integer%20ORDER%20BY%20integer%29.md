# `aggregates.hypothetical_position(integer ORDER BY integer)`

Example hypothetical-set aggregate

**Kind:** `hypothetical_set`

**Arguments:** `integer ORDER BY integer`

**Owner:** `dbmd`

**Returns:** `bigint`

**Direct arguments:** 1

**Transition function:** `aggregates.collect_integer(integer[],integer)`

**Transition type:** `integer[]`

**Transition space:** 0

**Final modify:** `read_write`

**Parallel:** `safe`

**Final function:** `aggregates.hypothetical_position(integer[],integer)`

**Initial condition:** `{}`

