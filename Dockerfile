# Builder stage
FROM rust:1.89-slim-bullseye AS builder

WORKDIR /app
RUN apt update && apt install -y lld clang -y
COPY . .
ENV SQLX_OFFLINE=true
RUN cargo build --release

# Runtime stage
FROM rust:1.89-slim-bullseye AS runtime
COPY --from=builder /app/target/release/newsletter newsletter
COPY ./configuration ./configuration
ENV APP_ENVIRONMENT=production
ENTRYPOINT ["./newsletter"]