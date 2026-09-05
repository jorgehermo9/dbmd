# Explain

## Purpose

`dbmd explain` shows how dbmd resolves a requested operation without requiring users to infer precedence across CLI flags, config, and built-in defaults.

## Expected output

- Config file location.
- Selected sources and deterministic order.
- Backend per source, with credentials redacted.
- Canonical versus overridden output path.
- Layout, directory variant, and source layout.
- Profile and template root.
- Required template entrypoints.
- Output files that can be determined before introspection.
- Environment variable names that were required, never their values.

## Behavior

Explain performs local parsing and resolution only. It does not connect to a
database; connection and compatibility diagnosis belongs to `doctor --connect`.

Output-shaping overrides accepted by `render` are also accepted by `explain` so users can inspect a one-off plan before executing it.

## Formats

The format is deterministic human-readable text. Structured JSON
is not part of the current command contract.

## Command

```sh
dbmd explain [--config dbmd.toml]
  [--source ID ...]
  [--output PATH | --stdout]
  [--template-root PATH]
```

These flags use the same config-relative path and precedence rules as
`dbmd render`.

## Safety

DSNs, passwords, tokens, and expanded secret values are always redacted. Error messages may name missing environment variables.
