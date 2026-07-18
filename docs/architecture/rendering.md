# Rendering Architecture

Status: explicit render context, deterministic SQLite and PostgreSQL rendering,
layout-neutral artifacts, stdout, complete custom template roots, and atomic
single-file/directory output are implemented. Additional built-in profiles and
a public compatibility promise for custom contexts remain.

## Current implementation

`dbmd-render` builds a versioned `RenderContext` from a complete
`DatabaseContext`, configures MiniJinja with strict undefined behavior, and
returns a `RenderedArtifact`. Presentation values, Markdown-safe cells, dynamic
code fences, qualified names, and backend summaries are computed before
templates execute. The application atomically replaces one output file or a
complete generated tree.

Current limitations:

- `agent` is the only embedded profile; arbitrary profile directories are
  selectable from a complete custom root.
- A custom root must contain the complete eight-file profile currently emitted
  by `dbmd init-templates`.
- The context is explicitly versioned but not yet promised as a stable external
  compatibility contract.

## Target pipeline

```text
DatabaseContext
  → RenderContextBuilder
  → RenderContext
  → TemplateResolver
  → MiniJinja
  → RenderedArtifact
```

`RenderedArtifact` is layout-neutral to downstream commands:

```rust
pub enum RenderedArtifact {
    SingleFile(Vec<u8>),
    Directory(BTreeMap<RelativePath, Vec<u8>>),
}
```

The exact Rust type may differ. The important property is that rendering completes in memory or a staging location before canonical artifact replacement.

## Render context

The context is a presentation-facing compatibility boundary. It should contain:

- Stable external backend tags.
- Qualified names and safe path components.
- Display-ready types and expressions.
- Constraint and relationship summaries.
- Provenance notes derived from typed facts.
- Deterministically ordered object lists.
- Source sections and navigation targets appropriate to the selected layout.

It must not contain credentials, driver handles, raw environment values, or arbitrary internal error types.

Business semantics are computed before template execution. Templates may choose whether to display an optional provenance note, but cannot infer an effective primary key by comparing unrelated fields.

## Template resolution

Resolution inputs are:

- Template root: custom or embedded.
- Profile.
- Layout.
- Artifact kind.

Backend is context data and can select internal partials; it is not a required top-level entrypoint dimension.

Custom roots are complete sets, not overlays. The current loader validates the
complete profile before connection, including both layouts and all supported
object entrypoints. Partials are private to each set.

See the [template product contract](../product/features/templates.md) for paths and precedence.

## Layouts

### Single file

The main database template renders all selected sources into one byte stream. Source headings depend on source-layout policy.

### Directory objects

An index links to stable object paths. Each table, view, function, enum, or other supported object uses its corresponding entrypoint. Filenames include a namespace component when required to avoid collisions.

### Directory sections

Section-oriented files are a later variant for teams preferring fewer files. The in-memory artifact abstraction should not assume one object per file, but the first directory implementation may support only objects.

## Markdown policy

Default artifacts contain schema context only and do not include a generated-by
line.

Do not include timestamps, tool versions, fingerprints, schema hashes, or generated comments by default. Artifact ownership comes from configuration and exact verification.

Markdown tables and code blocks must escape or delimit catalog-provided content safely. Golden fixtures should cover pipes, backticks, multiline comments, non-ASCII names, and SQL definitions containing Markdown fences.

## Determinism

- Context collections are ordered before reaching templates.
- Ordered maps or sorted key/value vectors back all template iteration.
- Templates do not call non-deterministic functions.
- Newline style and trailing newline policy are fixed.
- Embedded template versions change output only through reviewed source changes.

## Error handling

Template loading errors name the selected root, profile, layout, and missing entrypoint. Execution errors preserve MiniJinja source spans and useful context paths without dumping the full potentially sensitive context.

Validation runs before database connection when the required template set can be known from config. Artifact-kind-specific entrypoints discovered after introspection produce rendering errors without replacing existing output.

## Output writer

Rendering and writing are separate:

- Stdout receives only a resolved single-file artifact.
- Single-file output uses a temporary sibling file and rename.
- Directory output uses a complete temporary sibling tree, replacement, and
  rollback on a failed final rename.
- Verify compares the in-memory artifact through the same render path and never
  calls the canonical writer.

Path validation and destructive ownership rules live in command/output orchestration, not inside templates.

## Compatibility milestones

1. Stabilize default SQLite output with internal templates. Complete.
2. Introduce an explicit render-context type and snapshots. Complete.
3. Document a context version before promoting custom templates as compatible product surface. Context version `1` exists; compatibility policy remains.
4. Add directory object entrypoints. Complete for tables, views, triggers, and functions.
5. Add additional profiles only when their differences can be maintained with golden tests.
