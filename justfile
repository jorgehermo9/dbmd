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

# Run hermetic public-interface integration tests only.
test-integration snapshots="check": (_run-integration snapshots)

# Run adapter contracts for every backend, or one selected backend.
test-contract backend="all" snapshots="check": (_run-contract backend snapshots)

# Run compiled CLI end-to-end tests only.
test-e2e snapshots="check": (_run-e2e snapshots)

# Run workspace documentation tests.
test-doc:
    cargo test --workspace --all-features --doc

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

    run_app_backend() {
        local selected="$1"
        local profile="${NEXTEST_PROFILE:-default}"

        case "$selected" in
            sqlite)
                cargo nextest run -p dbmd-app --profile "$profile" -E 'kind(test) & binary(=render)'
                ;;
            duckdb)
                cargo nextest run -p dbmd-app --profile "$profile" -E 'kind(test) & binary(=duckdb)'
                ;;
            postgres|clickhouse|mysql|mariadb)
                cargo nextest run -p dbmd-app --features "${selected}-tests" --profile "$profile" -E "kind(test) & binary(=${selected})"
                ;;
        esac
    }

    require_nextest
    snapshot_environment "$snapshots"
    validate_backend "$backend"

    if [[ "$backend" != all ]]; then
        just _run-contract "$backend" "$snapshots"
        run_app_backend "$backend"
        exit 0
    fi

    just _run-unit "$snapshots"
    just _run-integration "$snapshots"
    just _run-contract all "$snapshots"
    for server_backend in postgres clickhouse mysql mariadb; do
        run_app_backend "$server_backend"
    done
    just _run-e2e "$snapshots"
    just test-doc

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
_run-integration snapshots:
    #!/usr/bin/env bash
    set -euo pipefail
    command -v cargo-nextest >/dev/null || { printf 'cargo-nextest is required: https://nexte.st/docs/installation/\n' >&2; exit 127; }
    case "$1" in
        check) export INSTA_UPDATE=no ;;
        update) export INSTA_UPDATE=always ;;
        *) printf 'unknown snapshot mode: %s\nexpected one of: check, update\n' "$1" >&2; exit 2 ;;
    esac
    cargo nextest run --workspace --profile "${NEXTEST_PROFILE:-default}" -E 'kind(test) - package(=dbmd) - package(=dbmd-backend-sqlite) - package(=dbmd-backend-postgres) - package(=dbmd-backend-clickhouse) - package(=dbmd-backend-mysql) - package(=dbmd-backend-mariadb) - package(=dbmd-backend-duckdb)'

[positional-arguments]
[private]
_run-contract backend snapshots:
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

    run_contract() {
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
            run_contract "$selected"
        done
    else
        run_contract "$backend"
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
            cargo clippy -p dbmd-app --test "$app_test" -- -D warnings
            ;;
        postgres|clickhouse|mysql|mariadb)
            feature="${backend}-tests"
            cargo clippy -p "dbmd-backend-${backend}" --all-targets --features "$feature" -- -D warnings
            cargo clippy -p dbmd-app --features "$feature" --test "$backend" -- -D warnings
            ;;
        *)
            printf 'unknown backend: %s\nexpected one of: all, sqlite, postgres, clickhouse, mysql, mariadb, duckdb\n' "$backend" >&2
            exit 2
            ;;
    esac
