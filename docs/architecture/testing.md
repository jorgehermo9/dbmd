# Testing Strategy

Status: two bootstrap unit tests implemented; fixture, snapshot, integration, and CLI layers are planned.

## Goals

Tests must prove semantic fidelity, deterministic output, safe filesystem behavior, and actionable command failures. Happy-path struct construction alone is insufficient for a database-introspection product.

## Test layers

### Core model tests

Use focused unit tests for real domain behavior:

- Qualification rules.
- Effective-fact derivation.
- Stable normalization order.
- Backend-specific helpers with semantic meaning.

Avoid tests that only repeat field assignment or serde derives.

### Config tests

Table-driven tests cover:

- Valid single- and multi-source TOML.
- Source-ID validation.
- Missing, duplicate, and empty selection.
- CLI/config/default precedence.
- Environment expansion and redaction.
- Layout/flag incompatibilities.
- Output-path resolution.

### Driver tests

Drivers use real databases where practical:

- SQLite temporary files in normal test runs.
- PostgreSQL containers in integration CI.
- ClickHouse containers in integration CI.

Mock catalog rows may isolate normalization edge cases, but they do not replace real catalog compatibility tests.

### Render-context tests

Snapshot the stable context separately from Markdown once the boundary exists. This distinguishes semantic/model regressions from presentation changes.

### Golden Markdown tests

Golden fixtures cover complete artifacts for each profile/layout/backend combination. Review output changes as product changes, not mechanical snapshot updates.

Include difficult content:

- Composite keys and foreign keys.
- Expressions and partial indexes.
- Pipes, backticks, newlines, Unicode, and long comments.
- Empty sections.
- Name collisions across namespaces and sources.
- Observed, effective, absent, and unknown facts.

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

Every golden test renders the same snapshot at least twice or otherwise asserts stable bytes. Driver tests deliberately create objects in non-sorted order and expect the canonical artifact's ordering.

No generated artifact contains current time, temporary paths, nondeterministic map order, driver row order, or environment-specific connection details.

## Quality gates

Run regularly:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --all-targets
```

Backend-container suites may be separate CI jobs, but SQLite and all pure unit/golden tests belong in the default suite.
