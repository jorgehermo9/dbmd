# Templates and Profiles

Status: embedded bootstrap implemented; selectable profiles, custom roots, and directory entrypoints are accepted but not implemented.

## Purpose

Templates control presentation while normalized snapshots and render contexts carry product semantics. Teams can choose built-in profiles or own a complete custom template set.

## Selection precedence

1. CLI template-root override for a one-off render.
2. `[templates].dir` in project configuration.
3. Embedded templates.

The selected template set is validated before dbmd opens a database connection.

## Profiles

`output.profile` selects an arbitrary profile directory. Built-in product profiles are expected to include:

- `agent` — default balance of compactness and explicit semantics.
- `agent-compact` — more aggressive context-window optimization after the default stabilizes.
- `human` — optional human-oriented formatting without changing snapshot semantics.

Only `agent` is required for the first useful release.

## Custom template roots

A custom root is a complete, independent template set:

```text
templates/dbmd/
  agent/
    single_file/
      database.md.j2
    directory/
      index.md.j2
      table.md.j2
      view.md.j2
      function.md.j2
    partials/
      columns.md.j2
      clickhouse/
        table-engine.md.j2
```

dbmd does not fall back from a missing custom entrypoint to an embedded template. This prevents accidental coupling to internal built-in paths.

## Required entrypoints

- `PROFILE/single_file/database.md.j2` for single-file output.
- `PROFILE/directory/index.md.j2` and `table.md.j2` for directory object output.
- Additional object entrypoints become required when the selected snapshot contains those artifact types.

Builtin partial paths are internal implementation details. Custom sets own any partials they reference.

## Template behavior

- Undefined values fail loudly.
- Missing entrypoints known from config are reported before connection; entrypoints required only after introspection fail rendering before output replacement.
- Template errors identify the template and source span where available.
- Templates choose structure and presentation; they do not derive hidden backend semantics.
- Credentials and raw connection settings are never present in the render context.

## Render-context compatibility

The current bootstrap serializes internal core structs with a few computed additions. That shape is explicitly unstable.

Before custom templates are advertised as a supported compatibility surface, dbmd will introduce and document a dedicated render context. Compatibility can then be versioned independently from internal Rust model changes.

An `explain` or diagnostic mode should make the resolved template root, profile, layout, entrypoints, and context version visible. A future context-dump mode may support template authors if it can avoid exposing sensitive data.

## Open decisions

- When to declare the render context stable enough for compatibility guarantees.
- Exact directory entrypoints for enums, extensions, and source indexes.
- Whether a safe machine-readable example context should be emitted for template development.
