# ADR-0004: Treat Custom Template Roots as Complete Sets

Status: accepted

Date: 2026-07-18

## Context

Custom templates can be implemented as overlays on embedded defaults or as independently complete trees. Overlays reduce initial copying but couple user templates to internal partial names, lookup order, and embedded template changes. Missing overrides may silently change behavior when dbmd upgrades.

Complete sets require explicit entrypoints but make project ownership and failure modes clear.

## Decision

Selecting a custom template directory replaces the embedded template set. dbmd requires the entrypoints needed by the selected profile, layout, and emitted artifact kinds. Custom partials are entirely owned by that template set.

dbmd validates the selected set before connecting when required entrypoints can be determined locally. It never falls back from a missing custom entrypoint to an embedded template.

## Consequences

- Custom rendering is isolated from undocumented builtin partial structure.
- Missing templates fail clearly instead of producing mixed output.
- Exported starter templates must form a complete, internally consistent tree.
- Projects own the maintenance cost of customized sets across render-context compatibility changes.
- dbmd must document and eventually version the render-context boundary.

## Alternatives considered

### Overlay custom files on embedded defaults

Rejected because it creates hidden coupling and upgrade-sensitive lookup behavior.

### Configure an arbitrary path per profile or artifact

Deferred because an opinionated directory convention is simpler to validate and document.

### Compile templates into the binary only

Rejected because teams need to tune artifacts for their agents, humans, and internal review conventions.
