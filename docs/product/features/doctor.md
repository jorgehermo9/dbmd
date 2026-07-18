# Doctor

Status: accepted, planned after the core render and verify lifecycle.

## Purpose

`dbmd doctor` answers: can dbmd operate successfully in this project?

It diagnoses setup and execution prerequisites without applying schema-quality opinions or comparing committed artifacts.

## Checks

- Config existence, syntax, and schema.
- Valid source IDs and selected-source references.
- Required environment variables without printing their values.
- Backend-specific connection fields.
- Output path safety and writability.
- Template root, profile, entrypoints, and compilation under strict undefined behavior where possible.
- Connection ability when connection checks are enabled.
- Introspection permissions.
- Backend-version compatibility with required metadata queries.

By default, doctor checks sources selected by the canonical artifact. `--all-sources` broadens checks to configured but unselected sources.

## Command boundary

- Doctor diagnoses operational correctness.
- Verify checks artifact freshness.
- Lint evaluates schema quality.

Doctor may reuse shared preflight code, but it is not a superset of verify's drift comparison or lint's policy engine.

## Output

Diagnostics should be grouped by stage and source, with actionable fixes. A failing connection must not prevent reporting independent local config or template errors when those checks can run safely.

## Open decisions

- Whether connection checks run by default or require an explicit flag for safety and speed.
- Which checks can run concurrently without making output confusing.
- Whether structured JSON output should accompany human diagnostics.
