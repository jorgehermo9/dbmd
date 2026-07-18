# Initialize

Status: base SQLite initialization implemented; template, CI, and agent-snippet
initialization remain planned.

## Purpose

`dbmd init` creates a safe, understandable starting contract without placing credentials in committed files.

## Base initialization

```sh
dbmd init
```

The command creates `dbmd.toml` when no project configuration exists. It should:

- Detect an obvious local SQLite database only when discovery is unambiguous.
- Otherwise generate an example source that requires explicit user editing.
- Reference secrets through environment variables.
- Default to `DATABASE.md`, the `agent` profile, and `single_file` layout.
- Explain what was created and the next `dbmd render` command.

It must not silently overwrite an existing config. A future explicit force flag may replace generated config, but interactive prompting is not required for automation.

## Template initialization

```sh
dbmd init-templates
```

This command copies a complete built-in template tree into a project-selected directory, initially `templates/dbmd`. The copied tree is a customization starting point and becomes independently owned by the project.

It validates destination conflicts and never presents custom templates as overlays on embedded defaults.

## CI initialization

```sh
dbmd init ci
```

Initial support targets GitHub Actions. The generated workflow should install a pinned or explicitly selected dbmd version, make required source credentials available through documented secrets, and run `dbmd verify`.

The command reports files it would replace and requires an explicit overwrite option for conflicting user-maintained workflows.

## Agent instructions

Initialization may print or write snippets for `AGENTS.md` and `CLAUDE.md` that tell agents:

- Where the canonical artifact lives.
- To read it before reconstructing schema state from migrations.
- To treat `dbmd verify` failure as a request to regenerate and review changes.
- Not to edit generated artifacts manually.

Generated snippets must not overwrite unrelated instructions.

## Open decisions

- Exact force and dry-run behavior for existing files.
- Whether base `init` offers agent-instruction snippets immediately or through a subcommand.
- Whether non-GitHub CI generators are justified after the initial integration.
