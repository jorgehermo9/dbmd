# Product Overview

Status: accepted product direction; the first useful SQLite render is
implemented and the canonical lifecycle is current.

## Thesis

dbmd is a Rust CLI that generates an agent-readable Markdown artifact representing live database structure. The canonical artifact is committed beside application code so agents and humans can inspect the current schema without replaying migrations or querying the database during every coding task.

The core product is not generic database documentation. It is database context optimized for code agents, with enough precision and stability for humans to review schema changes in pull requests.

## Problem

Code agents commonly learn database structure through weak sources:

- Migrations require reconstructing final state and may diverge from reality.
- Live queries consume turns, require credentials, and reduce repeatability.
- Raw schema dumps are accurate but noisy and poorly shaped for context windows.
- Human-oriented documentation is often stale or omits backend behavior that affects query correctness.

The missing artifact is an accurate, deterministic, reviewable, searchable, and compact representation of current database structure.

## Positioning

`tbls` is strong prior art and overlaps with the basic documentation use case. dbmd should not win by reproducing every `tbls` feature. It should focus on agent-first navigation, explicit backend semantics, compact output, and a complete generate–commit–verify–consume lifecycle.

Native tools such as `pg_dump --schema-only`, SQLite `.schema`, and ClickHouse `SHOW CREATE TABLE` are valuable metadata sources but are not the final agent-oriented artifact.

An MCP server may become complementary later, but plain committed files are the initial product boundary. They work with ordinary file tools, code review, and CI without requiring a live service during agent tasks.

## Target users

- Developers using code agents in applications backed by real databases.
- Teams that want `DATABASE.md` or a `database/` directory beside `README.md`, design docs, and agent instructions.
- Reviewers who want schema changes to produce clean pull-request diffs.
- Agent authors who need deterministic database context.

## Product principles

### Agent-readable first

Output should minimize context-window cost while preserving facts that affect correctness or performance.

### Explicit over inferred

Agents should not have to infer backend defaults when dbmd can state the effective behavior. Observed, effective, and unknown facts remain distinguishable.

### Determinism is a feature

The canonical artifact is committed. Stable ordering, formatting, paths, and comparison behavior are product requirements.

### Canonical config is the project contract

Committed configuration defines one canonical artifact. CLI flags support exploration and one-off alternatives without expanding the persistent project contract.

### Start narrow and go deep

A backend is useful only when dbmd preserves enough semantics to prevent common wrong assumptions. SQLite comes first, followed by PostgreSQL and ClickHouse depth.

### Complete the lifecycle

The product includes initialization, generation, verification, CI integration, and agent-consumption guidance. Rendering alone does not keep an artifact fresh or teach agents to use it.

## Goals

- Generate Markdown from actual current database structure.
- Preserve backend-specific semantics that affect correctness or performance.
- Produce deterministic output suitable for git diffs and CI verification.
- Keep secrets out of committed configuration.
- Support embedded defaults and complete custom template sets.
- Support multiple named sources while maintaining one canonical artifact.
- Provide a practical adoption and maintenance path.

## Non-goals

- Replace migration tools.
- Become a live MCP server in the initial product.
- Reimplement every feature of existing database-documentation tools before proving the agent-first format.
- Put volatile production statistics in the primary schema artifact by default.
- Pretend every backend fits one universal SQL dialect or namespace model.
- Guarantee a stable custom-template context before that compatibility surface is explicitly versioned.

## Canonical workflow

```sh
dbmd init
dbmd render
dbmd verify
```

- `init` creates safe-to-commit configuration and optional integration files.
- `render` resolves the selected sources, introspects them, and replaces the canonical artifact.
- `verify` renders to a temporary location and fails when the committed artifact differs.

Supporting commands such as `doctor`, `explain`, and `lint` have separate responsibilities and are specified under [features](features/README.md).

## Default product shape

- Configuration file: `dbmd.toml`.
- Canonical output: one configured artifact.
- Default output path: `DATABASE.md`.
- Default layout: `single_file`.
- Default profile: `agent`.
- Source model: one or more named connection-backed sources.
- Template model: embedded defaults or a complete custom template set.
- Verification model: byte-for-byte files and file-set comparison.
- Artifact metadata: schema context only by default; no timestamps, fingerprints, versions, or generated-by headers.

## Success criteria

- A developer can render a useful local SQLite database artifact in under five minutes.
- A code agent can answer ordinary schema questions by reading the artifact without database access.
- Pull requests show stable, comprehensible changes when the database structure changes.
- Backend-specific output prevents obvious wrong assumptions, especially around keys, engines, generated values, and namespace behavior.
- A project can install drift checking and agent instructions without designing its own lifecycle.

## Current status

The first useful SQLite render and the initial PostgreSQL depth slice are
implemented end to end. `dbmd render` reads `dbmd.toml`, expands environment
references, selects named SQLite and PostgreSQL sources in deterministic order,
introspects them through concrete adapters, renders the embedded Markdown
profile, and atomically replaces the configured artifact.

The active milestone is closing the remaining canonical-lifecycle edges in
[Phase 3](roadmap.md). Base/template/CI initialization, exact verification,
multi-source presentation, directory layout, configless SQLite, stdout, and
complete custom template roots are implemented. Agent-instruction generation
and release hardening remain while PostgreSQL depth continues in parallel.
