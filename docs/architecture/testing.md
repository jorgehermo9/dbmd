# Testing Strategy

Status: core, complete SQLite introspection fixtures, deterministic application golden rendering, atomic-failure preservation, and thin CLI help tests are implemented. Verify and directory-output suites remain future work.

## Goals

Tests must prove semantic fidelity, deterministic output, safe filesystem behavior, and actionable command failures. Happy-path struct construction alone is insufficient for a database-introspection product.

## Test layers

Each crate is tested at its public interface. Unit tests cover concentrated internal domain behavior; integration tests prove the behavior of the crate as a module. Tests should not expose internal seams solely to reach implementation details.

### Core model tests

Use focused unit tests for real domain behavior:

- Qualification rules.
- Effective-fact derivation.
- Stable normalization order.
- Backend-specific helpers with semantic meaning.

Avoid tests that only repeat field assignment or serde derives.

Core integration tests exercise construction of complete `DatabaseContext` values and their cross-object invariants without database, template, or filesystem dependencies.

### Config tests

Table-driven tests cover:

- Valid single- and multi-source TOML.
- Source-ID validation.
- Missing, duplicate, and empty selection.
- CLI/config/default precedence.
- Environment expansion and redaction.
- Layout/flag incompatibilities.
- Output-path resolution.

### Introspection tests

Introspection uses real databases where practical:

- SQLite temporary files in normal test runs.
- PostgreSQL containers in integration CI.
- ClickHouse containers in integration CI.

Mock catalog rows may isolate normalization edge cases, but they do not replace real catalog compatibility tests. SQLite integration tests execute `.sql` fixtures against temporary databases and snapshot the normalized `SourceSnapshot`, independently of Markdown rendering.

### Render-context tests

Snapshot the stable context separately from Markdown once the seam exists. This distinguishes semantic/model regressions from presentation changes.

### Golden Markdown tests

Golden fixtures cover complete artifacts for each profile/layout/backend combination. Review output changes as product changes, not mechanical snapshot updates.

Include difficult content:

- Composite keys and foreign keys.
- Expressions and partial indexes.
- Pipes, backticks, newlines, Unicode, and long comments.
- Empty sections.
- Name collisions across namespaces and sources.
- Observed, effective, absent, and unknown facts.

Renderer integration tests construct core values directly. This keeps presentation snapshots fast and makes a renderer regression distinguishable from an introspection regression.

### Application integration tests

Application tests exercise the complete operation interface without routing through Clap:

```text
dbmd.toml + SQLite .sql fixture
  → app::render
  → deterministic DATABASE.md
```

The comprehensive Phase 2 fixture covers tables, columns, primary and foreign keys, indexes, views, generated columns, strict tables, and `WITHOUT ROWID`. Focused cases cover config resolution, environment expansion, source selection, failure preservation, and atomic replacement.

### Filesystem tests

Use isolated temporary repositories to verify:

- Atomic single-file replacement.
- Directory replacement removes stale generated files.
- Render preserves previous output after pre-replacement failure.
- Dangerous paths and symlink output roots are rejected.
- Verify never changes the canonical artifact.
- Added, modified, deleted, and unchanged comparison states.

### CLI tests

Exercise the compiled binary for:

- Help and version output.
- Exit codes by failure category.
- Stdout versus file output.
- Compact verify summaries and complete diff mode.
- Credential redaction.

CLI tests remain deliberately few. Domain, introspection, rendering, configuration, and filesystem behavior belong to their owning crate rather than being retested exhaustively through the binary.

## Test directory structure

Use Cargo integration-test directories within the owning crate rather than a workspace-root `tests/` directory:

```text
crates/core/tests/
crates/introspect/tests/
  fixtures/sqlite/<case>/schema.sql
  snapshots/
crates/render/tests/
  snapshots/
crates/app/tests/
  fixtures/sqlite/<case>/schema.sql
  fixtures/sqlite/<case>/dbmd.toml
  snapshots/
crates/cli/tests/
```

Shared helpers stay local to the owning crate until genuine cross-crate duplication justifies a test-support crate.

Use Insta for structural and Markdown snapshots. Commit `.snap` files, review changes intentionally with `cargo insta review`, and run CI with snapshot updates disabled. Prefer ordinary assertions when a value is small enough to understand more clearly without a snapshot.

## First SQLite fixture matrix

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

Run regularly:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --all-targets
```

Backend-container suites may be separate CI jobs, but SQLite and all pure unit/golden tests belong in the default suite.
