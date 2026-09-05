# Initialize

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

It must not silently overwrite an existing config. Base initialization has no
force mode and does not require interactive prompting.

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

CI initialization targets GitHub Actions. The generated workflow installs a
pinned dbmd version, makes required source credentials available through
documented secrets, and runs `dbmd verify`.

The command refuses conflicting user-maintained workflows unless `--force` is
explicitly supplied.

## Agent instructions

```sh
dbmd init agents
dbmd init agents --file AGENTS.md
```

Without `--file`, the command prints a complete marked snippet. With an explicit
file it creates or replaces only the `<!-- dbmd:begin -->` through
`<!-- dbmd:end -->` block and preserves all unrelated text. Repeating the
command is idempotent. Symlinks, non-regular files, duplicate markers, and
malformed marker blocks are rejected.

The generated guidance tells agents:

- Where the canonical artifact lives.
- To read it before reconstructing schema state from migrations.
- To treat `dbmd verify` failure as a request to regenerate and review changes.
- Not to edit generated artifacts manually.

Generating the snippet only parses the configured output field, preserves any
`${NAME}` reference literally, and never requires database credentials or
expanded environment values.
