# Feature Specifications

These documents define observable product behavior. Each document states its implementation status so planned behavior is not mistaken for an available command.

## Core lifecycle

- [Render](render.md) — introspection, source selection, output writing, and artifact ownership.
- [Initialize](init.md) — project config, template export, CI setup, and agent instructions.
- [Verify](verify.md) — exact drift detection and CI output.
- [Multiple sources](multi-source.md) — source identity, ordering, headings, and directory nesting.
- [Templates and profiles](templates.md) — embedded profiles, custom template sets, and compatibility boundaries.

## Diagnostics and policy

- [Doctor](doctor.md) — setup and execution health.
- [Explain](explain.md) — resolved configuration and render planning.
- [Lint](lint.md) — schema quality and agent-friendliness.

## Integrations and optional artifacts

- [CI and agent integration](ci-and-agent-integration.md) — GitHub Actions, pre-commit, and generated agent guidance.
- [Statistics and manifests](statistics-and-manifest.md) — volatile or machine-readable companion artifacts.

## Status vocabulary

- Implemented: available in the CLI and covered by tests.
- Partial: a bootstrap form exists but does not satisfy the specification.
- Accepted: product behavior is chosen but not necessarily implemented.
- Proposed: direction is useful but may change after real usage.
- Deferred: explicitly outside the current milestone.
