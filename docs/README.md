# dbmd Documentation

This directory separates product behavior from implementation architecture and durable decisions.

## Product specification

- [Overview](product/overview.md) — thesis, users, goals, workflow, scope, and success criteria.
- [Concepts](product/concepts.md) — the product model and relationships between sources, snapshots, artifacts, layouts, profiles, and drift.
- [Feature specifications](product/features/README.md) — observable behavior of each command and product surface.

Product documents describe durable behavior and scope. Availability belongs in
the user-facing command reference, not in status ledgers inside specifications.

## Architecture

- [Overview](architecture/overview.md) — design principles, workspace shape, boundaries, and data flow.
- [Schema model](architecture/schema-model.md) — normalized model, backend extensions, source aggregation, and fact provenance.
- [Rendering](architecture/rendering.md) — render context, templates, deterministic output, and writing artifacts.
- [Configuration and CLI](architecture/config-and-cli.md) — configuration resolution, command orchestration, validation, and safety.
- [Testing](architecture/testing.md) — test layers, fixtures, and backend coverage.

Architecture documents describe how the product is built. Code sketches are directional design aids rather than progress markers.

## Decisions and agent configuration

- [Architecture decision records](adr/README.md) capture durable trade-offs.
- [Agent configuration](agents/) tells engineering skills how to use GitHub Issues, triage labels, and domain documentation.
- The canonical domain glossary is [CONTEXT.md](../CONTEXT.md).

## Documentation rules

- Put stable product intent in `product/overview.md`.
- Put observable feature behavior beside that feature under `product/features/`.
- Put implementation boundaries and design mechanics under `architecture/`.
- Put resolved, hard-to-reverse trade-offs under `adr/`.
- Put unresolved questions in the closest owning document, not in a global question dump.
- Keep temporal plans, phases, progress ledgers, and session notes out of persistent docs.
- Put temporary planning files under `docs/plan/<planning-session-slug>/` and delete them when no longer needed.
- Prefer links to duplicated prose when a rule has one canonical owner.
