# Agent guidance

Read [CONTEXT.md](CONTEXT.md) before naming product or domain concepts. Start with [docs/README.md](docs/README.md) for the complete documentation map.

## Project documentation

### Product

Read [docs/product/overview.md](docs/product/overview.md) and [docs/product/concepts.md](docs/product/concepts.md) before planning product work. For observable command or feature behavior, read the owning specification under `docs/product/features/`.

### Architecture

Read [docs/architecture/overview.md](docs/architecture/overview.md) before changing implementation boundaries, then read the relevant focused document under `docs/architecture/`. Product specifications own user-visible behavior; architecture documents own implementation design. Accepted decisions under `docs/adr/` override older exploratory prose when they overlap.

### Planning scratchpads

Do not put temporal plans, implementation phases, progress ledgers, session
notes, or next-step lists in persistent product, architecture, agent, or skill
documentation. Use `docs/plan/<planning-session-slug>/` as a temporary
scratchpad when a planning session needs files. Remove that directory when the
plan is implemented, abandoned, or otherwise no longer needed. Persistent docs
describe durable product contracts, architecture, and accepted decisions only.

## Development workflows

Use the root [`justfile`](justfile) for formatting, linting, tests, and snapshot
updates. Run `just` to list recipes; prefer `just check [backend]`,
`just test [backend]`, and `just snapshots [backend]` over manual Cargo or
`INSTA_UPDATE` commands. Use `just test-examples [target]` and
`just examples-update [target]` for executable documentation. Optional
selectors default to `all`.

## Agent skills

### Rust coding

Any task that writes, modifies, reviews, or refactors Rust code must invoke and follow [`rust-skills`](.agents/skills/rust-skills/SKILL.md). Read the rule files relevant to the task before editing, prioritize critical and high-impact guidance, and run formatting, Clippy, and the applicable test suite afterward.

### Testing

Every code change must invoke and follow
[`create-tests`](.agents/skills/create-tests/SKILL.md), derive its coverage
matrix before implementation, and close every applicable row before completion.
Load the skill before authoring or editing any test, fixture, or snapshot; all
test work must follow its layer, fixture, assertion, and determinism guidance.

### Examples

Load [`create-examples`](.agents/skills/create-examples/SKILL.md) before
authoring, changing, or auditing anything under `examples/`. Example projects
are executable product documentation; their test-only manifests and harnesses
stay under the owning test suite. Load `create-tests` as well when executable
verification, expected artifacts, fixtures, or snapshots change.

### Issue tracker

Issues and PRDs are tracked in GitHub Issues for `jorgehermo9/dbmd`. External PRs are not a triage surface. See `docs/agents/issue-tracker.md`.

### Triage labels

Triage uses `needs-triage`, `needs-info`, `ready-for-agent`, `ready-for-human`, and `wontfix`. See `docs/agents/triage-labels.md`.

### Domain docs

This is a single-context repository with `CONTEXT.md` at the root and ADRs under `docs/adr/`. See `docs/agents/domain.md`.
