---
name: create-examples
description: Author, change, or audit executable dbmd projects under examples/. Use for example configuration, schema setup, Compose services, walkthroughs, committed artifacts, or example-suite registration.
---

# Create Examples

Treat each example as executable product documentation first and reusable
verification input second. A reader must be able to understand the database,
configuration, and resulting agent-readable artifact without reading test code.

Before editing, read `CONTEXT.md`, the owning product specifications, and the
README for every represented backend. Load `create-tests` before changing the
example manifest, harness, expected artifacts, or executable verification; its
coverage matrix and test-layer rules remain authoritative.

## Information architecture

Keep examples within this vocabulary:

- `quickstart/` provides the shortest complete first experience.
- `backends/<backend>/` demonstrates a backend's supported schema surface.
- `full/` demonstrates the complete multi-backend product.
- `workflows/<workflow>/` teaches one cross-cutting product behavior.

An example directory contains only user-facing material. Keep suite manifests,
test helpers, and CI-only configuration under the owning test directory.

## Runnable project contract

Every runnable example provides:

- a `README.md` that explains the scenario, prerequisites, commands, important
  configuration choices, and what to inspect in the generated artifact;
- one or more real `dbmd*.toml` project configurations;
- readable, ordered schema sources under `schema/<source-id>/`;
- committed canonical artifacts at the paths declared by those configurations;
- a small local `justfile` exposing `render`, `verify`, and `down`.

`just render` owns all required setup before invoking dbmd. `just verify` makes
the same database state available before verifying. `just down` removes
example-owned databases and containers. Keep the recipes opinionated and avoid
forwarding the underlying tools' full option surfaces.

For SQLite and DuckDB, recreate disposable database files from the committed
schema sources. For PostgreSQL, ClickHouse, MySQL, and MariaDB, pin the exact
supported image in `compose.yaml`, mount the source's schema directory read-only
into the vendor initialization directory, and define a bounded health check.
Users never apply fixture DDL by hand.

Schema filenames use numeric prefixes when order matters. Keep DDL realistic,
commented where the feature is not self-evident, and organized by product
concept rather than by the tests that happen to consume it. Backend showcases
should exercise the full fixture-backed supported surface; document intentional
exclusions instead of implying unsupported semantics.

## Configuration and artifacts

Use stable source IDs and human display names deliberately. Connection secrets
belong in environment references with obviously fake local defaults supplied by
the example workflow. Never commit real credentials.

Demonstrate meaningful configuration choices rather than permutations. Add a
second config when it teaches a distinct output contract such as directory
layout; use workflow examples for cross-cutting concerns such as custom
templates or selection order.

Commit generated output so the example remains useful without execution.
Artifacts must be deterministic and exclude credentials, mapped endpoints,
temporary paths, timestamps, and host-specific values. Review generated diffs
as documentation, not merely as snapshots to accept.

## Executable verification

Register every runnable example in the application integration suite manifest.
The manifest describes only test orchestration that cannot be derived safely
from the project itself, such as schema-to-source mapping and special lifecycle
checks. Product behavior remains in the example's real configuration.

The suite must consume the same schema files as the user workflow and enforce:

- bidirectional manifest inventory;
- required README, config, schema, artifact, Just recipe, and Compose structure;
- exact supported server images and read-only initialization mounts;
- exact render output and fresh verification;
- repeated-render determinism;
- credential, endpoint, and temporary-path exclusion.

Place exhaustive backend fidelity assertions in backend compatibility
integrations. Example verification owns the documented application workflow.
Use CLI E2E only for representative process-boundary examples.

## Completion

Run the example's human workflow when its required tools are available, then
`just test-examples [target]`; regenerate committed output only through
`just examples-update [target]`. Run any representative CLI E2E case.
Run formatting and strict Clippy for changed Rust. Every coverage-matrix row
must be closed, every committed artifact inspected, and the example inventory
must be exact before completion.
