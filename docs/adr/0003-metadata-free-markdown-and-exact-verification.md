# ADR-0003: Keep Generated Markdown Metadata-Free and Verify Exact Output

Status: accepted

Date: 2026-07-18

## Context

Generated headers, versions, timestamps, and fingerprints can signal ownership or freshness. They also consume agent context, create diff churn, and tempt verification to trust metadata rather than actual generated content.

Semantic or whitespace-normalized comparison could reduce formatting-only failures, but it would make verification behavior more complex and allow manual edits to diverge from the renderer's canonical result.

## Decision

Default committed Markdown contains schema context only. It does not contain timestamps, dbmd versions, fingerprints, schema hashes, or generated-by headers.

Freshness is established by rendering from canonical config into a temporary location and comparing exact file bytes and directory file sets. Missing, changed, added, and stale files are drift.

## Consequences

- Agent context is spent on database structure rather than tool boilerplate.
- Repeated output must be deterministic across unchanged inputs.
- Manual edits fail verification even when semantically equivalent.
- Verification does not need to parse Markdown or trust embedded claims.
- Tool/version metadata may require a separate manifest if a concrete machine consumer appears.

## Alternatives considered

### Generated-by and version headers

Rejected by default because ownership is already established through config, generated paths, workflow, and verification.

### Timestamped artifacts

Rejected because they guarantee churn and defeat exact comparison.

### Embedded schema fingerprints

Rejected for MVP because they create a second value that can drift and are unnecessary when actual output is regenerated.

### Normalized or semantic comparison

Deferred because deterministic generation is simpler, more transparent, and ensures the committed artifact is exactly reproducible.
