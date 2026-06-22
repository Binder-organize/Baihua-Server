syntax=docker/dockerfile:1

# Stage 1: Build
FROM rust:1.85-slim-bookworm AS build
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src/ src/
COPY migrations/ migrations/
RUN cargo build --release --locked

# Stage 2: Runtime
FROM debian:bookworm-slim
WORKDIR /app
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates curl && rm -rf /var/lib/apt/lists/*
COPY --from=build /app/target/release/baihua-server .
COPY --from=build /app/migrations/ migrations/
EXPOSE 2424
ENV BAIHUA_ENV=production
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD curl -f http://localhost:2424/health || exit 1
CMD ["./baihua-server"]
