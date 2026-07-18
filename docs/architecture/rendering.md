# Rendering Architecture

Status: deterministic embedded SQLite rendering and atomic single-file output
implemented; dedicated render context, profiles, and directory artifacts remain
target architecture.

## Current implementation

`dbmd-render` embeds `database.md.j2` and `table.md.j2`, configures MiniJinja with
strict undefined behavior, serializes one core `SourceSnapshot`, and builds
fallible per-table presentation values such as `qualified_name` and the
ClickHouse `engine_clause`. The application resolves configured SQLite sources,
renders them in deterministic order, and atomically replaces one output file.

This is an appropriate bootstrap but has intentional limitations:

- Internal serde shape leaks into templates.
- Each render call accepts one source snapshot rather than a complete database
  context; the application currently concatenates selected source documents.
- Only one implicit profile and single-file layout exist.
- Core and introspection enforce collection ordering, but there is not yet a
  versioned presentation-facing render context.
- Directory artifacts do not exist.

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

Custom roots are complete sets, not overlays. Only the entrypoints required by the selected layout and emitted artifact kinds are mandatory. Partials are private to each set.

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

Rendering and writing are separate. The single-file path is implemented; the
remaining entries describe the complete target:

- Stdout receives only a resolved single-file artifact.
- Single-file output uses a temporary sibling file and rename.
- Directory output uses a temporary sibling tree and replacement.
- Verify compares a temporary artifact through the same render path and never calls the canonical writer.

Path validation and destructive ownership rules live in command/output orchestration, not inside templates.

## Compatibility milestones

1. Stabilize default SQLite output with internal templates.
2. Introduce an explicit render-context type and snapshots.
3. Document a context version before promoting custom templates as compatible product surface.
4. Add directory object entrypoints.
5. Add additional profiles only when their differences can be maintained with golden tests.
