# Explain

Status: proposed and planned after configuration resolution exists.

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
- Planned output files when they can be known before introspection.
- Environment variable names that were required, never their values.

## Behavior

Explain performs local parsing and resolution by default. A future connection-aware mode may include backend versions or introspection plans, but it must be explicit.

Output-shaping overrides accepted by `render` are also accepted by `explain` so users can inspect a one-off plan before executing it.

## Formats

Human-readable text is the initial requirement. JSON is valuable for support tooling but should follow only after the resolved-config model stabilizes.

## Safety

DSNs, passwords, tokens, and expanded secret values are always redacted. Error messages may name missing environment variables.
