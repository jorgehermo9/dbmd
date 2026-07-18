# ADR-0002: Use One Canonical Project Artifact

Status: accepted

Date: 2026-07-18

## Context

Teams may want different output profiles, layouts, subsets, or destinations. Persistently configuring multiple named outputs would make `render`, `verify`, CI, and agent instructions choose among competing artifacts. It would also complicate initialization and drift semantics before the core format is proven.

At the same time, developers need one-off experimentation and projects may draw context from multiple databases.

## Decision

`dbmd.toml` declares one canonical artifact contract:

- One output path.
- One selected profile and layout.
- Zero or more explicitly ordered source IDs, with omission meaning all sources sorted by ID.
- One selected embedded or custom template set.

Multiple named sources are first-class. They combine into the single canonical artifact.

The default is a single `DATABASE.md`; directory output is explicit opt-in. CLI flags can produce one-off alternate renders, but canonical lifecycle commands operate on committed config by default.

## Consequences

- `dbmd render` and `dbmd verify` have one unambiguous default target.
- Agent instructions can point to one canonical location.
- Multi-source projects retain one review and freshness contract.
- Power users can experiment without expanding committed config complexity.
- Projects needing multiple permanently verified artifacts must wait for evidence that the added model is justified or use explicit separate config files.

## Alternatives considered

### Named configured outputs

Deferred because it creates selection, precedence, verification, and initialization questions before a single artifact is useful.

### Adapt layout automatically by schema size

Rejected because crossing a size threshold would cause surprising large path and diff changes.

### Support only one source

Rejected for MVP because real applications often use separate application and analytics databases, and adding source identity later would destabilize paths and template context.
