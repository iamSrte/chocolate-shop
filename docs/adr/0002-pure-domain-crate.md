# Business rules live in a pure `domain` crate

Pricing and box rules live in `backend/domain`, a crate with no IO dependencies (no axum, sqlx or tokio). The `api` crate loads data, calls into `domain`, and stores the results. Prices must be exact and box validation must agree between client and server, so a separate crate lets the compiler enforce that this code stays pure and unit-testable. It also leaves the door open to compiling the box rules to WASM for the box builder, instead of rewriting them in TypeScript.

## Considered Options

- **Modules inside the `api` crate**: simpler, but purity would be kept by convention only.
- **One crate per area** (`pricing`, `boxes`, ...): stronger boundaries, but too much manifest overhead for the current size.
