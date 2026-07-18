# Templates and Profiles

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
- `agent-compact` — more aggressive context-window optimization.
- `human` — optional human-oriented formatting without changing snapshot semantics.

The embedded template set provides the `agent` profile.

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

The loader requires the complete set before connection, independent of
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
core model structs. Context versioning identifies shape but does not constitute
a stable external compatibility guarantee.

`explain` exposes the resolved template root, profile, layout, and required
entrypoints without exposing expanded environment values. A machine-readable
example context is outside this contract.
