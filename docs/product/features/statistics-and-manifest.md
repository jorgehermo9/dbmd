# Statistics and Manifests

## Principle

The canonical Markdown artifact contains stable structural context. Volatile statistics and tool-oriented metadata do not appear there by default.

## Statistics

Potential statistics include approximate row counts, table sizes, cardinality estimates, and last-analyzed times. These values create noisy diffs and may require elevated permissions or expensive queries.

Statistics use a separate artifact such as `DATABASE.stats.md` or a
machine-readable equivalent. The artifact identifies collection source and time
because volatility is part of its meaning.

Statistics collection is opt-in and must not run implicitly during ordinary render or verify.

## Manifest

A manifest may list generated paths, source IDs, hashes, and format versions for
external tooling. Canonical verification does not depend on a manifest: dbmd
compares actual temporary files with the dbmd-owned configured path.

A manifest should be added only when a concrete consumer needs machine-readable discovery. It must not force agents to parse metadata before reading schema context or become a second truth that can drift from Markdown.

## Generated Markdown metadata

Default Markdown includes no:

- Timestamp.
- Fingerprint or schema hash.
- dbmd version.
- Generated-by header or comment.

Source headings and namespace qualification are schema context, not tool metadata.
