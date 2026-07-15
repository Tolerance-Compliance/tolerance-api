# Migration plan: Fly.io → Cloudflare Containers (FedRAMP jurisdiction)

> Status: **planned, not started**. Tracked in Linear (tolerance-api container
> migration). This document is the agreed design; nothing here is deployed.

## What this service is (facts that shape the plan)

Stateless, public, read-only Rust/Axum service: REST + MCP + Swagger over the
bundled NIST/FAR corpus (32 MB JSON, 11 documents, parsed and indexed into
memory at boot, ~100–120 MB RSS). No database, no volumes, no secrets, no auth,
CORS fully open — all deliberate: it serves public standards text, and `/mcp` +
TOON content negotiation are a public LLM-facing product surface. Consumers:
the dashboard's ask/chat routes and the 800-171A objective backbone (TOL-715),
plus external MCP/REST users, all via `https://api.tolerance.app` (today a
Cloudflare-proxied CNAME to Fly, region `iad`, v0.1.4x).

Why Containers and not a Worker: tokio-full/hyper/mio serving plus `std::fs`
boot loads do not run on wasm32. The existing Alpine/musl image lifts and
shifts unchanged.

## Target architecture

House container pattern (mirrors `tolerance-auth-container`, minus FIPS — this
is public data, no FIPS mandate):

- Thin Rust/WASM shim worker → `jurisdiction("fedramp")` Durable Object
  singleton (`"api"`) → container on port 3000.
- `[containers.constraints] jurisdiction = "fedramp"` on dev AND production.
  Posture-consistency pinning (public data, so this is the boundary story, not
  data protection).
- **Edge caching in the shim** — the differentiator: the corpus is immutable
  per release, so GETs cache aggressively with version-scoped cache keys
  (deploy = clean bust) and `Vary: Accept` (JSON vs TOON must never
  cross-contaminate). `POST /mcp` and `POST .../poam/validate` are never
  cached. This neutralizes the DO singleton's single-location latency.
- Keep-warm cron pinging `/health` (auth-container precedent; boot parses
  32 MB). Instance type `standard-1`.
- Production shim gets the `api.tolerance.app` custom domain — the fleet's
  first public-route container. Add a WAF rate-limit rule (stays auth-less).

## Repo changes riding along

- Dockerfile: `COPY Cargo.lock` + `cargo build --locked` (today's image builds
  are UNPINNED — supply-chain fix).
- New `worker/` shim crate (auth-container layout); `wrangler.toml` with dev +
  `[env.production]`.
- Account cleanup first: DELETE the stale `tolerance-api` WASM worker on the
  Cloudflare account (a dead 2026-05-31 experiment; nothing routes to it) so
  the name is free for dev. Production worker = `tolerance-api-production`.
- CI: replace `.github/workflows/deploy.yml` (Fly, homegrown toml-cli version
  bump) with the house container `ci-cd.yml`: fmt / clippy / check / test /
  deploy dry-run / cargo-deny / gitleaks → release-please (**manifest seeded at
  the current version** so numbering continues) → environment-gated
  `wrangler deploy --env production`. Repo secrets: `RELEASE_PLEASE_TOKEN`,
  `CLOUDFLARE_API_TOKEN` (Workers + Containers scopes), `CLOUDFLARE_ACCOUNT_ID`.
  `FLY_API_TOKEN` is deleted after decommission.
- Tidy the stale IronShield-Tech links in README.md.

## Cutover (zero-downtime, instant rollback)

1. Scaffold + dev deploy → **parity harness**: scripted response diff of every
   GET endpoint (both `Accept: application/json` and `text/toon`), OpenAPI spec
   equality, MCP tool-call + POA&M-validate smoke — Cloudflare dev vs live Fly.
2. Ship through CI to `tolerance-api-production` (no public route yet);
   validate on a temporary hostname (`api-cf.tolerance.app`).
3. Cutover: attach the `api.tolerance.app` custom domain to the production
   worker (zone already on Cloudflare — atomic origin swap from the Fly CNAME).
   Rollback = detach the domain; Fly keeps running untouched.
4. Bake ~48 h (tail + the dashboard's ask/chat/objective routes as canaries) →
   Fly `min_machines_running=0` → after a quiet week: `fly apps destroy`,
   remove `FLY_API_TOKEN`, delete `fly.toml` + `deploy.yml` + this section's
   Fly references.

## Open decisions

- DO singleton lands in ENAM (matches Fly `iad`); revisit multi-instance only
  if cache-miss latency ever matters.
- Stays public/no-auth (product surface) with the WAF rate limit as guardrail.
- Post-migration nicety (separate ticket): dashboard callers switch from
  public fetch to a service binding.
