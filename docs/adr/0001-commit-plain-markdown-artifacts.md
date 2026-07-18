# ADR-0001: Commit Plain Markdown Artifacts

Status: accepted

Date: 2026-07-18

## Context

Code agents need accurate database structure during ordinary repository work. They can reconstruct it from migrations, connect to live databases, consume raw schema dumps, or query a dedicated server. Each approach adds credentials, turns, runtime dependencies, or context noise.

The artifact also needs to be reviewable when schema structure changes and consumable by ordinary file-reading tools across agent ecosystems.

## Decision

dbmd's primary product is plain Markdown generated from current database structure and committed beside application code.

The initial product is a CLI lifecycle, not an MCP server or hosted service. Live protocols may complement committed artifacts later but do not replace them as the canonical context surface.

## Consequences

- Agents can inspect structure without database credentials during most coding tasks.
- Schema changes appear in normal pull-request diffs.
- Deterministic ordering and compact presentation become product-critical.
- Teams must regenerate and verify artifacts to avoid staleness.
- Markdown cannot carry every machine-readable detail efficiently; optional companion formats require separate justification.

## Alternatives considered

### Replay migrations

Rejected as the primary surface because final state is expensive to reconstruct and migrations can diverge from live structure.

### Query live databases on demand

Rejected as the default because it requires credentials, consumes interaction turns, and makes agent behavior less repeatable.

### Raw backend schema dumps

Rejected as the final artifact because they are noisy, backend-shaped, and difficult to navigate within context windows.

### MCP-only service

Deferred because it introduces a live runtime dependency and reduces compatibility with ordinary repository tools and code review.
