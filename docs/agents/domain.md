# Domain Documentation

This is a single-context repository.

## Layout

```text
/
├── CONTEXT.md
└── docs/
    └── adr/
```

- Read root [CONTEXT.md](../../CONTEXT.md) before naming product concepts in issues, tests, architecture proposals, or user-facing output.
- Read relevant records under [docs/adr/](../adr/README.md) before changing the corresponding product or architecture boundary.
- There is no `CONTEXT-MAP.md` and no per-crate context split.

## Use the glossary

Use the glossary's canonical term when it defines a concept. In particular, distinguish source, source ID, source snapshot, database context, canonical artifact, namespace, observed fact, effective fact, unknown fact, and drift.

Do not substitute “database documentation” when the intended concept is the agent-readable artifact, and do not use “schema” for every backend namespace.

If required language is missing, use the domain-modeling workflow to resolve and record it rather than inventing competing synonyms in one feature.

## Respect decisions

If proposed work conflicts with an accepted ADR, surface the conflict explicitly. Do not silently rewrite the decision or treat older exploratory prose as stronger than an ADR.

When a decision changes, add a superseding ADR and update owning product/architecture docs to link to it.
