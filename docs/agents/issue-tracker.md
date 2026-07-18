# Issue Tracker: GitHub

Issues and published PRDs for this repository live in [GitHub Issues](https://github.com/jorgehermo9/dbmd/issues). Use the `gh` CLI for issue operations from this clone.

## Repository

- Repository: `jorgehermo9/dbmd`
- Tracker: GitHub Issues
- External PRs as a triage request surface: no

External pull requests are not automatically pulled into the issue triage state machine. Review PRs through the normal pull-request workflow.

## Conventions

- Create: `gh issue create --title "..." --body "..."`
- Read with discussion: `gh issue view <number> --comments`
- List: `gh issue list --state open --json number,title,body,labels,comments`
- Comment: `gh issue comment <number> --body "..."`
- Add or remove labels: `gh issue edit <number> --add-label "..."` or `--remove-label "..."`
- Close with context: `gh issue close <number> --comment "..."`

Infer the repository from the current clone. Use explicit `--repo jorgehermo9/dbmd` when operating outside it.

## Skill operations

When a skill says “publish to the issue tracker,” create a GitHub issue. When it says “fetch the relevant ticket,” read the issue body, labels, and comments.

Specs and tickets should be self-contained enough for their intended triage state. Use the label mapping in [triage-labels.md](triage-labels.md).

## Wayfinding

The `wayfinder` skill uses one map issue with child issues:

- Label the map `wayfinder:map` when that label exists.
- Link tickets as GitHub sub-issues where supported; otherwise use a task list in the map and add `Part of #<map>` to each child.
- Represent blocking with GitHub issue dependencies where supported; otherwise begin the child body with `Blocked by: #<number>`.
- A frontier ticket is open, unassigned, and has no open blockers.
- Claim work with `gh issue edit <number> --add-assignee @me`.
- Resolve by recording the result, closing the child, and updating the map's decisions-so-far.

Create wayfinding-specific labels only when that workflow is first used; the five triage labels are the setup baseline.
