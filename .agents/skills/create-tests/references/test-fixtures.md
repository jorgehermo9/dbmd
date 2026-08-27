# Test fixtures and external harnesses

Read this reference before creating or refactoring shared fixture files,
factories, scenarios, project fixtures, or Testcontainers support. The coverage
matrix, test layers, test shape, and `rstest` guidance remain in
[SKILL.md](../SKILL.md).

## Fixture files, factories, and scenarios

Use these terms consistently:

- A **fixture file** is committed input such as DDL, TOML, or an expected
  artifact. Keep it beside the owning crate under `tests/fixtures/`.
- A **factory** creates fresh valid test state with deterministic defaults and
  explicit overrides. It creates or fails; it never finds-or-reuses state.
- A **scenario** composes factories into a meaningful repeated starting state,
  such as a configured two-source SQLite project. Its name describes the
  starting state, not the assertion it is intended to make pass.
- A **harness** owns an external runtime boundary, such as a compiled CLI
  process or a Testcontainers server.

Keep the organization simple:

- One-off setup stays in the test.
- Repeated setup shared by tests in one crate lives under that crate's
  `tests/support/`.
- Cross-crate RAII infrastructure lives in `dbmd-test-support` only when more
  than one crate uses the same lifecycle contract.
- Add a named scenario only when several tests genuinely share the same
  meaningful starting state. Compose factories directly for one-off states.

Factories own construction, paths, and cleanup. Tests own behavior-driving
choices and assertions. Avoid factories that accept a large bag of loosely
typed options, scenarios created for one test, assertion-heavy setup helpers,
or a trait hierarchy for test data.

Use absolute fixed instants, stable round values, obviously fake credentials,
and explicit DDL. Avoid random identifiers unless collision behavior is the
contract; isolated resources should make randomness unnecessary.

## Database and project fixtures

Model the lifecycle required by the test seam instead of forcing every backend
through one shared fixture type:

- A SQLite or DuckDB backend compatibility test uses a backend-local
  `TestDatabase` under that backend crate's `tests/support/`. It owns a real
  temporary directory and database file, verifies the supported runtime version
  before fixture DDL runs, executes explicit DDL, and exposes the database path
  or a fresh connection. Drop setup connections before read-only introspection
  unless an open connection is part of the behavior under test. This fixture
  models a database, not a dbmd project, so it does not wrap `TestProject`.
- An application integration uses an app-local project fixture that composes
  `dbmd_test_support::TestProject` with application configuration, explicit
  environment, source databases, and request construction. Embedded databases
  are created inside that project tree so relative-path and artifact behavior is
  real.
- A CLI end-to-end test uses a CLI-local project fixture that composes
  `TestProject` with embedded-database creation and process execution. It owns
  the command's working directory and exposes status, streams, and filesystem
  effects without embedding assertions.
- A server-backed compatibility or application integration uses the shared
  Testcontainers fixture for server lifecycle, then keeps its DDL, scenarios,
  and assertions in the owning crate.

Consistency means common fixture concepts and comparable ergonomics, not one
repository-wide database abstraction. Do not add a common fixture trait, wrap
an embedded database in `TestProject`, or move driver-specific helpers into
`dbmd-test-support` solely for symmetry. Share an implementation only when
multiple crates use the same lifecycle contract.

## Testcontainers

PostgreSQL, ClickHouse, MySQL, and MariaDB integration tests use declarative
fixtures in `dbmd-test-support`. A new server fixture owns its complete
lifecycle:

- an exact supported image tag, plus a digest when upstream tag mutability is a
  realistic risk;
- explicit environment, command, exposed ports, and wait strategy;
- a bounded startup timeout and actionable startup errors;
- mapped host endpoints without assuming Docker runs on localhost;
- a runtime version assertion before fixture DDL executes;
- obviously fake credentials scoped to the temporary server;
- an RAII container guard retained for the fixture's lifetime;
- isolated per-case databases or schemas and cleanup guards where the backend
  supports them.

Backend compatibility and application integration tests reuse the same server
fixture API. They do not start containers through shell commands, Compose,
ambient container names, or shared long-lived services. Keep server feature
dependencies scoped so selecting one backend does not compile unrelated client
drivers.

Use a real deterministic server failure when the database can produce it. When
the desired malformed catalog value cannot exist in the supported server,
unit-test the decoder outcome instead of replacing part of a container-backed
test with a mock.

## Maintainability checks

Before finishing fixture or harness work:

- Each helper still has multiple callers or owns cross-crate infrastructure.
- Every scenario remains a meaningful repeated state rather than a test-name
  alias.
- Moving setup into a helper did not hide the precondition that drives the
  behavior.
- Tests remain order-independent and safe under parallel execution.
- No fixture contains current time, ambient paths, host-dependent values, or
  secrets.
- Skips and ignored tests carry an issue URL explaining the temporary gap.
