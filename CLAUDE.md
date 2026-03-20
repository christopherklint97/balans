# Balans

Swedish accounting application — Rust backend + React frontend.

## Build & Run

```sh
# Backend
cargo run -p server          # starts on :3100
cargo test -p server         # run tests (set JWT_SECRET env var first)
cargo clippy -p server       # lint

# Frontend
cd frontend
pnpm install
pnpm dev                     # starts on :5173
pnpm build                   # production build
pnpm lint                    # eslint
```

## Architecture

- **Workspace crate:** `crates/server` (the only crate for now)
- **Database:** SQLite via SQLx with compile-time checked queries. Migrations in `crates/server/migrations/` are applied automatically on startup.
- **Auth:** JWT tokens — `JWT_SECRET` env var is required. Middleware in `src/auth/middleware.rs` protects all routes under `/api` except login/register.
- **Monetary values:** Stored as integer cents (i64). The `money` module handles formatting and conversion.
- **Frontend routing:** TanStack Router with file-based route generation (`routeTree.gen.ts` is auto-generated, do not edit manually).
- **API client:** `frontend/src/api/client.ts` handles auth headers and base URL. Queries/mutations in `queries.ts`.

## Conventions

- All monetary amounts are in Swedish kronor (SEK), stored as cents (ören).
- Account numbers follow the Swedish BAS kontoplan standard.
- API routes are nested under `/api`. Public auth routes: `/api/auth/login`, `/api/auth/register`. Everything else requires a Bearer token.
- Backend errors use the `AppError` type in `src/error.rs` which maps to appropriate HTTP status codes.
- Frontend uses shadcn/ui components in `src/components/ui/`.
