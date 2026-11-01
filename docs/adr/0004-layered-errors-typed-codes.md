# Layered errors with one central mapping to typed codes

Each `domain` module owns a `thiserror` enum (`boxes::BoxError`, `pricing::PricingError`, ...) with no HTTP knowledge, keeping ADR 0002's purity. The `api` crate wraps each one in `AppError` via `#[from]`, and one `match` in `api/src/error.rs` maps every variant to an HTTP status and an `ErrorCode`. `ErrorCode` is a single snake_case enum exposed in the OpenAPI spec, so the TS client gets a string union and the frontend branches on checked values. A new domain variant without a mapping fails to compile, and every status decision sits in one file.

Codes are flat snake_case named with `GLOSSARY.md` terms (`box_not_full`, `piece_not_allowed_in_mold`). Once shipped, a code is never renamed or reused: only added or deprecated, since the frontend depends on it.

## Considered Options

- **One crate-wide `DomainError`**: fewer types, but one enum that grows with every module.
- **Per-feature API error enums implementing `IntoResponse`**: no central match, but status and code choices spread across features and drift.
- **`code` as a free `&'static str`**: simpler, but the frontend sees `string` and typos go unnoticed.

## Consequences

- Unexpected failures go to `Database(sqlx::Error)` or `Internal(anyhow::Error)` and render as `internal` with no details. Constraint violations are never auto-mapped: a repo checks the constraint name and returns a specific error.
- Every non-2xx response, including extractor rejections, unknown routes, 405s, timeouts and panics, uses the same `{ code, message }` body.
