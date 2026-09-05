# Feature Specifications

These documents define durable observable product behavior. The root command
reference describes availability; feature specifications do not double as
implementation ledgers.

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
