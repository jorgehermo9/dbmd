# dbmd

dbmd is a Rust CLI that turns live database structure into deterministic,
agent-readable Markdown. Commit the generated `DATABASE.md` or `database/`
artifact beside the application so agents and humans can inspect current schema
state without replaying migrations.

SQLite and PostgreSQL are supported through concrete introspection adapters.
Output can use the embedded agent profile or a complete project-owned template
set, one file or a deterministic directory tree, and one or more ordered named
sources.

## Canonical workflow

```sh
dbmd init
dbmd render
dbmd verify
```

- `init` creates a safe-to-commit `dbmd.toml` and conservatively detects one
  obvious local SQLite database.
- `render` resolves configured sources, introspects them, renders Markdown in
  memory, and atomically replaces the owned artifact.
- `verify` renders without modifying the artifact and exits unsuccessfully when
  bytes or directory file sets differ. Add `--diff` for a unified diff.

The minimal configuration is:

```toml
[sources.local]
backend = "sqlite"
path = "dev.db"

[output]
path = "DATABASE.md"
```

PostgreSQL credentials stay outside committed config:

```toml
[sources.app]
backend = "postgres"
url = "${DATABASE_URL}"

[output]
path = "DATABASE.md"
sources = ["app"]
```

## Product commands

```sh
# Inspect config resolution without connecting or printing secret values.
dbmd explain

# Check config, environment, output safety, and templates locally.
dbmd doctor

# Explicitly add connection, permission, compatibility, and introspection checks.
dbmd doctor --connect

# Preview or safely install an idempotent agent-instruction block.
dbmd init agents
dbmd init agents --file AGENTS.md

# Export the complete embedded template profile for customization.
dbmd init-templates

# Generate a protected repository-local GitHub Actions verification workflow.
dbmd init ci

# Render one SQLite database without project configuration.
dbmd render --backend sqlite --path dev.db --stdout
```

Render and explain accept repeated `--source` flags in requested order, an
alternate `--output`, `--stdout`, and `--template-root`. Canonical `verify`
accepts no output-shaping overrides.

## Documentation

- [Product overview](docs/product/overview.md)
- [Product concepts](docs/product/concepts.md)
- [Feature specifications](docs/product/features/README.md)
- [Architecture overview](docs/architecture/overview.md)
- [Complete documentation index](docs/README.md)

The canonical domain glossary lives in [CONTEXT.md](CONTEXT.md). Backend schema
coverage and known gaps live beside each adapter:
[SQLite](crates/backends/src/sqlite/README.md) and
[PostgreSQL](crates/backends/src/postgres/README.md).

## Development

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --all-targets
```

PostgreSQL container suites:

```sh
cargo test -p dbmd-backends --features postgres-tests --test postgres
cargo test -p dbmd-app --features postgres-tests --test postgres
```

The workspace uses Rust 2021 and is licensed under Apache-2.0.
