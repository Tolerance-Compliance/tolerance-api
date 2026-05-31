FROM rust:alpine AS builder
RUN apk add --no-cache musl-dev
WORKDIR /app
# Workspace manifest + the two native members (core + native). The `data/`
# directory is needed at build time too: core embeds data/cmmc-scoring.json via
# include_str!.
COPY Cargo.toml ./
COPY core ./core
COPY native ./native
COPY data ./data
RUN cargo build --release -p tolerance-api --bin tolerance-api

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
