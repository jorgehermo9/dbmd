# Statistics and Manifests

Status: deferred and optional.

## Principle

The canonical Markdown artifact contains stable structural context. Volatile statistics and tool-oriented metadata do not appear there by default.

## Statistics

Potential statistics include approximate row counts, table sizes, cardinality estimates, and last-analyzed times. These values create noisy diffs and may require elevated permissions or expensive queries.

If implemented, statistics use a separate artifact such as `DATABASE.stats.md` or a machine-readable equivalent. The artifact identifies collection source and time because volatility is part of its meaning.

Statistics collection is opt-in and must not run implicitly during ordinary render or verify.

## Manifest

A future manifest could list generated paths, source IDs, hashes, and format versions for external tooling. MVP verification does not need it: dbmd can compare actual temporary files with the dbmd-owned configured path.

A manifest should be added only when a concrete consumer needs machine-readable discovery. It must not force agents to parse metadata before reading schema context or become a second truth that can drift from Markdown.

## Generated Markdown metadata

Default Markdown includes no:

- Timestamp.
- Fingerprint or schema hash.
- dbmd version.
- Generated-by header or comment.

Source headings and namespace qualification are schema context, not tool metadata.

## Open decisions

- The first concrete consumer that justifies a manifest.
- Whether stats belong to dbmd core or a separate optional command/package.
- Stable formats and retention expectations for volatile companion artifacts.
