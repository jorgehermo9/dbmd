set shell := ["bash", "-euo", "pipefail", "-c"]

# List the supported development workflows.
default:
    @just --list

# Format Rust and Just sources.
fmt:
    cargo fmt --all
    just --fmt --unstable

# Check Rust and Just formatting without changing files.
fmt-check:
    cargo fmt --all -- --check
    just --fmt --unstable --check

# Validate the GitHub Actions workflow and its shell steps.
workflow-lint:
    @command -v actionlint >/dev/null || { printf 'actionlint is required: https://github.com/rhysd/actionlint\n' >&2; exit 127; }
    actionlint

# Run every test layer, or one backend's adapter and application slice.
test backend="all" snapshots="check": (_run-test-suite backend snapshots)

# Run workspace unit tests only.
test-unit snapshots="check": (_run-unit snapshots)

# Run every integration subtype, or one backend's compatibility and application slice.
test-integration backend="all" snapshots="check": (_run-integration-suite backend snapshots)

# Run hermetic crate integration tests only.
test-integration-hermetic snapshots="check": (_run-integration-hermetic snapshots)

# Run real-database compatibility integrations for every backend, or one selected backend.
test-integration-backend backend="all" snapshots="check": (_run-integration-backend backend snapshots)

# Run application integrations for every target, local targets, or one backend.
test-integration-application target="all" snapshots="check": (_run-integration-application target snapshots)

# Run compiled CLI end-to-end tests only.
test-e2e snapshots="check": (_run-e2e snapshots)

# Run executable example application integrations for all targets or one selected target.
test-examples target="all": (_run-examples target "check")

# Regenerate committed example artifacts for all targets or one selected target.
examples-update target="all": (_run-examples target "update")
    @printf 'Example artifacts were updated directly; review them as user-facing documentation.\n'

# Update snapshots for every test layer, or one backend slice.
snapshots backend="all": (_run-test-suite backend "update")
    @printf 'Snapshot files were updated directly; review their git diff before committing.\n'

# Run strict Clippy for the workspace, or one backend and its application slice.
lint backend="all": (_run-lint backend)

# Run formatting, workflow validation, strict Clippy, and tests.
check backend="all": fmt-check workflow-lint (lint backend) (test backend "check")

[positional-arguments]
[private]
_run-test-suite backend snapshots:
    #!/usr/bin/env bash
    set -euo pipefail

    backend="$1"
    snapshots="$2"

    require_nextest() {
        command -v cargo-nextest >/dev/null || {
            printf 'cargo-nextest is required: https://nexte.st/docs/installation/\n' >&2
            exit 127
        }
    }

    snapshot_environment() {
        case "$1" in
            check)
                export INSTA_UPDATE=no
                ;;
            update)
                export INSTA_UPDATE=always
                ;;
            *)
                printf 'unknown snapshot mode: %s\nexpected one of: check, update\n' "$1" >&2
                exit 2
                ;;
        esac
    }

    validate_backend() {
        case "$1" in
            all|sqlite|postgres|clickhouse|mysql|mariadb|duckdb)
                ;;
            *)
                printf 'unknown backend: %s\nexpected one of: all, sqlite, postgres, clickhouse, mysql, mariadb, duckdb\n' "$1" >&2
                exit 2
                ;;
        esac
    }

    require_nextest
    snapshot_environment "$snapshots"
    validate_backend "$backend"

    if [[ "$backend" != all ]]; then
        just _run-integration-suite "$backend" "$snapshots"
        exit 0
    fi

    just _run-unit "$snapshots"
    just _run-integration-suite all "$snapshots"
    just _run-e2e "$snapshots"

[positional-arguments]
[private]
_run-unit snapshots:
    #!/usr/bin/env bash
    set -euo pipefail
    command -v cargo-nextest >/dev/null || { printf 'cargo-nextest is required: https://nexte.st/docs/installation/\n' >&2; exit 127; }
    case "$1" in
        check) export INSTA_UPDATE=no ;;
        update) export INSTA_UPDATE=always ;;
        *) printf 'unknown snapshot mode: %s\nexpected one of: check, update\n' "$1" >&2; exit 2 ;;
    esac
    cargo nextest run --workspace --all-features --profile "${NEXTEST_PROFILE:-default}" -E 'kind(lib) | kind(bin)'

[positional-arguments]
[private]
_run-integration-suite backend snapshots:
    #!/usr/bin/env bash
    set -euo pipefail

    backend="$1"
    snapshots="$2"
    case "$backend" in
        all|sqlite|postgres|clickhouse|mysql|mariadb|duckdb) ;;
        *)
            printf 'unknown backend: %s\nexpected one of: all, sqlite, postgres, clickhouse, mysql, mariadb, duckdb\n' "$backend" >&2
            exit 2
            ;;
    esac

    if [[ "$backend" == all ]]; then
        just _run-integration-hermetic "$snapshots"
        just _run-integration-backend all "$snapshots"
        just _run-integration-application all "$snapshots"
    else
        just _run-integration-backend "$backend" "$snapshots"
        just _run-integration-application "$backend" "$snapshots"
    fi

[positional-arguments]
[private]
_run-integration-hermetic snapshots:
    #!/usr/bin/env bash
    set -euo pipefail
    command -v cargo-nextest >/dev/null || { printf 'cargo-nextest is required: https://nexte.st/docs/installation/\n' >&2; exit 127; }
    case "$1" in
        check) export INSTA_UPDATE=no ;;
        update) export INSTA_UPDATE=always ;;
        *) printf 'unknown snapshot mode: %s\nexpected one of: check, update\n' "$1" >&2; exit 2 ;;
    esac
    cargo nextest run --workspace \
        --exclude dbmd \
        --exclude dbmd-app \
        --exclude dbmd-backend-sqlite \
        --exclude dbmd-backend-postgres \
        --exclude dbmd-backend-clickhouse \
        --exclude dbmd-backend-mysql \
        --exclude dbmd-backend-mariadb \
        --exclude dbmd-backend-duckdb \
        --profile "${NEXTEST_PROFILE:-default}" \
        -E 'kind(test)'

[positional-arguments]
[private]
_run-integration-backend backend snapshots:
    #!/usr/bin/env bash
    set -euo pipefail
    command -v cargo-nextest >/dev/null || { printf 'cargo-nextest is required: https://nexte.st/docs/installation/\n' >&2; exit 127; }

    backend="$1"
    snapshots="$2"
    profile="${NEXTEST_PROFILE:-default}"

    case "$snapshots" in
        check) export INSTA_UPDATE=no ;;
        update) export INSTA_UPDATE=always ;;
        *) printf 'unknown snapshot mode: %s\nexpected one of: check, update\n' "$snapshots" >&2; exit 2 ;;
    esac

    run_backend() {
        local selected="$1"
        case "$selected" in
            sqlite|duckdb)
                cargo nextest run -p "dbmd-backend-${selected}" --profile "$profile" -E 'kind(test)'
                ;;
            postgres|clickhouse|mysql|mariadb)
                cargo nextest run -p "dbmd-backend-${selected}" --features "${selected}-tests" --profile "$profile" -E 'kind(test)'
                ;;
            *)
                printf 'unknown backend: %s\nexpected one of: all, sqlite, postgres, clickhouse, mysql, mariadb, duckdb\n' "$selected" >&2
                exit 2
                ;;
        esac
    }

    if [[ "$backend" == all ]]; then
        for selected in sqlite duckdb postgres clickhouse mysql mariadb; do
            run_backend "$selected"
        done
    else
        run_backend "$backend"
    fi

[positional-arguments]
[private]
_run-integration-application target snapshots:
    #!/usr/bin/env bash
    set -euo pipefail
    command -v cargo-nextest >/dev/null || { printf 'cargo-nextest is required: https://nexte.st/docs/installation/\n' >&2; exit 127; }

    target="$1"
    snapshots="$2"
    profile="${NEXTEST_PROFILE:-default}"

    case "$snapshots" in
        check) export INSTA_UPDATE=no ;;
        update) export INSTA_UPDATE=always ;;
        *) printf 'unknown snapshot mode: %s\nexpected one of: check, update\n' "$snapshots" >&2; exit 2 ;;
    esac

    run_application() {
        local selected="$1"
        case "$selected" in
            local)
                cargo nextest run -p dbmd-app --profile "$profile" -E 'kind(test)'
                ;;
            sqlite)
                cargo nextest run -p dbmd-app --profile "$profile" -E 'kind(test) & binary(=render)'
                just _run-examples sqlite check
                ;;
            duckdb)
                cargo nextest run -p dbmd-app --profile "$profile" -E 'kind(test) & binary(=duckdb)'
                just _run-examples duckdb check
                ;;
            postgres|clickhouse|mysql|mariadb)
                cargo nextest run -p dbmd-app --features "${selected}-tests" --profile "$profile" -E "kind(test) & (binary(=${selected}) | binary(=examples_${selected}))"
                ;;
            full)
                just _run-examples full check
                ;;
            *)
                printf 'unknown application integration target: %s\nexpected one of: all, local, sqlite, postgres, clickhouse, mysql, mariadb, duckdb, full\n' "$selected" >&2
                exit 2
                ;;
        esac
    }

    if [[ "$target" == all ]]; then
        run_application local
        for server_backend in postgres clickhouse mysql mariadb; do
            run_application "$server_backend"
        done
        run_application full
    else
        run_application "$target"
    fi

[positional-arguments]
[private]
_run-e2e snapshots:
    #!/usr/bin/env bash
    set -euo pipefail
    command -v cargo-nextest >/dev/null || { printf 'cargo-nextest is required: https://nexte.st/docs/installation/\n' >&2; exit 127; }
    case "$1" in
        check) export INSTA_UPDATE=no ;;
        update) export INSTA_UPDATE=always ;;
        *) printf 'unknown snapshot mode: %s\nexpected one of: check, update\n' "$1" >&2; exit 2 ;;
    esac
    cargo nextest run -p dbmd --profile "${NEXTEST_PROFILE:-default}" -E 'kind(test)'

[positional-arguments]
[private]
_run-examples target update:
    #!/usr/bin/env bash
    set -euo pipefail
    command -v cargo-nextest >/dev/null || { printf 'cargo-nextest is required: https://nexte.st/docs/installation/\n' >&2; exit 127; }

    target="$1"
    update="$2"
    profile="${NEXTEST_PROFILE:-default}"
    case "$update" in
        check)
            export DBMD_EXAMPLES_UPDATE=no
            ;;
        update)
            export DBMD_EXAMPLES_UPDATE=always
            ;;
        *)
            printf 'unknown example update mode: %s\nexpected one of: check, update\n' "$update" >&2
            exit 2
            ;;
    esac

    run_target() {
        local selected="$1"
        case "$selected" in
            local)
                cargo nextest run -p dbmd-app --profile "$profile" -E 'kind(test) & binary(=examples)'
                ;;
            sqlite)
                cargo nextest run -p dbmd-app --profile "$profile" -E 'kind(test) & binary(=examples) & test(/sqlite|layout|custom_template|canonical_lifecycle|embedded_multi_source|example_inventory/)'
                ;;
            duckdb)
                cargo nextest run -p dbmd-app --profile "$profile" -E 'kind(test) & binary(=examples) & test(/duckdb|embedded_multi_source|example_inventory/)'
                ;;
            postgres|clickhouse|mysql|mariadb)
                cargo nextest run -p dbmd-app --features "${selected}-tests" --profile "$profile" -E "kind(test) & binary(=examples_${selected})"
                ;;
            full)
                cargo nextest run -p dbmd-app --features full-examples --profile "$profile" -E 'kind(test) & binary(=examples_full)'
                ;;
            *)
                printf 'unknown example target: %s\nexpected one of: all, local, sqlite, postgres, clickhouse, mysql, mariadb, duckdb, full\n' "$selected" >&2
                exit 2
                ;;
        esac
    }

    if [[ "$target" == all ]]; then
        run_target local
        for selected in postgres clickhouse mysql mariadb full; do
            run_target "$selected"
        done
    else
        run_target "$target"
    fi

[positional-arguments]
[private]
_run-lint backend:
    #!/usr/bin/env bash
    set -euo pipefail

    backend="$1"
    case "$backend" in
        all)
            cargo clippy --workspace --all-targets --all-features -- -D warnings
            ;;
        sqlite|duckdb)
            cargo clippy -p "dbmd-backend-${backend}" --all-targets --all-features -- -D warnings
            app_test=render
            [[ "$backend" == duckdb ]] && app_test=duckdb
            cargo clippy -p dbmd-app --test "$app_test" --test examples -- -D warnings
            ;;
        postgres|clickhouse|mysql|mariadb)
            feature="${backend}-tests"
            cargo clippy -p "dbmd-backend-${backend}" --all-targets --features "$feature" -- -D warnings
            cargo clippy -p dbmd-app --features "$feature" --test "$backend" --test "examples_${backend}" -- -D warnings
            ;;
        *)
            printf 'unknown backend: %s\nexpected one of: all, sqlite, postgres, clickhouse, mysql, mariadb, duckdb\n' "$backend" >&2
            exit 2
            ;;
    esac
