# dbmd

dbmd is being built as a Rust CLI that turns live database structure into deterministic, agent-readable Markdown. The generated artifact will be committed beside the application so code agents and humans can inspect the current database structure without replaying migrations or querying a database during every coding task.

The product is in its bootstrap stage. The repository currently contains the normalized schema-model sketch, an embedded MiniJinja renderer, and a placeholder CLI. SQLite introspection and real project configuration are the next milestone.

## Intended workflow

```sh
dbmd init
dbmd render
dbmd verify
```

- `init` creates safe-to-commit project configuration.
- `render` introspects configured sources and writes the canonical Markdown artifact.
- `verify` regenerates in a temporary location and reports whether the committed artifact has drifted.

Only the placeholder `render` command exists today. See the [product roadmap](docs/product/roadmap.md) for the implementation sequence.

## Documentation

- [Product overview](docs/product/overview.md)
- [Product concepts](docs/product/concepts.md)
- [Feature specifications](docs/product/features/README.md)
- [Architecture overview](docs/architecture/overview.md)
- [Architecture decisions](docs/adr/README.md)
- [Complete documentation index](docs/README.md)

The project glossary lives in [CONTEXT.md](CONTEXT.md).

## Development

```sh
cargo test --workspace --all-targets
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
```

The workspace uses Rust 2021 and is licensed under Apache-2.0.
