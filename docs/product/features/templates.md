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
      backends/
        sqlite/source.md.j2
        postgres/source.md.j2
    directory/
      root.md.j2
      enum.md.j2
      table.md.j2
      view.md.j2
      trigger.md.j2
      function.md.j2
      backends/
        sqlite/source.md.j2
        postgres/source.md.j2
```

dbmd does not fall back from a missing custom entrypoint to an embedded template. This prevents accidental coupling to internal built-in paths.

## Required entrypoints

- `PROFILE/single_file/database.md.j2`.
- `PROFILE/directory/root.md.j2`.
- `PROFILE/directory/table.md.j2`, `view.md.j2`, `trigger.md.j2`, and
  `function.md.j2`, plus `enum.md.j2`.
- `PROFILE/{single_file,directory}/backends/<backend>/source.md.j2` for every
  backend compiled into dbmd.

The loader requires the complete set before connection, independent of
the selected layout or selected source. `dbmd init-templates` creates exactly
this tree for the current binary.

Builtin partial paths are internal implementation details. Custom sets own any partials they reference.

## Template behavior

- Undefined values fail loudly.
- Missing entrypoints known from config are reported before connection; entrypoints required only after introspection fail rendering before output replacement.
- Template errors identify the template and source span where available.
- Templates choose structure and presentation; they do not derive hidden backend semantics.
- Credentials and raw connection settings are never present in the render context.

## Render-context compatibility

Custom templates receive the dedicated, versioned render context rather than
core model structs. The common root template receives `context`:

| Field | Meaning |
| --- | --- |
| `context.version` | Integer render-context shape version. |
| `context.sources` | Sources in resolved operation order. |

Every source value has this common envelope:

| Field | Meaning |
| --- | --- |
| `id` | Stable source ID used for selection and nested paths. |
| `name` | Markdown-ready display name, falling back to the source ID. |
| `has_display_name` | Whether `name` came from an explicit configured display name. |
| `backend` | Stable backend tag such as `sqlite` or `postgres`. |
| `single_file_template` | Backend-owned entrypoint included by `single_file/database.md.j2`. |
| `directory_template` | Backend-owned source-index entrypoint selected internally by the renderer; available to templates for inspection but normally not dispatched manually. |
| `nested` | Whether source headings/directories are explicit for this render. |
| `data` | Opaque backend-owned presentation payload documented below. |

A single-file backend entrypoint receives `source`. A directory backend
entrypoint also receives `source`; each declared directory object template
receives `source`, `object`, `heading` (`#`), and `detail_heading` (`##`).
Generated object paths and object template selection are backend-owned and are
not derived by custom templates.

Backend payload references:

- [SQLite template context](../../../crates/backends/sqlite/README.md#template-context)
- [PostgreSQL template context](../../../crates/backends/postgres/README.md#template-context)

All catalog-derived strings in these payloads are already Markdown-ready.
Collections retain deterministic catalog order. Optional values are either a
string or null; an empty list means that object family has no represented
members.

Context versioning identifies shape but does not constitute an indefinitely
stable external compatibility guarantee. A field addition, removal, or meaning
change is a reviewed product-contract change: update the backend reference,
embedded templates, render-context snapshots, and context version when old
custom templates cannot continue to render correctly. Strict undefined-value
handling makes incompatible custom profiles fail before output replacement.
The context documented here is version `2`.

`explain` exposes the resolved template root, profile, layout, and required
entrypoints without exposing expanded environment values.
