# `aggregates.integer_total(integer)`

Adds integers with ordinary, moving, and parallel aggregation support

**Kind:** `normal`

**Arguments:** `integer`

**Owner:** `dbmd`

**Returns:** `bigint`

**Direct arguments:** 0

**Transition function:** `aggregates.total_step(bigint,integer)`

**Transition type:** `bigint`

**Transition space:** 0

**Final modify:** `shareable`

**Parallel:** `safe`

**Final function:** `aggregates.total_final(bigint)`

**Combine function:** `aggregates.total_combine(bigint,bigint)`

**Moving transition function:** `aggregates.total_step(bigint,integer)`

**Moving inverse function:** `aggregates.total_inverse(bigint,integer)`

**Moving final function:** `aggregates.total_final(bigint)`

**Moving transition type:** `bigint`

**Initial condition:** `0`

**Moving initial condition:** `0`

**Moving transition space:** 0

**Moving final modify:** `read_write`

