# Testing Strategy

## Goals

Tests must prove semantic fidelity, deterministic output, safe filesystem behavior, and actionable command failures. Happy-path struct construction alone is insufficient for a database-introspection product.

## Test taxonomy

Test layers describe the seam and confidence being exercised. They are not
synonyms for runtime cost, fixture technology, or CI jobs.

- **Unit tests** live beside implementation under `src/` and concentrate on
  internal logic such as parsing, normalization, semantic translation, and
  otherwise impractical dependency outcomes.
- **Integration tests** live under an owning crate's `tests/` directory and use
  its public interface with real hermetic dependencies such as temporary files,
  SQLite, DuckDB, configuration, templates, and filesystem operations.
- **Backend contract tests** exercise a backend adapter against the exact
  supported database version using real DDL and catalog queries. A temporary
  embedded database and a Testcontainers server are the same test layer; only
  their execution requirements differ.
- **End-to-end tests** execute the compiled `dbmd` binary and assert its process
  status, output streams, artifact bytes, and filesystem effects.
- **Documentation tests** compile and execute public examples where the example
  itself is part of the interface.

Snapshots are an assertion technique used within these layers, not a separate
test layer. Application integration tests may use a server-backed adapter while
remaining application-interface tests; CI may colocate them with that backend's
contract lane to reuse the same compilation graph.

## Test layers

Each crate is tested at its public interface. Unit tests cover concentrated internal domain behavior; integration tests prove the behavior of the crate as a module. Tests should not expose internal seams solely to reach implementation details.

### Core envelope tests

Use focused unit tests for real domain behavior:

- Source-ID validation.
- Source envelope construction.
- Selected source order.
- Empty and duplicate source rejection.

Avoid tests that only repeat field assignment or serde derives.

Core integration tests exercise `DatabaseContext<C>` invariants without a
database, template, filesystem dependency, or concrete backend catalog.

### Config tests

Table-driven tests cover:

- Valid single- and multi-source TOML.
- Source-ID validation.
- Missing, duplicate, and empty selection.
- CLI/config/default precedence.
- Environment expansion and redaction.
- Layout/flag incompatibilities.
- Output-path resolution.

### Backend contract tests

Introspection uses real databases where practical:

- SQLite temporary files in normal test runs.
- DuckDB temporary files in normal test runs.
- PostgreSQL, ClickHouse, MySQL, and MariaDB containers in opt-in contract
  suites.

Mock catalog rows may isolate normalization edge cases, but they do not replace real catalog compatibility tests. SQLite integration tests execute `.sql` fixtures against temporary databases and snapshot the normalized `SourceSnapshot`, independently of Markdown rendering.

### Render-context tests

Snapshot the stable context separately from Markdown. This distinguishes
semantic/model regressions from presentation changes.

### Golden Markdown tests

Golden fixtures cover complete artifacts for each profile/layout/backend combination. Review output changes as product changes, not mechanical snapshot updates.

Include difficult content:

- Composite keys and foreign keys.
- Expressions and partial indexes.
- Pipes, backticks, newlines, Unicode, and long comments.
- Empty sections.
- Name collisions across namespaces and sources.
- Observed, effective, absent, and unknown facts.

Renderer integration tests construct presentation values directly. Backend
integration tests separately prove the catalog-to-render-context mapping. This
keeps presentation tests fast and prevents `dbmd-render` from depending on
backend catalogs.

### Application integration tests

Application tests exercise the complete operation interface without routing through Clap:

```text
dbmd.toml + SQLite .sql fixture
  → app::render
  → deterministic DATABASE.md
```

The comprehensive SQLite application fixture covers tables, columns, primary
and foreign keys, indexes, views, generated columns, strict tables, and
`WITHOUT ROWID`. Focused cases cover config resolution, environment expansion,
source selection, failure preservation, and atomic replacement.

### Filesystem tests

Use isolated temporary repositories to verify:

- Atomic single-file replacement.
- Directory replacement removes stale generated files.
- Render preserves previous output after pre-replacement failure.
- Dangerous paths and symlink output roots are rejected.
- Verify never changes the canonical artifact.
- Added, modified, deleted, and unchanged comparison states.

### End-to-end tests

Exercise the compiled binary through the CLI crate's `tests/e2e.rs` target for:

- Help and version output.
- Exit codes by failure category.
- Stdout versus file output.
- Compact verify summaries and complete diff mode.
- Credential redaction.
- Credential-free explain output and render-override precedence.
- Doctor's local default, explicit connection mode, all-source scope, and exit status.
- Agent-instruction preview and explicit safe file update.

End-to-end tests remain deliberately focused on command coordination. Domain,
introspection, rendering, configuration, and filesystem depth belong to their
owning module rather than being retested exhaustively through the binary.

## Test directory structure

Use Cargo integration-test directories within the owning crate rather than a workspace-root `tests/` directory:

```text
crates/core/tests/
crates/backends/sqlite/tests/
  fixtures/<case>/schema.sql
  snapshots/
crates/backends/postgres/tests/
  fixtures/<case>/schema.sql
  snapshots/
crates/backends/clickhouse/tests/
  fixtures/
  snapshots/
crates/backends/mysql/tests/
  fixtures/
  snapshots/
crates/backends/mariadb/tests/
  fixtures/
  snapshots/
crates/backends/duckdb/tests/
  fixtures/
  snapshots/
crates/backends/composition/tests/
crates/render/tests/
  snapshots/
crates/app/tests/
  fixtures/sqlite/<case>/schema.sql
  fixtures/sqlite/<case>/dbmd.toml
  fixtures/postgres/<case>/schema.sql
  snapshots/
crates/cli/tests/e2e.rs
```

Shared helpers stay local to the owning crate until genuine cross-crate duplication justifies a test-support crate.

Use Insta for structural and Markdown snapshots. Commit `.snap` files, review changes intentionally with `cargo insta review`, and run CI with snapshot updates disabled. Prefer ordinary assertions when a value is small enough to understand more clearly without a snapshot.

## SQLite fixture matrix

At minimum, create databases covering:

- Ordinary rowid table.
- Composite primary key.
- Foreign key with update/delete actions.
- Unique, partial, and expression indexes.
- Generated/hidden columns.
- Strict table.
- `WITHOUT ROWID` table.
- View with multiline definition.
- Attached database or explicit `main` namespace behavior.
- SQLite version capability detection.

## Determinism checks

Every golden test renders the same snapshot at least twice or otherwise asserts stable bytes. Introspection tests deliberately create objects in non-sorted order and expect canonical ordering in the resulting source snapshot.

No generated artifact contains current time, temporary paths, nondeterministic map order, catalog row order, or environment-specific connection details.

## Quality gates

Use the root `justfile` as the executable testing interface:

```sh
just test-unit
just test-integration
just test-contract
just test-e2e
just test-doc
just check
```

`just test` and `just check` accept an optional backend selector; omission means
the complete workspace. `cargo-nextest` selects unit and integration binaries by
Cargo target kind, so a new test cannot be silently classified by a hand-kept
file list. Doctests continue through Cargo because nextest does not execute
rustdoc tests.

Server-backed suites require Docker. The complete `just test` gate includes
them; their integration-test binaries own pinned containers and can also be
run independently while iterating:

```sh
just test-contract postgres
just test-contract clickhouse
just test-contract mysql
just test-contract mariadb
just test-contract duckdb
```

Use `just test <backend>` when the matching application integration target
must run with the adapter contract. Nextest's repository configuration keeps
tests within each server family serialized while CI runs different families in
parallel. Retries are disabled so intermittent failures remain visible.

The shared lifecycle implementation lives in `crates/test-support`; fixture SQL,
assertions, and snapshots remain in the crate whose public seam they test.
Catalog coverage is documented beside every adapter in its `README.md`.

## CI execution lanes

CI lanes model execution cost and compilation graphs rather than redefining the
test taxonomy:

- **Quality** runs formatting, strict all-target/all-feature Clippy, and
  workflow validation.
- **Tests** runs unit, hermetic integration, SQLite and DuckDB contracts,
  end-to-end, and documentation tests in one job so local build artifacts are
  reused.
- **Backend / _name_** matrix jobs run each server backend's adapter contract
  tests and matching application integration test in parallel.
- **CI** is the stable aggregate status for branch protection.

Every Rust lane has a distinct dependency cache because its enabled features
and compilation products differ. Pull requests restore trusted default-branch
caches; only successful `main` pushes save them. Obsolete runs for the same ref
are cancelled, matrix fail-fast is disabled, and every job has an explicit
timeout.
