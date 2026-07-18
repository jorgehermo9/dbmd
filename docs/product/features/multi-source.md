# Multiple Sources

Status: accepted MVP behavior; not implemented.

## Configuration

Sources use one canonical named shape:

```toml
[sources.analytics]
display_name = "Analytics"
backend = "clickhouse"
url = "${CLICKHOUSE_URL}"
database = "default"

[sources.app]
backend = "postgres"
url = "${DATABASE_URL}"

[sources.local]
backend = "sqlite"
path = "./dev.db"
```

There is no singular `[source]` shorthand in MVP. Backend fields live directly inside each source table while all supported sources are connection-backed.

## Identity

The table key is the stable source ID. `display_name` is optional presentation text and does not affect CLI selection, ordering references, output paths, or verification identity.

Source IDs accept ASCII letters, numbers, `_`, and `-`. dbmd rejects identifiers that would require slugification or could escape an output directory.

## Selection

```toml
[output]
sources = ["analytics", "app"]
```

- Omitted selection renders all configured sources sorted by source ID.
- Configured selection renders only the listed sources in list order.
- Repeated CLI `--source` flags replace config selection and preserve flag order.
- Empty selection and duplicate source IDs in an explicit order are invalid.

## Single-file layout

With `source_layout = "auto"`:

- Exactly one source omits a redundant source wrapper.
- Multiple sources receive explicit source sections.

With `source_layout = "nested"`, even one source receives a source section. Multiple selected sources still produce one file.

## Directory layout

With `source_layout = "auto"`:

- One source writes objects directly beneath the output root.
- Multiple sources receive stable source-ID directories.

With `source_layout = "nested"`, source directories are always present.

```text
database/
  index.md
  app/
    index.md
    tables/public.users.md
  analytics/
    index.md
    tables/default.events.md
```

Direct source-ID directories are preferred over an extra `sources/` wrapper. Display names may appear in headings but never in paths.

## Failure isolation

The canonical render succeeds only if every selected source succeeds. Partial canonical artifacts are unsafe because they look complete. Errors identify the failing source without exposing its credentials.

One-off future modes may permit partial diagnostic output, but they must not write the canonical artifact.

## Deferred source kinds

SQL dumps, prior snapshots, command output, and catalog JSON are deferred. Adding them requires an explicit source-kind model rather than overloading connection fields.

## Open decisions

- Location and precedence of source-specific object filters.
- Whether display names ever support limited variables; the default remains plain text.
- Whether Unicode display names need additional rendering rules. Source IDs remain ASCII slugs for MVP.
