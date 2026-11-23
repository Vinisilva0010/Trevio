# 0002. Private keys exist only on the user's device

- Status: Accepted
- Date: 2026-11-22

## Context

Trevio is a self-custodial wallet. Holding keys or user funds would make
Trevio a custodian, with the legal, operational and security burden that
implies, and would make the backend a single target for theft.

## Decision

Keys are generated on the phone and kept only in the device secure storage.
They are never sent to the backend, logged, backed up by Trevio, or included in
analytics or crash reports. Transactions are signed on the device. The backend
sees only public addresses and signed transactions or their results.

## Consequences

- A lost device without the user's own backup means lost funds; the app must
  make backup and recovery clear. Trevio cannot recover keys.
- Backend compromise cannot directly move user funds.
- Code in `app/src/wallet/` and `app/src/auth/` is P0/P1 and always reviewed
  by the CTO.
- No backend feature may require receiving key material.
