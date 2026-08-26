# dbmd-backends

`dbmd-backends` is the compile-time composition root for the database families
built into dbmd. It owns the closed `Backend`, `SourceConfig`, `Source`, and
`Catalog` enums, plus introspection dispatch, render-context composition, and
the combined embedded template manifest.

Concrete backend behavior lives in sibling crates:

- [`dbmd-backend-sqlite`](../sqlite/README.md) owns SQLite configuration,
  catalogs, introspection, render preparation, templates, fixtures, and tests.
- [`dbmd-backend-postgres`](../postgres/README.md) owns the corresponding
  PostgreSQL implementation.
- [`dbmd-relational`](../relational/README.md) owns only shared relational
  vocabulary and presentation support whose meaning is equivalent across
  multiple backend crates.

`dbmd-core` remains unaware of the compiled backend set. Its generic
`SourceSnapshot<C>` and `DatabaseContext<C>` envelopes become heterogeneous only
when this crate instantiates `C` with the closed `Catalog` enum.

This is compile-time extensibility, not a runtime plugin interface. Adding a
backend creates another sibling backend crate and explicitly wires it into this
composition root; it does not add vendor types to core or render.

## Tests

The composition integration tests prove heterogeneous ordering and rendering:

```sh
just test-integration
```

Backend-specific fixture suites run at their owning seams:

```sh
just test-contract sqlite
just test-contract postgres
```

User-visible behavior remains in the
[product documentation](../../../docs/product/overview.md), while cross-crate
design remains in the
[architecture documentation](../../../docs/architecture/overview.md).
