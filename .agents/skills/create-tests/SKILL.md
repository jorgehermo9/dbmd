---
name: create-tests
description: Design, create, change, or audit Rust tests in dbmd. Use whenever test coverage, fixtures, snapshots, test tiers, or test files are touched; do not use merely to execute an unchanged suite.
---

# Create Tests

Test observable contracts at the repository's accepted module seams. Read the
owning product specification, `docs/architecture/testing.md`, and the relevant
Rust testing rules before changing a test.

## Coverage matrix

Before code, derive a coverage matrix from the contract rather than the current
implementation. Include every distinct happy path, edge, error, determinism,
safety/redaction, and compatibility behavior that applies.

| # | Behavior | Class | Preconditions / input | Observable outcome | Layer | Status |
|---|----------|-------|-----------------------|--------------------|-------|--------|
| 1 | renders a selected source | Happy | valid config and schema | exact artifact bytes | Integration | ❌ missing |

Use these statuses:

- `✅ path › test_name` when covered.
- `❌ missing` when coverage remains.
- `⏭️ N/A — reason` only for an explicit, defensible exclusion.

One row represents one independently failing behavior. A test may cover several
rows only when they form one coherent scenario with shared expensive setup and
each outcome remains explicit in the assertions. Split scenarios whose failures
would need different diagnoses.

Make a second pass after implementation. Completion means every row is `✅` or
has a justified `⏭️`; surface the completed matrix in the final response and PR
description. Keep matrices in conversation or a temporary `docs/plan/`
scratchpad, never in durable product or architecture documentation.

## Test layers

Choose the narrowest layer that proves the behavior through its owning
interface.

### Unit

Place `#[cfg(test)]` modules beside implementation under `src/`. Use unit tests
for parsers, normalization, state transitions, semantic token translation, and
failure branches that require a dependency to return an otherwise impractical
response. Private access is acceptable only when the private behavior itself is
the concentrated module logic.

### Integration

Place integration tests under the owning crate's `tests/` directory and use
only its public interface. Choose one independently runnable subtype:

- **Hermetic crate integration** composes crate modules with local resources
  such as temporary files and templates, excluding the application and concrete
  backend crates.
- **Backend compatibility integration** exercises an adapter against the exact
  supported database version using real temporary databases or pinned
  Testcontainers images and real DDL. Prove catalog fidelity, semantic
  normalization, deterministic order, render-context mapping, redaction, and
  repeat introspection. Backend mocks do not satisfy a compatibility row.
- **Application integration** exercises the application API without spawning
  `dbmd`, using configuration, templates, rendering, filesystem replacement,
  and local or server-backed adapters as required.

Keep backend fixtures and snapshots beside the owning backend crate. A backend
coverage README claims only surfaces backed by executable fixtures or an
explicit documented exclusion.

### End-to-end

Exercise the compiled `dbmd` binary from `crates/cli/tests/`. Assert exit status,
stdout, stderr, artifact bytes, and filesystem effects. Keep this tier focused
on user-visible command coordination; backend catalog depth belongs to backend
compatibility integrations and application behavior belongs to application
integrations.

Cargo `examples/` binaries are not a test layer. Test user-facing example
projects through application integration or CLI E2E at the product seam they
demonstrate.

## Fault selection

Use a real failure when it is deterministic at the seam: invalid configuration,
unsafe paths, permission failures, authentication rejection, malformed DDL,
connection refusal, container loss, or catalog incompatibility. Preserve real
resources through the path and assert the public error and absence of partial
effects.

When a database cannot emit the malformed value on demand, unit-test the
acquisition decoder with the synthetic row or response. A test that replaces
part of a live backend with a fake is a unit test wearing integration setup;
split the synthetic reaction from the real backend compatibility scenario.

## Fixtures and determinism

- Own resources with RAII guards so cleanup runs after assertion failures.
- Pin database image versions and verify the runtime version before fixtures.
- Make DDL explicit when a default expands from host state, including CPU sets,
  paths, time zones, locale, or server capacity.
- Create isolated temporary repositories and databases. Do not share mutable
  state across tests.
- Run repeat acquisition/rendering when determinism is part of the contract.
- Assert secrets are absent from catalogs, debug output, errors, and Markdown.

## Assertions and snapshots

Assert small semantic facts directly. Use Insta for rich normalized catalogs,
render contexts, Markdown, CLI output, and error reports where reviewing the
whole diff is valuable. A snapshot does not replace focused assertions for
critical identity, semantic-enum, safety, or redaction invariants.

CI always uses `INSTA_UPDATE=no`. Update snapshots only through
`just snapshots [backend]`, inspect every diff, and reject ambient host data,
temporary paths, timestamps, unordered collections, or credentials.

## Test shape

- Name the behavior and expected outcome, not the function being called.
- Arrange, act, and assert visibly; comments are optional when the structure is
  already clear.
- Build expectations from specifications, documented database behavior, or
  worked literals rather than mirroring production code.
- Prefer public results and effects over calls to internal helpers.
- Keep shared helpers local until duplication proves a repository-level test
  module is warranted.
- Run independent scenarios safely in parallel; serialize only a proven shared
  resource through the test runner configuration.

## Workflow

For behavior changes, work red → green one matrix row at a time. Use the root
`justfile` for tier and backend selection. Before completion, run the affected
tier, formatting, strict Clippy, and the smallest complete backend and
application integration slices; use `just check` for the full local gate.
