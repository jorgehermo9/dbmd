# CI and Agent Integration

## GitHub Action

The official action installs dbmd and runs the canonical drift check. Its minimal interface should support:

- Explicit dbmd version selection or a documented pinning strategy.
- Working directory for monorepo consumers.
- Config path override.
- Environment and secret forwarding through normal GitHub Actions mechanisms.
- Optional unified diff output.

The action must preserve `dbmd verify` exit semantics rather than introducing a second comparison implementation.

## Generated workflow

`dbmd init ci` generates a GitHub Actions workflow that:

1. Checks out the repository.
2. Installs the selected dbmd version.
3. Provides an explicit location for source secrets through normal workflow
   environment configuration.
4. Runs `dbmd verify`.

The generated install command pins the current dbmd version. Existing workflow
files are protected unless `--force` is explicit.

Generators for other CI systems are outside the current product scope.

## Pre-commit

The pre-commit integration is a copy-paste hook recipe for local drift checks.
Managed hook generation is outside this contract. Hooks must
not hide the normal `dbmd render` and `dbmd verify` commands.

## Agent instructions

`dbmd init agents` prints a snippet or updates an explicitly named regular file
through an isolated idempotent marker block. Generated guidance tells agents to:

- Read the canonical artifact before reconstructing database state from migrations.
- Prefer the artifact for structural questions when verification is expected to be current.
- Run or request `dbmd verify` when freshness is uncertain.
- Regenerate through `dbmd render`; never hand-edit generated files.
- Query a live database only when the artifact cannot answer an operational or data question.

## Open skill package

A distributable agent skill may teach artifact navigation across supported layouts. It should be ecosystem-neutral where possible and avoid depending on privileged database access.

The skill must treat the canonical artifact as context, not as proof that production data or runtime state matches a development environment.

The snippet is ecosystem-neutral and can be installed in `AGENTS.md`,
`CLAUDE.md`, or another instruction file selected by the project.
