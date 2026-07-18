# Agent guidance

Read [CONTEXT.md](CONTEXT.md) before naming product or domain concepts. Start with [docs/README.md](docs/README.md) for the complete documentation map.

Phase 1 is complete and Phase 2, the first useful SQLite render, is the active milestone. Do not describe placeholder rendering as live database introspection.

## Project documentation

### Product

Read [docs/product/overview.md](docs/product/overview.md), [docs/product/concepts.md](docs/product/concepts.md), and [docs/product/roadmap.md](docs/product/roadmap.md) before planning product work. For observable command or feature behavior, read the owning specification under `docs/product/features/`.

### Architecture

Read [docs/architecture/overview.md](docs/architecture/overview.md) before changing implementation boundaries, then read the relevant focused document under `docs/architecture/`. Product specifications own user-visible behavior; architecture documents own implementation design. Accepted decisions under `docs/adr/` override older exploratory prose when they overlap.

## Agent skills

### Rust coding

Any task that writes, modifies, reviews, or refactors Rust code must invoke and follow [`rust-skills`](.agents/skills/rust-skills/SKILL.md). Read the rule files relevant to the task before editing, prioritize critical and high-impact guidance, and run formatting, Clippy, and the applicable test suite afterward.

### Issue tracker

Issues and PRDs are tracked in GitHub Issues for `jorgehermo9/dbmd`. External PRs are not a triage surface. See `docs/agents/issue-tracker.md`.

### Triage labels

Triage uses `needs-triage`, `needs-info`, `ready-for-agent`, `ready-for-human`, and `wontfix`. See `docs/agents/triage-labels.md`.

### Domain docs

This is a single-context repository with `CONTEXT.md` at the root and ADRs under `docs/adr/`. See `docs/agents/domain.md`.
