# Render

Status: configured SQLite single-file and directory rendering implemented;
one-off flags, stdout, custom profiles, and non-SQLite backends remain.

## Purpose

`dbmd render` produces an agent-readable artifact from current database structure. A plain invocation uses the canonical project contract in `dbmd.toml` and replaces the configured output.

## Configured usage

```sh
dbmd render
dbmd render --config path/to/dbmd.toml
```

Resolution order is:

1. CLI overrides.
2. Project configuration.
3. Built-in optional defaults.

Output-shaping overrides are permitted for one-off renders. They do not redefine what a later plain `dbmd verify` checks.

Today the only CLI override is `--config`; the remaining overrides in this
specification are planned behavior.

## One-off usage

Status: planned.

A developer should be able to try dbmd without first creating a config file:

```sh
dbmd render --backend sqlite --path ./dev.db --output DATABASE.md
```

Backend-specific connection arguments are mutually exclusive with conflicting configured source selection. Validation occurs before opening a database connection.

## Preflight

Before introspection, render validates:

- Config syntax and schema.
- Source IDs and selected source existence.
- Non-empty source selection.
- Required environment variables.
- Backend-specific connection fields.
- Layout and destination compatibility.
- Template root, selected profile, and required entrypoints.
- Output path safety.

Connection and introspection errors remain distinct from local preflight failures.

## Source selection and order

The [multiple-sources specification](multi-source.md#selection) owns source identity, selection precedence, validation, and deterministic ordering. Render applies that resolved order without reinterpretation.

## Output destinations

Status: configured single-file and directory output implemented; stdout
planned.

`--stdout` is valid only when the resolved layout produces one file. It does not change the selected layout.

Without `--stdout`, render writes to the resolved output path:

- `single_file` renders one Markdown file.
- `directory` renders a complete dbmd-owned directory tree.

Missing parent directories are created after path validation.

## Artifact ownership and safety

A configured output file is dbmd-owned and may be replaced unconditionally. A directory output is fully dbmd-owned and must not contain user-maintained files.

Render refuses dangerous or nonsensical directory paths, including:

- Empty paths, `.`, `..`, or `/`.
- The repository root.
- The user's home directory.
- `.git` or a path inside `.git`.
- A symlink used as the output root.

Render does not prompt before replacing a valid configured destination. Prompts would make automation inconsistent; the committed config is the authorization boundary.

Writes are best-effort atomic:

- A file is rendered to a temporary sibling and renamed over the destination.
- A directory is rendered to a temporary sibling tree and swapped into place only after successful generation.

Failure before replacement must leave the previous canonical artifact intact when the platform permits it.

## Default content

The default `agent` profile includes supported instances of:

- Sources and namespaces.
- Tables, columns, comments, constraints, and indexes.
- Views, materialized views, functions, enums, and extensions where supported.
- Fully qualified foreign-key targets and actions.
- Backend facts that affect query shape or performance.

Examples include SQLite generated columns and `WITHOUT ROWID`, PostgreSQL index predicates and row-level security, and ClickHouse engines, keys, TTLs, codecs, and settings.

Committed Markdown excludes timestamps, hashes, dbmd versions, and generated-by headers by default. Source identity appears as schema context when multiple sources require disambiguation.

## Determinism

Normalization establishes stable source, namespace, object, column, constraint, index, and backend-map order before rendering. Templates must not depend on database catalog row order or hash-map iteration.

Repeated renders of unchanged structure, configuration, profile, and templates must be byte-identical.

## Errors

Errors identify the owning stage:

- Configuration.
- Environment expansion.
- Validation.
- Connection.
- Introspection.
- Normalization.
- Template loading or execution.
- Output writing.

Messages should name the source and relevant path without printing credentials.

## Open decisions

- Whether archive or manifest stdout modes are useful for directory layout after MVP.
- Whether subset filters belong under each source, under output selection, or both.
- Whether directory `sections` should ship beside `objects` or follow later.
