# Lint

Status: accepted product scope; deferred until render and verify are stable.

## Purpose

`dbmd lint` answers: is this database structure well documented and agent-friendly?

Lint evaluates policy. It is separate from doctor so setup failures are not confused with opinions about schema quality, and separate from verify so artifact freshness does not silently become a quality gate.

## Candidate rules

- Missing table or column comments.
- Ambiguous columns such as `status` without documented values or constraints.
- Foreign-key-looking columns without declared foreign keys.
- Foreign keys without useful indexes where the backend makes that relevant.
- Tables without primary keys.
- Undocumented views or functions.
- PostgreSQL enum/check values omitted from output.
- ClickHouse tables with missing or suspicious `ORDER BY`.
- `ReplacingMergeTree` tables without clear version/deletion semantics.
- Functions missing volatility or other behavior metadata where available.

## Policy model

Rules need:

- Stable identifiers.
- Backend applicability.
- Default severity.
- Per-project severity overrides.
- Allowlists or scoped suppressions.
- Actionable messages with schema-object identity.

Lint warnings must not alter generated Markdown unless a profile explicitly chooses to render them.

## Exit behavior

Configured error-severity findings exit non-zero. Warnings are reported without failing by default. Invalid lint configuration is an operational error, not a lint finding.

## Open decisions

- Initial rule set and default severities.
- Suppression syntax and whether suppressions live in `dbmd.toml` or a separate policy file.
- Human text, JSON, SARIF, or a staged combination for CI consumption.
