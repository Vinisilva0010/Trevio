# Trevio context

Read this first. It is the short version of where the project stands.

## People and roles

| Who | Role |
| --- | --- |
| Vinicius | CTO and backend. Owns architecture, security reviews, `wallet/`, `auth/` and `api/` in the app |
| Designer | Designs the app in Figma, then writes the frontend code and plugs it into `app/` |
| Frontend / marketing helpers | Support on UI and go-to-market; no access to key handling code |
| Investor (with partners) | Funds the project |

## What we are building

A self-custodial mobile wallet for Brazil. Users receive and pay with Pix while
holding USDC/USDT on Solana. A licensed partner (PSAV) performs KYC and the
BRL <-> stablecoin conversion. Trevio never holds user keys or user funds.

## Architecture summary

```text
 Phone (Expo / React Native)          Trevio backend (Rust)         Licensed partner (PSAV)
 - generates and keeps keys    <----> - Axum API, PostgreSQL  <----> - KYC
 - signs transactions locally         - order tracking, no keys     - Pix + BRL <-> stablecoin
        |                             - no user funds                 conversion
        v
     Solana (USDC / USDT)
```

- Backend: Rust, Axum 0.8, SQLx 0.9, PostgreSQL 18.
- Mobile app: Expo / React Native, built from Figma designs. Web is read-only.
- Contract-first: `contracts/openapi.yaml` is the source of truth.
- Money: integer minor units only. BRL in centavos; USDC/USDT in 6-decimal base units.

## Decisions so far

- [ADR 0001](decisions/0001-monorepo.md): one monorepo.
- [ADR 0002](decisions/0002-keys-only-on-device.md): keys exist only on the device.
- [ADR 0003](decisions/0003-licensed-partner-does-kyc-and-conversion.md): partner does KYC and conversion (Proposed).
- The database is never migrated at startup.
- STRICT work needs an approved design and independent review before code.

## Current state (Phase 0)

Done: repository skeleton, Rust workspace with `/health` and `/ready`,
OpenAPI contract for those two routes, local Postgres, CI definition, docs.

Not started, by design (STRICT, waiting for review): money types, order state
machine, authentication, database schema, partner integration. Drafts:

- [Money and amounts](design/20261122-money-and-amounts/design.md) (HOLD)
- [Order state machine](design/20261122-order-state-machine/design.md) (HOLD)

See [TECH_DEBT.md](TECH_DEBT.md) for open items.
