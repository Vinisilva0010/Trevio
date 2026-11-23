# 0001. One monorepo for backend, app, contracts and infra

- Status: Accepted
- Date: 2026-11-22

## Context

Trevio has a Rust backend, an Expo mobile app, an API contract and local
infrastructure. The team is very small. The app and the backend must agree on
the HTTP contract, and contract changes are STRICT work that is easier to
review when both sides are visible in one change.

## Decision

Keep everything in one repository: `api/`, `app/`, `contracts/`, `infra/`,
`docs/`. Ownership inside it is enforced with `CODEOWNERS`; the sensitive
folders of the app (`wallet/`, `auth/`, `api/`) always need the CTO's review.

## Consequences

- A contract change, its server implementation and its client update can be
  reviewed together.
- CI must stay fast and path-aware as the repo grows (see TD-008).
- Access control is per path via CODEOWNERS, not per repository. Anyone with
  write access can read everything; secrets must therefore never be committed.
- Splitting later is possible because the folders have clear boundaries.
