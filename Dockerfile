# Stage 1: Build frontend
FROM node:24-slim AS frontend-builder
RUN corepack enable && corepack prepare pnpm@latest --activate
WORKDIR /app/frontend
COPY frontend/package.json frontend/pnpm-lock.yaml ./
RUN pnpm install --frozen-lockfile
COPY frontend/ ./
RUN pnpm build

# Stage 2: Build backend (with dependency caching)
FROM rust:1-slim AS backend-builder
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
WORKDIR /app

# Copy manifests and create dummy source to cache dependency compilation
COPY Cargo.toml Cargo.lock ./
COPY crates/server/Cargo.toml crates/server/Cargo.toml
RUN mkdir -p crates/server/src && echo "fn main() {}" > crates/server/src/main.rs
ENV DATABASE_URL=sqlite://build.db
RUN cargo build --release -p balans-server

# Now copy real source and rebuild (only app code recompiles)
COPY crates/ crates/
RUN touch crates/server/src/main.rs && cargo build --release -p balans-server

# Stage 3: Runtime
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=backend-builder /app/target/release/balans-server ./
COPY --from=frontend-builder /app/frontend/dist ./static/
COPY crates/server/migrations/ ./migrations/

ENV STATIC_DIR=/app/static
ENV DATABASE_URL=sqlite:///app/data/balans.db
ENV PORT=3100

EXPOSE 3100
VOLUME ["/app/data"]

CMD ["./balans-server"]
