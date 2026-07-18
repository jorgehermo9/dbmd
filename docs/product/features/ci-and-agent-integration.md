# CI and Agent Integration

Status: protected GitHub Actions workflow generation is implemented; a reusable
official action, pre-commit recipe, agent snippets, and skill package remain.

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

Other CI systems follow only in response to demand.

## Pre-commit

The project will document a copy-paste hook recipe for local drift checks. Generation may follow if users need managed installation. Hooks must not hide the normal `dbmd render` and `dbmd verify` commands.

## Agent instructions

Generated snippets should tell compatible agents to:

- Read the canonical artifact before reconstructing database state from migrations.
- Prefer the artifact for structural questions when verification is expected to be current.
- Run or request `dbmd verify` when freshness is uncertain.
- Regenerate through `dbmd render`; never hand-edit generated files.
- Query a live database only when the artifact cannot answer an operational or data question.

## Open skill package

A distributable agent skill may teach artifact navigation across supported layouts. It should be ecosystem-neutral where possible and avoid depending on privileged database access.

The skill must treat the canonical artifact as context, not as proof that production data or runtime state matches a development environment.

## Open decisions

- Exact GitHub Action inputs and release pinning policy.
- Initial agent ecosystems for packaged instructions.
- Whether the skill should execute verification or only recommend it based on task authority.
