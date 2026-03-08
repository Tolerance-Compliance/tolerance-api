FROM rust:alpine AS builder
RUN apk add --no-cache musl-dev
WORKDIR /app
COPY Cargo.toml ./
COPY src ./src
COPY assets ./assets
RUN cargo build --release --bin tolerance-api

FROM alpine AS runner
RUN apk add --no-cache ca-certificates
WORKDIR /app
ENV PORT=3000

COPY --from=builder /app/target/release/tolerance-api ./
COPY data/ ./data/

RUN addgroup -S -g 1001 rustapp && adduser -S -u 1001 -G rustapp rustapp
USER rustapp

EXPOSE 3000
CMD ["./tolerance-api"]
