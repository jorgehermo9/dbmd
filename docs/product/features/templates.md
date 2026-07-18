# Templates and Profiles

Status: embedded `agent`, arbitrary custom profile selection, complete custom
roots, directory entrypoints, preflight validation, CLI precedence, and
`init-templates` are implemented. Additional embedded profiles and a stable
external context-compatibility promise remain.

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
      root.md.j2
      index.md.j2
      enum.md.j2
      table.md.j2
      view.md.j2
      trigger.md.j2
      function.md.j2
```

dbmd does not fall back from a missing custom entrypoint to an embedded template. This prevents accidental coupling to internal built-in paths.

## Required entrypoints

- `PROFILE/single_file/database.md.j2`.
- `PROFILE/directory/root.md.j2` and `index.md.j2`.
- `PROFILE/directory/table.md.j2`, `view.md.j2`, `trigger.md.j2`, and
  `function.md.j2`, plus `enum.md.j2`.

The current loader requires the complete set before connection, independent of
the selected layout. `dbmd init-templates` creates exactly this tree.

Builtin partial paths are internal implementation details. Custom sets own any partials they reference.

## Template behavior

- Undefined values fail loudly.
- Missing entrypoints known from config are reported before connection; entrypoints required only after introspection fail rendering before output replacement.
- Template errors identify the template and source span where available.
- Templates choose structure and presentation; they do not derive hidden backend semantics.
- Credentials and raw connection settings are never present in the render context.

## Render-context compatibility

Custom templates receive the dedicated, versioned render context rather than
core model structs. The compatibility policy is still pre-release and may
change until a stable external contract is declared.

An `explain` or diagnostic mode should make the resolved template root, profile, layout, entrypoints, and context version visible. A future context-dump mode may support template authors if it can avoid exposing sensitive data.

## Open decisions

- When to declare the render context stable enough for compatibility guarantees.
- Exact directory entrypoints for enums, extensions, and source indexes.
- Whether a safe machine-readable example context should be emitted for template development.
