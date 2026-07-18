# Agent guidance

Read [CONTEXT.md](CONTEXT.md) before naming product or domain concepts. Start with [docs/README.md](docs/README.md) for product and architecture documentation. Architecture decisions under `docs/adr/` override older exploratory prose when they overlap.

The implementation is currently a Phase 1 bootstrap. Do not describe placeholder rendering as live database introspection.

## Agent skills

### Issue tracker

Issues and PRDs are tracked in GitHub Issues for `jorgehermo9/dbmd`. External PRs are not a triage surface. See `docs/agents/issue-tracker.md`.

### Triage labels

Triage uses `needs-triage`, `needs-info`, `ready-for-agent`, `ready-for-human`, and `wontfix`. See `docs/agents/triage-labels.md`.

### Domain docs

This is a single-context repository with `CONTEXT.md` at the root and ADRs under `docs/adr/`. See `docs/agents/domain.md`.
