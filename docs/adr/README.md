# Architecture Decision Records

ADRs record decisions that are costly to reverse, surprising without context, and chosen through a real trade-off. Product and architecture docs link here instead of repeatedly re-explaining resolved choices.

## Records

- [ADR-0001: Commit plain Markdown artifacts](0001-commit-plain-markdown-artifacts.md)
- [ADR-0002: Use one canonical project artifact](0002-one-canonical-project-artifact.md)
- [ADR-0003: Keep generated Markdown metadata-free and verify exact output](0003-metadata-free-markdown-and-exact-verification.md)
- [ADR-0004: Treat custom template roots as complete sets](0004-custom-template-roots-are-complete-sets.md)

## Format

Each record contains:

- Status and date.
- Context and forces.
- Decision.
- Consequences.
- Alternatives considered.

Accepted ADRs remain in place if superseded. A later ADR links to and replaces the earlier decision rather than rewriting history.
