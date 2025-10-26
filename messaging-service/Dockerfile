FROM rust:1.89-alpine AS deps

WORKDIR /app
RUN apk add --no-cache pkgconfig openssl-dev build-base

COPY Cargo.toml Cargo.lock ./
COPY shared/Cargo.toml ./shared/Cargo.toml
COPY auth-service/Cargo.toml ./auth-service/Cargo.toml
COPY messaging-service/Cargo.toml ./messaging-service/Cargo.toml
COPY social-service/Cargo.toml ./social-service/Cargo.toml
COPY api-gateway/Cargo.toml ./api-gateway/Cargo.toml

RUN mkdir -p shared/src auth-service/src messaging-service/src social-service/src api-gateway/src && \
    echo "fn main() {}" > messaging-service/src/main.rs && \
    echo "pub fn lib() {}" > shared/src/lib.rs && \
    echo "fn main() {}" > auth-service/src/main.rs && \
    echo "fn main() {}" > social-service/src/main.rs && \
    echo "fn main() {}" > api-gateway/src/main.rs

WORKDIR /app/messaging-service
RUN cargo build --release

FROM rust:1.89-alpine AS builder

WORKDIR /app
RUN apk add --no-cache pkgconfig openssl-dev build-base

COPY --from=deps /app/target ./target
COPY --from=deps /usr/local/cargo /usr/local/cargo

COPY Cargo.toml Cargo.lock ./
COPY shared ./shared
COPY auth-service ./auth-service
COPY messaging-service ./messaging-service
COPY social-service ./social-service
COPY api-gateway ./api-gateway

WORKDIR /app/messaging-service
RUN touch src/main.rs && cargo build --release

FROM alpine:3.20 AS runner
RUN apk add --no-cache ca-certificates
RUN adduser -D -u 1000 appuser

WORKDIR /app

COPY --from=builder /app/target/release/messaging-service .

RUN chown -R appuser:appuser /app
USER appuser

EXPOSE 8082

CMD ["./messaging-service"]
