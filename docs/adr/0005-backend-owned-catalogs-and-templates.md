# ADR-0005: Let Backend Modules Own Catalogs and Templates

Status: accepted

Date: 2026-07-19

## Context

Relational databases share names such as table, column, index, and trigger, but
their actual catalog semantics do not form one lossless universal object model.
PostgreSQL triggers alone may combine events, run per row or statement, expose
transition tables, reference constraint metadata, and carry enablement state.
SQLite triggers have a different and smaller grammar. Adding every distinction
to common enums makes unrelated crates know every supported backend and turns
the template API into a flattened least-common-denominator model.

dbmd still needs one small application API for a configured set of compiled-in
backends. Runtime third-party driver loading is not a current product goal.

## Decision

`dbmd-core` owns only backend-neutral identity envelopes and aggregate
invariants: `SourceId`, generic `SourceSnapshot<C>`, and
`DatabaseContext<C>`. Concrete normalized catalogs belong to vertical backend
modules under `dbmd-backends::{sqlite, postgres}`. Each backend module owns its
source configuration type, catalog types, introspection, render-context mapping,
embedded backend templates, coverage documentation, and fixture tests.

`dbmd-backends` is the compile-time composition root. It owns the closed enums
and dispatch functions that let the application work with the backends compiled
into dbmd. Adding a backend changes this root and the application configuration
parser, but does not require adding vendor variants to core or render.

`dbmd-render` owns only presentation types, Markdown helpers, template execution,
and artifact assembly. It does not import backend catalogs. A backend converts
its catalog into the dedicated versioned render context and supplies a
namespaced template manifest. Templates whose behavior depends on one backend
live beside that backend; genuinely common artifact/object templates remain in
`dbmd-render`.

No public driver trait or runtime plugin ABI is introduced. A trait will be
considered only when a real substitutability consumer exists.

## Consequences

- Backend catalogs can represent vendor semantics honestly without widening a
  universal enum hierarchy.
- Core and render do not change when a backend gains a new catalog feature.
- The composition root intentionally changes when a compiled-in backend is
  added; this is explicit compile-time wiring rather than accidental coupling.
- Some relational leaf value types may be shared inside `dbmd-backends` when
  their semantics are genuinely equivalent.
- Backend-specific template files are part of a complete custom profile and are
  validated before connecting to selected sources.
- Cross-backend policy and output features operate on explicit presentation or
  capability seams rather than reaching through concrete catalog structs.

## Alternatives considered

### One universal relational catalog in core

Rejected because backend extensions accumulate throughout common objects and
force every consumer to understand every vendor.

### Duplicate all concepts independently

Rejected as a rule. Equivalent leaf values such as source identity, foreign-key
actions, and index ordering can be shared without sharing aggregate catalogs.

### Runtime backend plugins

Deferred. A stable ABI, discovery, version negotiation, and third-party template
contract would add substantial product surface without a present consumer.
