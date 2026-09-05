# dbmd Domain Glossary

This file defines the canonical product language for dbmd. It intentionally contains no implementation details.

## Terms

### Agent-readable artifact

The Markdown output produced by dbmd. It prioritizes explicit database semantics, compact navigation, deterministic diffs, and efficient use of an agent's context window. Use this term instead of the broader “database documentation” when referring to dbmd's primary output.

### Backend

The database family whose metadata rules dbmd understands, such as SQLite, PostgreSQL, or ClickHouse. A backend determines how a source is introspected and which semantics must be preserved.

### Catalog

The backend-owned, normalized structural content inside a source snapshot. A catalog is not a copy of vendor catalog rows: it expresses the database family's semantics in stable dbmd types while retaining raw definitions where they are the fidelity backstop.

### Canonical artifact

The one output destination declared by a project's committed configuration. It is the artifact that `render` updates and `verify` checks. Alternate one-off renders are not canonical artifacts.

### Database context

The ordered collection of source snapshots selected for one operation. It contains the stable schema information an agent needs to reason correctly about the selected databases: objects, relationships, constraints, indexes, definitions, comments, and backend-specific behavior. It does not include project configuration, credentials, templates, output policy, or volatile operational statistics by default.

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

### Schema object

A database object represented by dbmd, such as a table, view, materialized view, function, enum, extension, constraint, or index.

### Schema surface

The persistent and connection-visible database structures a backend can expose for introspection. Schema-surface coverage concerns the resulting objects and semantics, not the historical DDL operations that created them.

### Source

A configured database connection that dbmd can introspect. Every source has a stable source ID, a backend, connection settings, and an optional display name.

### Source ID

The filesystem-safe key that identifies a source in configuration, CLI selection, generated paths, and verification. A display name never replaces the source ID.

### Source snapshot

The normalized, point-in-time structural description produced by introspecting one source. It is a common identity envelope containing the source ID, optional display name, and exactly one backend-owned catalog.

### Template set

A complete collection of templates selected from embedded defaults or a custom template root. A custom template set owns its entrypoints and partials; it is not an overlay on embedded templates.

### Unknown fact

A relevant fact that dbmd cannot determine confidently. Unknown is distinct from absent, empty, false, or a backend default.

## Relationships

- A project configuration declares one or more sources and one canonical artifact.
- Introspecting a source produces a source snapshot.
- Selecting and ordering source snapshots produces a database context.
- A profile, output layout, and template set transform a database context into an agent-readable artifact.
- Comparing a fresh artifact with the canonical artifact detects drift.
