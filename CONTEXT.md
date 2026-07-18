# dbmd Domain Glossary

This file defines the canonical product language for dbmd. It intentionally contains no implementation details.

## Terms

### Agent-readable artifact

The Markdown output produced by dbmd. It prioritizes explicit database semantics, compact navigation, deterministic diffs, and efficient use of an agent's context window. Use this term instead of the broader “database documentation” when referring to dbmd's primary output.

### Backend

The database family whose metadata rules dbmd understands, such as SQLite, PostgreSQL, or ClickHouse. A backend determines how a source is introspected and which semantics must be preserved.

### Canonical artifact

The one output destination declared by a project's committed configuration. It is the artifact that `render` updates and `verify` checks. Alternate one-off renders are not canonical artifacts.

### Database context

The stable schema information an agent needs to reason correctly about a database: objects, relationships, constraints, indexes, definitions, comments, and backend-specific behavior. Volatile operational statistics are not database context by default.

### Drift

A byte or file-set difference between the committed canonical artifact and a fresh render from the configured sources.

### Effective fact

A fact dbmd can state after applying a documented backend rule, even when the database catalog does not store it explicitly. An effective fact must not be presented as directly observed.

### Namespace

A backend-defined container used to qualify schema objects. PostgreSQL schemas, SQLite database names such as `main`, and ClickHouse databases are namespace examples. Use “schema” only for a backend concept explicitly named schema or for the full structural definition of a database.

### Observed fact

A fact read directly from database catalog metadata.

### Output layout

The file organization of an agent-readable artifact. The supported product concepts are `single_file` and `directory`; directory output may further use object-oriented or section-oriented files.

### Profile

A named presentation policy for generated artifacts, such as `agent`, `agent-compact`, or `human`. A profile changes rendering, not the underlying source snapshot.

### Project snapshot

The ordered collection of source snapshots selected for one render.

### Schema object

A database object represented by dbmd, such as a table, view, materialized view, function, enum, extension, constraint, or index.

### Source

A configured database connection that dbmd can introspect. Every source has a stable source ID, a backend, connection settings, and an optional display name.

### Source ID

The filesystem-safe key that identifies a source in configuration, CLI selection, generated paths, and verification. A display name never replaces the source ID.

### Source snapshot

The normalized, point-in-time structural description produced by introspecting one source. It contains schema objects and the backend facts needed to understand them.

### Template set

A complete collection of templates selected from embedded defaults or a custom template root. A custom template set owns its entrypoints and partials; it is not an overlay on embedded templates.

### Unknown fact

A relevant fact that dbmd cannot determine confidently. Unknown is distinct from absent, empty, false, or a backend default.

## Relationships

- A project configuration declares one or more sources and one canonical artifact.
- Introspecting a source produces a source snapshot.
- Selecting and ordering source snapshots produces a project snapshot.
- A profile, output layout, and template set transform a project snapshot into an agent-readable artifact.
- Comparing a fresh artifact with the canonical artifact detects drift.
