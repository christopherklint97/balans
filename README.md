# Balans

Swedish accounting and bookkeeping application for small businesses. Supports double-entry bookkeeping, the BAS kontoplan, SIE export/import, K2 compliance, asset management, and annual report filing.

## Tech Stack

**Backend:** Rust, Axum, SQLite (SQLx), JWT authentication
**Frontend:** React 19, TypeScript, TanStack Router & Query, Tailwind CSS, shadcn/ui

## Getting Started

### Prerequisites

- [Rust](https://rustup.rs/) (stable)
- [Node.js](https://nodejs.org/) 20+
- [pnpm](https://pnpm.io/)

### Setup

1. Copy the environment file and set a JWT secret:

```sh
cp .env.example .env
# Edit .env and set a strong JWT_SECRET
```

2. Start the backend:

```sh
cargo run -p server
```

The server runs on `http://localhost:3100` by default. The SQLite database is created automatically at `data/balans.db`.

3. Start the frontend:

```sh
cd frontend
pnpm install
pnpm dev
```

The frontend runs on `http://localhost:5173` and proxies API requests to the backend.

## Project Structure

```
crates/server/          Rust API server
  migrations/           SQL migrations (applied automatically)
  src/
    auth/               JWT authentication & middleware
    db/                 Database pool, migrations, seeding
    routes/             API endpoint handlers
    models/             Data models
    assets/             Fixed asset management
    filing/             Annual report generation
    k2/                 K2 compliance checks
    report/             Financial reporting (balance sheet, income statement)
    sie/                SIE format import/export
    tax/                Tax calculations

frontend/               React SPA
  src/
    api/                API client, queries, types
    auth/               Auth context & helpers
    routes/             Page components
    components/ui/      shadcn/ui components
```

## Environment Variables

| Variable       | Description                         | Default                    |
| -------------- | ----------------------------------- | -------------------------- |
| `DATABASE_URL` | SQLite connection string            | `sqlite://data/balans.db`  |
| `JWT_SECRET`   | Secret key for signing JWT tokens   | **required**               |
| `PORT`         | Server port                         | `3100`                     |

## License

[MIT](LICENSE)
