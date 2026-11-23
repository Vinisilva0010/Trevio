# Trevio

Trevio is a self-custodial mobile wallet for Brazil. Users receive and pay with
Pix while holding USDC or USDT on Solana.

- **Self-custodial.** Keys are generated and stored only on the user's phone.
  Trevio never holds user keys or user funds.
- **Licensed partner.** A licensed partner (PSAV) performs KYC and the
  BRL <-> stablecoin conversion.
- **Money is integers.** Amounts are always integer minor units, never floats.

Status: Phase 0 (bootstrap). Only the server skeleton exists; money, auth and
database features are in design review (see `docs/design/`).

## Layout

| Path | What |
| --- | --- |
| `api/` | Rust backend (Axum, SQLx, PostgreSQL). Binary: `trevio-api` |
| `api/migrations/` | Database migrations (none yet; applied manually) |
| `app/` | Mobile app (Expo / React Native); to be added, see `app/README.md` |
| `contracts/` | `openapi.yaml`, the source of truth for the HTTP API |
| `infra/` | Local development infrastructure (Docker Compose) |
| `docs/` | Context, decisions (ADRs), design drafts, tech debt, reports |

## Run locally

Requirements: Rust (version pinned in `rust-toolchain.toml`), Docker with
Compose.

```bash
cp .env.example .env     # edit the placeholder password; .env is git-ignored
docker compose --env-file .env -f infra/docker-compose.yml up -d

set -a; source .env; set +a
cargo run -p trevio-api

curl -i http://127.0.0.1:8080/health
curl -i http://127.0.0.1:8080/ready
```

Checks that must pass before a PR:

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

The server refuses to start if `DATABASE_URL` is missing or invalid, and it
never runs migrations by itself.

## More

- [CONTRIBUTING.md](CONTRIBUTING.md): workflow and rules
- [SECURITY.md](SECURITY.md): reporting vulnerabilities
- [docs/CONTEXT.md](docs/CONTEXT.md): people, architecture, decisions
- [docs/FRONTEND_INTEGRATION.md](docs/FRONTEND_INTEGRATION.md): app developers start here
