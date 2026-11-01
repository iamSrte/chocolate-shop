set dotenv-load

compose := "docker compose -f infra/compose.dev.yml"

default:
    @just --list

# Start local Postgres
db-up:
    {{compose}} up -d db

# Stop local services
db-down:
    {{compose}} down

# Drop, recreate and migrate the local database
[working-directory: 'backend']
db-reset:
    sqlx database reset -y --source api/migrations

# Refresh the committed .sqlx/ query cache. Run after changing any query! macro.
[working-directory: 'backend']
sqlx-prepare:
    cargo sqlx prepare --workspace -- --all-targets

# Run the API (reads .env)
[working-directory: 'backend']
dev-api:
    cargo run -p api

# Run the API, restarting on save
[working-directory: 'backend']
watch-api:
    bacon run-api

# Format all code
[working-directory: 'backend']
fmt:
    cargo fmt --all

# Run all tests. nextest skips doc tests, so they run separately.
[working-directory: 'backend']
test:
    cargo nextest run --workspace
    cargo test --workspace --doc

# Regenerate OpenAPI spec and TS client from the Rust API
openapi:
    cd backend && cargo run -q -p api --bin openapi > ../frontend/api-client/openapi.json
    cd frontend && bun run --filter @chocolate-shop/api-client generate

# Fail if the spec or TS client on disk is stale. Fix: `just openapi`.
# Generates into a temp dir, so it never touches the working tree.
openapi-check:
    #!/usr/bin/env bash
    set -euo pipefail
    tmp="$(mktemp -d)"
    trap 'rm -rf "$tmp"' EXIT
    (cd backend && cargo run -q -p api --bin openapi) > "$tmp/openapi.json"
    (cd frontend/api-client && bun x openapi-typescript "$tmp/openapi.json" -o "$tmp/schema.d.ts")
    diff -u frontend/api-client/openapi.json "$tmp/openapi.json"
    diff -u frontend/api-client/src/schema.d.ts "$tmp/schema.d.ts"

# Lint and typecheck; fails on warnings
lint:
    cd backend && cargo fmt --all -- --check
    cd backend && cargo clippy --workspace --all-targets -- -D warnings
    cd frontend && bun run --filter '*' typecheck

# Install the cargo tools the recipes use
tools:
    cargo install --locked cargo-nextest bacon
    cargo install --locked sqlx-cli --no-default-features --features rustls,postgres

# Everything CI runs
check: lint test openapi-check
