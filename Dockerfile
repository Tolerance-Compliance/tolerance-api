FROM rust:alpine AS builder
RUN apk add --no-cache musl-dev
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release --bin tolerance-api

FROM alpine AS runner
RUN apk add --no-cache ca-certificates
WORKDIR /app
ENV PORT=3000

COPY --from=builder /app/target/release/tolerance-api ./
COPY cprt-sp_800_171_3_0_0-20260215-171034.json ./
COPY cprt-sp_800_172_1_0_0.json ./

RUN addgroup -S -g 1001 rustapp && adduser -S -u 1001 -G rustapp rustapp
USER rustapp

EXPOSE 3000
CMD ["./tolerance-api"]
