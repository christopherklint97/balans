# Repository Guidelines

## Project Structure & Module Organization

Balans is a Rust API plus React SPA for Swedish bookkeeping. Backend code lives in `crates/server/src/`, with route handlers in `routes/`, data models in `models/`, reporting in `report/`, K2 closing logic in `k2/`, SIE import/export in `sie/`, and SQL migrations in `crates/server/migrations/`. Frontend code lives in `frontend/src/`, with API helpers in `api/`, page routes in `routes/`, auth state in `auth/`, and shared UI components in `components/ui/`. Built frontend assets are emitted to `frontend/dist/`.

## Build, Test, and Development Commands

- `cp .env.example .env`: create local configuration; set `JWT_SECRET`.
- `cargo run -p balans-server`: run the API on `http://localhost:3100`.
- `JWT_SECRET=test-secret cargo test`: run Rust unit tests.
- `cd frontend && pnpm install`: install frontend dependencies.
- `pnpm --dir frontend dev`: run the Vite dev server on `http://localhost:5173`.
- `pnpm --dir frontend build`: typecheck and build the frontend.
- `pnpm --dir frontend lint`: run ESLint for frontend code.

## Coding Style & Naming Conventions

Use `cargo fmt` for Rust formatting and keep modules focused by domain. Rust functions and modules use `snake_case`; structs and enums use `PascalCase`. TypeScript uses React function components, `PascalCase` component names, and `camelCase` variables. Prefer existing shadcn/ui components and local API/query patterns before adding new abstractions.

## Testing Guidelines

Rust tests are standard `#[test]` unit tests colocated with modules. Add tests for accounting calculations, parsers, report formatting, and validation rules when behavior changes. Run `JWT_SECRET=test-secret cargo test` before pushing backend changes. For frontend changes, run `pnpm --dir frontend build`; run `pnpm --dir frontend lint` when touching UI or TypeScript structure.

## Commit & Pull Request Guidelines

Recent commits use concise, imperative messages such as `Add voucher correction flow` or `Make annual report PDF download work on mobile browsers`. Keep commits scoped to one feature or fix. Pull requests should include a short summary, test results, linked issue when applicable, and screenshots for visible UI changes.

## Security & Configuration Tips

Never commit real secrets or production database files. Required runtime configuration is documented in `README.md`; `JWT_SECRET` is mandatory. SQLite defaults to `sqlite://data/balans.db`, and migrations run automatically on server startup.
