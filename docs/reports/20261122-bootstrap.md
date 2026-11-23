# Bootstrap report, 2026-11-22 (Phase 0, technical part)

## 1. Environment (read-only check)

| Tool | Result |
| --- | --- |
| git | 2.43.0 |
| rustc | 1.100.0 (0ef1bc386 2026-11-09) |
| cargo | 1.100.0 |
| rustfmt / clippy | 1.11.0-stable / 0.1.100 |
| docker | 29.1.3 |
| docker compose | 2.40.3 |
| psql | not installed |
| node / npm | not installed |
| cargo-deny | not installed |

## 2. What was done

- Root files: README, .gitignore, .gitattributes, .editorconfig, SECURITY,
  CONTRIBUTING, CODEOWNERS, PR template, dependabot. No LICENSE (TD-001).
- Rust workspace (edition 2024, toolchain pinned to installed 1.100.0,
  `unsafe_code = forbid`, clippy lints incl. `unwrap_used`) with `api/`
  (`trevio-api`):
  - typed config (`DATABASE_URL`, `BIND_ADDR`, `LOG_LEVEL`), fail closed,
    secrets redacted;
  - JSON logging, request-id middleware (validated, echoed), no bodies or
    query strings logged;
  - `GET /health`, `GET /ready` (SELECT 1, 2 s timeout, 503 problem+json);
  - one `ApiError` -> RFC 9457 problem+json, fixed strings only;
  - graceful shutdown (SIGINT/SIGTERM), 10 s request timeout, 64 KiB body limit;
  - `api/migrations/README.md` only.
- `contracts/openapi.yaml` (3.1), `contracts/README.md`, `contracts/redocly.yaml`;
  tests that fail on any route served but undocumented (and the reverse),
  dangling `$ref`, or an operation requiring auth.
- `infra/docker-compose.yml`, `infra/README.md`, `.env.example`.
- `.github/workflows/ci.yml` and `deny.toml`.
- Docs: CONTEXT, TECH_DEBT, ADRs 0001-0003 (0003 Proposed), two STRICT design
  drafts (HOLD), FRONTEND_INTEGRATION, `app/README.md`.
- No STRICT code written: no money logic, state machine, auth, tables or
  partner integration.

## 3. Commands and results

| Command | Result |
| --- | --- |
| `cargo fmt --check` | PASS |
| `cargo clippy --all-targets --locked -- -D warnings` | PASS |
| `cargo test --locked` | PASS: 8 unit (config), 8 http, 6 contract; 1 ignored (database) |
| `docker compose ... config` without credentials | fails with "required variable ... missing" (intended) |
| Real-DB test, throwaway random password exported only in that command | PASS: `test ready_is_200_with_a_real_database ... ok` (healthcheck reached `healthy`) |
| Cleanup: `docker compose down -v` | PASS: container, network and volume removed |
| Binary with no `DATABASE_URL` / bad `BIND_ADDR` | exit code 2 with a clear message, no values echoed |
| `cargo deny check` | NOT RUN (tool not installed) |
| OpenAPI lint (Redocly) / Prism / openapi-typescript | NOT RUN (Node not installed) |
| CI workflow | NOT RUN (needs GitHub) |

The `postgres:18` image stays in the local Docker cache.

Action SHAs, verified against the official repositories with `git ls-remote`
(peeled commit of each tag): checkout v7.0.1 `3d3c42e5...`, rust-cache v2.9.3
`d1527c8f...`, cargo-deny-action v2.1.3 `17aeb3c8...`, setup-node v7.1.0
`949feb24...`. All pinned by full SHA, so no tag-only TECH_DEBT item.

Dependency versions checked with `cargo search`/`cargo info` (axum 0.8.9,
sqlx 0.9.1, tokio 1.54.1, tower-http 0.7.1, serde 1.0.229, tracing 0.1.44,
tracing-subscriber 0.3.23, thiserror 2.0.21, uuid 1.29.3; dev: tower 0.5.3,
http-body-util 0.1.5, serde_norway 0.9.42). npm tools checked via the registry:
@redocly/cli 2.82.0, @stoplight/prism-cli 5.16.0, openapi-typescript 7.13.0.

## 4. Assumptions

1. Rust 1.100 is the "stable already installed"; `rust-version = "1.100"`.
2. Pool is lazy so the server starts and `/health` answers with Postgres down;
   `/ready` reports the truth. Pool max 5 connections.
3. `RUST_LOG` is ignored; level comes only from `LOG_LEVEL`.
4. Timeouts/limits (10 s, 64 KiB, 2 s ready) are constants, not env vars.
5. Problem `type` uses `urn:trevio:problem:*` because no domain is decided.
6. SECURITY.md uses GitHub private vulnerability reporting (no e-mail
   published); it must be enabled (TD-003).
7. `.gitignore` ignores `android/` and `ios/` (Expo-generated); revisit if the
   app ejects to committed native code.
8. Redocly "minimal" ruleset for the OpenAPI CI job.
9. Design drafts cover only the Pix -> stablecoin (buy) flow.
10. Amounts above 2^53-1 travel as decimal strings (money draft).
11. `Cargo.lock` is committed (binary crate).

## 5. Skipped and why

- cargo-deny, OpenAPI lint, mock server and client generation: tools missing.
- GitHub-side settings (branch protection, vulnerability reporting): not
  possible from the repo (TD-002, TD-003).
- LICENSE, Dockerfile, app code, any STRICT implementation: by instruction.

## 6. Tools the human must install

- **Node.js LTS + npm** (needed by the designer and to validate the contract
  locally): https://nodejs.org/en/download (or your OS package manager / nvm
  https://github.com/nvm-sh/nvm). Check: `node --version`.
- **cargo-deny** (run the dependency policy locally):
  `cargo install cargo-deny --locked` (https://github.com/EmbarkStudios/cargo-deny)
- **psql** (optional, for inspecting the DB): `sudo apt install postgresql-client`
  (https://www.postgresql.org/download/linux/ubuntu/)
- **sqlx-cli** (only when the first migration exists):
  `cargo install sqlx-cli --no-default-features --features rustls,postgres`
  (https://github.com/launchbadge/sqlx/tree/main/sqlx-cli)

## 7. Open questions

- Who owns the code (license) and under which entity?
- Designer's GitHub handle for CODEOWNERS?
- Which partner (PSAV), and what are its API, webhooks, quote validity and
  refund mechanics?
- Does the partner or the user sign the on-chain transfer?
- Is the reverse flow (pay a Pix with stablecoin) in the first release?
- Where will the API be deployed, and who terminates TLS?

## 8. Commit plan (run in order, one at a time, from the repo root)

1. Repository hygiene
   ```bash
   git add .gitignore .gitattributes .editorconfig
   git commit -m "chore: add gitignore, gitattributes and editorconfig"
   ```
2. Project policies and GitHub config
   ```bash
   git add README.md SECURITY.md CONTRIBUTING.md .github/CODEOWNERS .github/pull_request_template.md .github/dependabot.yml
   git commit -m "docs: add readme, security policy, contributing guide and github templates"
   ```
3. Rust workspace and toolchain
   ```bash
   git add Cargo.toml Cargo.lock rust-toolchain.toml clippy.toml deny.toml api/Cargo.toml
   git commit -m "chore(api): add rust workspace, pinned toolchain and lint policy"
   ```
4. Server skeleton
   ```bash
   git add api/src api/migrations/README.md
   git commit -m "feat(api): add config, logging, health and ready endpoints"
   ```
5. Contract and its tests
   ```bash
   git add contracts api/tests/openapi.rs
   git commit -m "feat(contracts): add openapi spec for health and ready with route coverage test"
   ```
6. Server tests
   ```bash
   git add api/tests/http.rs api/tests/database.rs
   git commit -m "test(api): cover health, ready, errors, request id and body limit"
   ```
7. Local infrastructure
   ```bash
   git add infra .env.example
   git commit -m "chore(infra): add local postgres compose setup"
   ```
8. CI
   ```bash
   git add .github/workflows/ci.yml
   git commit -m "ci: add fmt, clippy, tests, cargo-deny and openapi validation"
   ```
9. Context, ADRs and tech debt
   ```bash
   git add docs/CONTEXT.md docs/TECH_DEBT.md docs/decisions
   git commit -m "docs: add project context, tech debt and architecture decisions"
   ```
10. STRICT design drafts
    ```bash
    git add docs/design
    git commit -m "docs(design): add draft money and order state machine designs on hold"
    ```
11. Frontend integration
    ```bash
    git add app/README.md docs/FRONTEND_INTEGRATION.md
    git commit -m "docs(app): add frontend structure and integration guide"
    ```
12. Report
    ```bash
    git add docs/reports/20261122-bootstrap.md
    git commit -m "docs: add bootstrap report"
    ```

Note: the repository is on `feature/20261122-bootstrap`. The workflow expects a
`dev` branch; create it and open the PR against it yourself.
