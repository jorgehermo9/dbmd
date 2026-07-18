# Verify

## Purpose

`dbmd verify` answers one question: does the committed canonical artifact exactly match a fresh render from the canonical project contract?

## Canonical behavior

```sh
dbmd verify
dbmd verify --config path/to/dbmd.toml
dbmd verify --diff
```

Verify accepts operational flags such as `--config` and diagnostic verbosity. It does not accept output-shaping overrides that would redefine the artifact under test.

## Process

1. Run shared config, environment, template, selection, and connection preflight.
2. Introspect the canonical selected sources.
3. Render into a temporary file or directory.
4. Compare the temporary artifact with the configured destination.
5. Exit successfully only when bytes and file sets match.

Verify never creates, removes, or edits the configured output.

## Comparison semantics

- Single-file comparison is byte-for-byte.
- Directory comparison covers relative paths and file bytes.
- Missing committed output is drift, not a setup error.
- Extra stale files inside a dbmd-owned output directory are drift.
- Manually edited but semantically equivalent Markdown is drift.
- Timestamps and metadata lines receive no special treatment because default artifacts do not contain them.

Whitespace-normalized or semantic comparison is outside the verification contract.

## Default drift report

Output is compact and CI-friendly:

```text
error: canonical artifact has drifted

Changed:
  modified  database/app/tables/users.md
  added     database/app/tables/posts.md
  deleted   database/app/tables/old_table.md

Run:
  dbmd render
```

The summary uses full status words rather than git's single-letter codes.

## Diff mode

`dbmd verify --diff` prints a git-style unified diff and still exits non-zero when drift exists. Explicit diff mode is not truncated by default and does not require a built-in pager.

## Failure categories

Setup, connection, introspection, and rendering failures are not reported as drift. The command must distinguish “the artifact differs” from “dbmd could not produce a trustworthy comparison.”
