set shell := ["bash", "-euo", "pipefail", "-c"]

# List the supported development workflows.
default:
    @just --list

# Format every workspace crate.
fmt:
    cargo fmt --all

# Check workspace formatting without changing files.
fmt-check:
    cargo fmt --all -- --check

# Run workspace tests or one backend's crate and application integration tests.
test backend="all" snapshots="check": (_run-workflow "test" backend snapshots)

# Accept workspace snapshots or one backend's crate and application snapshots.
snapshots backend="all": (test backend "update")

# Run strict Clippy for the workspace or one backend's crate and application integration test.
lint backend="all": (_run-workflow "lint" backend)

# Run formatting, strict Clippy, and tests for the workspace or one backend.
check backend="all": fmt-check (lint backend) (test backend "check")

[private]
[positional-arguments]
_run-workflow operation backend snapshots="":
    #!/usr/bin/env bash
    set -euo pipefail

    operation="$1"
    backend="$2"
    snapshots="$3"

    case "$backend" in
        all)
            backend_package=
            app_test=
            app_feature=
            ;;
        sqlite)
            backend_package=dbmd-backend-sqlite
            app_test=render
            app_feature=
            ;;
        postgres)
            backend_package=dbmd-backend-postgres
            app_test=postgres
            app_feature=postgres-tests
            ;;
        clickhouse)
            backend_package=dbmd-backend-clickhouse
            app_test=clickhouse
            app_feature=clickhouse-tests
            ;;
        mysql)
            backend_package=dbmd-backend-mysql
            app_test=mysql
            app_feature=mysql-tests
            ;;
        mariadb)
            backend_package=dbmd-backend-mariadb
            app_test=mariadb
            app_feature=mariadb-tests
            ;;
        duckdb)
            backend_package=dbmd-backend-duckdb
            app_test=duckdb
            app_feature=
            ;;
        *)
            printf 'unknown backend: %s\nexpected one of: all, sqlite, postgres, clickhouse, mysql, mariadb, duckdb\n' "$backend" >&2
            exit 2
            ;;
    esac

    case "$operation" in
        test)
            case "$snapshots" in
                check)
                    snapshot_update=no
                    ;;
                update)
                    snapshot_update=always
                    ;;
                *)
                    printf 'unknown snapshot mode: %s\nexpected one of: check, update\n' "$snapshots" >&2
                    exit 2
                    ;;
            esac

            export INSTA_UPDATE="$snapshot_update"

            if [[ "$backend" == all ]]; then
                cargo test --workspace --all-features
            else
                cargo test -p "$backend_package" --all-features

                if [[ -n "$app_feature" ]]; then
                    cargo test -p dbmd-app --features "$app_feature" --test "$app_test"
                else
                    cargo test -p dbmd-app --test "$app_test"
                fi
            fi
            ;;
        lint)
            if [[ "$backend" == all ]]; then
                cargo clippy --workspace --all-targets --all-features -- -D warnings
            else
                cargo clippy -p "$backend_package" --all-targets --all-features -- -D warnings

                if [[ -n "$app_feature" ]]; then
                    cargo clippy -p dbmd-app --features "$app_feature" --test "$app_test" -- -D warnings
                else
                    cargo clippy -p dbmd-app --test "$app_test" -- -D warnings
                fi
            fi
            ;;
        *)
            printf 'unknown workflow operation: %s\n' "$operation" >&2
            exit 2
            ;;
    esac
