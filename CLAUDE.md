# Balans

Swedish accounting application — Rust backend + React frontend.

## Build & Run

```sh
# Backend
cargo run -p balans-server    # starts on :3100
cargo test -p balans-server   # run tests (set JWT_SECRET env var first)
cargo clippy -p balans-server # lint

# Frontend
cd frontend
pnpm install
pnpm dev                     # starts on :5173
pnpm build                   # production build
pnpm lint                    # eslint
```

## Environment Variables

| Variable | Required | Default | Description |
|---|---|---|---|
| `JWT_SECRET` | Yes | — | Secret key for JWT token signing |
| `DATABASE_URL` | No | `sqlite://data/balans.db` | SQLite database path |
| `APP_MODE` | No | `saas` | `saas` (multi-tenant) or `fixed` (single-tenant) |
| `FIXED_COMPANY_ID` | No | — | Required when `APP_MODE=fixed` — the company users are assigned to |
| `PORT` | No | `3100` | Server port |
| `STATIC_DIR` | No | `frontend/dist` | Path to built frontend assets |

## Architecture

- **Workspace crate:** `crates/server` (the only crate for now)
- **Database:** SQLite via SQLx with compile-time checked queries. Migrations in `crates/server/migrations/` are applied automatically on startup.
- **Auth:** JWT tokens — `JWT_SECRET` env var is required. Middleware in `src/auth/middleware.rs` protects all routes under `/api` except login/register.
- **User approval:** New users register with `status = 'pending'`. First user is auto-approved as admin. Admins approve/reject subsequent users via `/api/admin/users/{id}/approve|reject`.
- **App state:** `AppState { pool, config }` in `src/config.rs` — all route handlers receive `State<AppState>` and access the pool via `state.pool`.
- **Company access:** `src/access.rs` provides `verify_company_access()` and `verify_fiscal_year_access()` helpers. Role hierarchy: owner > admin > member > viewer. System admins bypass company access checks.
- **App modes:** SaaS mode (default) — users create companies and are auto-added as owner. Fixed mode — users are assigned to a pre-configured company on registration.
- **Monetary values:** Stored as integer cents (i64). The `money` module handles formatting and conversion.
- **Frontend routing:** TanStack Router with file-based route generation (`routeTree.gen.ts` is auto-generated, do not edit manually).
- **API client:** `frontend/src/api/client.ts` handles auth headers and base URL. Queries/mutations in `queries.ts`.

## Conventions

- All monetary amounts are in Swedish kronor (SEK), stored as cents (ören).
- Account numbers follow the Swedish BAS kontoplan standard.
- API routes are nested under `/api`. Public auth routes: `/api/auth/login`, `/api/auth/register`. Everything else requires a Bearer token.
- Admin routes are under `/api/admin/` and require system admin role.
- Backend errors use the `AppError` type in `src/error.rs` which maps to appropriate HTTP status codes (including 403 Forbidden for access control).
- Frontend uses shadcn/ui components in `src/components/ui/`.
- UI text is in Swedish.
