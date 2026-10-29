# Monorepo split by ecosystem, with a generated TS client

Backend (Rust) and frontend (TypeScript) live in one repo so an API change and the regenerated TS client land in the same commit. The OpenAPI spec comes from the Rust code (utoipa), and the client is generated from it. CI fails when the committed client is stale. The repo is split into `backend/` (Cargo workspace) and `frontend/` (Bun workspace), and each side keeps its own configs and lockfile. This keeps the root small, makes CI path filters a single folder per side, and lets the API image build from `backend/` alone.

## Considered Options

- **Two repos**: the client would have to be published as a package or copied between repos, and the two would drift.
- **`apps/` + `packages/` with both workspace configs at the root**: the usual JS-monorepo shape. Rejected because Rust and JS configs pile up together at the root, and a Rust library has no natural home.
