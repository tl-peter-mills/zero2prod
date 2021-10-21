FROM lukemathwalker/cargo-chef:latest-rust-1.53.0 as chef
WORKDIR /app

FROM chef as planner
COPY . .
# Compute a lock-like file for deps
RUN cargo chef prepare --recipe-path recipe.json

FROM chef as builder
COPY --from=planner /app/recipe.json recipe.json
# Build project deps
RUN cargo chef cook --release --recipe-path recipe.json

COPY . .
ENV SQLX_OFFLINE true
# Build project
RUN cargo build --release --bin zero2prod

# Runtime stage
FROM debian:bullseye-slim AS runtime
WORKDIR /app

# Install OpenSSL
RUN apt-get update -y \
    && apt-get install -y --no-install-recommends openssl \
    # Cleanup
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/zero2prod zero2prod
# Copy configuration files
COPY configuration configuration
ENV APP_ENVIRONMENT production
ENTRYPOINT ["./zero2prod"]