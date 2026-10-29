# Handmade Chocolate Shop

Online shop for handmade chocolates: box builder, preset boxes, recipe-based pricing.
Full plan: `docs/project-plan.md`.

## Layout

Monorepo split by ecosystem (ADR 0001). Each side owns its workspace configs; the root holds only shared files. `just` runs everything from the root.

- `backend/`: Cargo workspace (`Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`).
  - `domain/`: pure business rules (ADR 0002). No axum, sqlx or tokio. Modules follow glossary sections: `money`, `pieces`, `boxes`, `pricing`.
  - `api/`: Axum + SQLx + PostgreSQL. HTTP, DB, auth. Calls into `domain`.
- `frontend/`: Bun workspace (`package.json`, `bun.lock`, `tsconfig.base.json`).
  - `api-client/`: TS client generated from the API's OpenAPI spec (utoipa). Never hand-edit.
  - `web/`: TanStack Start (SSR/prerender). Not started; waits for design language.
- `infra/`: `compose.dev.yml` for local Postgres. VPS deploy files go here later.

Backend first. Frontend work starts only after the design language is ready.

## API conventions

- Code is grouped by feature: `api/src/<feature>/` (e.g. `catalog`, `pricing`, `cart`, `orders`, `auth`, `health`). Files, added only when needed:
  - `mod.rs`: builds the feature's `OpenApiRouter`.
  - `routes.rs`: public handlers. `admin.rs`: admin handlers.
  - `repo.rs`: SQL. `dto.rs`: request/response types (`ToSchema`).
- All routes live under `/api` on the site's domain, with no version prefix. Admin routes go under `/api/admin/...`, behind an admin-role guard.
- Errors: return `AppError` (`api/src/error.rs`). It renders as `{ "code", "message" }`; the frontend branches on `code`.
- SQL: use the `query!`/`query_as!` checked macros. Commit the `.sqlx/` cache (`cargo sqlx prepare --workspace`) so CI builds offline.
- Migrations: `api/migrations/`, embedded and run at API startup.
- Tests: domain logic gets plain unit tests. DB code is tested with `#[sqlx::test]` against real Postgres.

## Commands

- `just db-up`: local Postgres. Copy `.env.example` to `.env` first.
- `just dev-api`: run the API on `API_ADDR` (default `127.0.0.1:3000`). Needs the DB up, since migrations run at startup.
- `just openapi`: regenerate `frontend/api-client/openapi.json` + `src/schema.d.ts`. Run after any API change; CI fails on drift.
- `just check`: fmt check, clippy `-D warnings`, typecheck, tests. Same as CI.

`frontend/api-client` pins TypeScript 5: `openapi-typescript` uses the TS 5 compiler API, which TS 7 lacks.

## Rules

- Use `GLOSSARY.md` terms in code, API paths and docs.
- Money is integer tomans. Never floats.
- Pricing and box rules live in `backend/domain`, with unit tests.
- Server re-validates every box at checkout.

## Working with Claude

- Responses: use the `caveman` skill.
- Commit messages: use the `caveman-commit` skill.
- Planning, grilling (`grill-with-docs`, `grill-me`, etc.): ask questions with the built-in AskUserQuestion interface, not inline prose.
- CI: GitHub Actions (or self-hosted equivalent). Deploy target: VPS, not set up yet.

## Agent skills

### Issue tracker

Issues live as local markdown files under `.scratch/<feature>/`. See `docs/agents/issue-tracker.md`.

### Triage labels

Default five-role vocabulary, recorded as a `Status:` line per issue file. See `docs/agents/triage-labels.md`.

### Domain docs

Single-context: root `GLOSSARY.md` + `docs/adr/`. See `docs/agents/domain.md`.
