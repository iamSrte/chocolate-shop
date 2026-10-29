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

# Run the API (reads .env)
[working-directory: 'backend']
dev-api:
    cargo run -p api

# Regenerate OpenAPI spec and TS client from the Rust API
openapi:
    cd backend && cargo run -q -p api --bin openapi > ../frontend/api-client/openapi.json
    cd frontend && bun run --filter @chocolate-shop/api-client generate

# Format all code
[working-directory: 'backend']
fmt:
    cargo fmt --all

# Lint and typecheck; fails on warnings
lint:
    cd backend && cargo fmt --all -- --check
    cd backend && cargo clippy --workspace --all-targets -- -D warnings
    cd frontend && bun run --filter '*' typecheck

# Run all tests
[working-directory: 'backend']
test:
    cargo test --workspace

# Everything CI runs
check: lint test
