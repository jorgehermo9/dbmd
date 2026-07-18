# Rendering Architecture

## Boundary

`dbmd-render` is backend-neutral. It owns versioned presentation structs,
Markdown escaping, strict MiniJinja execution, common artifact templates, and
in-memory artifact assembly. It does not import `dbmd-core` or any backend
catalog.

Each backend maps its catalog to a `RenderSource`. That mapping computes
qualified names, display-ready facts, backend summaries, trigger/function
details, paths, and Markdown-safe values before template execution.

```text
backend Catalog
  → backend render mapping
  → RenderSource
  → RenderContext
  → common + selected backend templates
  → RenderedArtifact
```

## Template ownership

Common templates that contain no database-family assumptions live in
`dbmd-render`, including artifact roots and common object layouts. Templates
whose structure or content depends on a backend live beside that backend under
`backends/src/<backend>/templates/` and use namespaced internal template names.

The compiled backend composition root supplies the manifests required by the
selected sources. This lets template completeness be validated from the
resolved source types before a database connection is opened.

## Custom roots

A custom root remains a complete independent set under ADR-0004. It contains
the common profile entrypoints plus backend files for every backend compiled
into dbmd. `dbmd init-templates` exports that complete tree. Missing files never
fall back to embedded content.

The durable external paths are documented in the
[template product contract](../product/features/templates.md). Internal
MiniJinja names are implementation details.

## Render context

The context contains stable external backend tags, source identity, qualified
names, safe path components, presentation-ready schema facts, and
deterministically ordered objects. It never contains credentials, connection
settings, environment values, driver handles, or internal errors.

Context version `1` identifies its shape but is not a promise that arbitrary
custom templates remain source-compatible forever. Changes are reviewed through
render-context and Markdown tests.

## Artifacts and layouts

`RenderedArtifact` is either one byte buffer or an ordered map of validated
relative paths to bytes. Rendering finishes in memory before output
replacement.

- Single-file layout renders all selected backend source fragments into one
  document.
- Directory layout renders a root/source index plus stable files for each
  represented object family.
- Source nesting follows the resolved source-layout policy.

The application owns stdout, atomic file/tree replacement, and exact
verification comparison. The renderer never owns canonical filesystem policy.

## Determinism and safety

- Catalog and context collections are ordered before templates execute.
- Templates use strict undefined behavior and no nondeterministic functions.
- Markdown tables escape catalog-provided pipes and line breaks.
- Code fences expand around stored SQL containing backticks.
- Generated paths use encoded safe components and reject absolute/traversal
  forms.
- Artifacts contain no timestamps, connection details, or generated metadata by
  default.
