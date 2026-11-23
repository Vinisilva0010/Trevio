# Design: order state machine

- Class: STRICT. Security level: P0
- Status: **HOLD** (awaiting human confirmation and independent review)
- Date: 2026-11-22
- No code, table or partner call may be written from this draft until the
  status is "Approved". Depends on the money design.

## Goal

A single, explicit lifecycle for an order that turns a Pix payment into
stablecoin in the user's own wallet, where every transition is allowed or
rejected by one function, and every unclear situation ends in a safe state.

Out of scope: the money types (separate design), the partner API shape, the
login system, the reverse flow (paying a Pix with stablecoin, which needs its
own states and its own design), notifications, UI.

## Sources and assumptions that may be wrong

Sources: product description; ADR 0002 (keys on device); ADR 0003 (partner,
still Proposed).

1. The partner does not exist yet in a contract. Everything about its events
   (Pix received, conversion done, refund) is assumed and may be wrong.
2. The partner holds the fiat leg and performs refunds; Trevio only records
   and triggers. If Trevio ever has to move fiat itself, this design is wrong.
3. Trevio does not sign the on-chain transfer; the partner (or a partner
   wallet) sends stablecoin to the user's address. If the user must sign,
   states change.
4. Events from the partner can arrive late, duplicated or out of order.
5. 24 hours is the hold period for risk reasons (`Held24h`); the real rule
   comes from the partner or compliance.
6. "Confirmed on chain" means finality as defined by Solana commitment level
   `finalized`; the level must be confirmed.

## States

Main path: `Created` -> `AwaitingPix` -> `PixReceived` -> `Converting` ->
`Sent` -> `ConfirmedOnChain`.

Exits: `Expired`, `UnderReview`, `Held24h`, `Rejected`, `Refunded`.

| From | To | Trigger |
| --- | --- | --- |
| Created | AwaitingPix | Partner returns a Pix charge for the quote |
| Created | Expired | Quote or order expires before a charge exists |
| AwaitingPix | PixReceived | Partner confirms the exact expected amount |
| AwaitingPix | Expired | Pix not paid before the charge expires |
| AwaitingPix | UnderReview | Amount differs, payer mismatch, or duplicate payment |
| PixReceived | Converting | Checks passed, conversion started |
| PixReceived | Held24h | Risk rule requires a hold |
| PixReceived | UnderReview | Any check is inconclusive |
| Held24h | Converting | Hold elapsed and checks pass |
| Held24h | Rejected | Checks fail |
| Converting | Sent | Partner reports stablecoin transfer submitted |
| Converting | UnderReview | Timeout or unknown partner result |
| Sent | ConfirmedOnChain | Transaction finalized for the right amount and address |
| Sent | UnderReview | Not finalized in time, or amount/address mismatch |
| UnderReview | Converting | Human approves |
| UnderReview | Rejected | Human rejects |
| Rejected | Refunded | Money was received and the partner refunds it |

Terminal: `ConfirmedOnChain`, `Expired`, `Refunded`, and `Rejected` when no
money was received. `UnderReview` is never terminal: it needs a human decision.

## Smallest change that solves it

- One `OrderState` enum and one pure function
  `transition(state, event) -> Result<state, TransitionError>` with an
  exhaustive table. Everything not in the table is an error.
- An append-only event log per order (who/what caused each transition, when,
  external reference). The state is derived from, and consistent with, the log.
- Idempotency: every external event carries a unique id; processing the same
  id twice has no extra effect.
- Optimistic concurrency: a transition applies only if the order is still in
  the state the caller read.
- Amounts use the money design types; the order stores the accepted quote.

## Tests that prove it

Must pass:

- Every row of the table above, one test each.
- Full happy path to `ConfirmedOnChain`.
- Duplicate event is a no-op; replaying the full log yields the same state.
- Two concurrent transitions: exactly one wins.

Must fail:

- Every pair not in the table (generated exhaustively), for example
  `Created -> Sent`, `ConfirmedOnChain -> anything`, `Refunded -> Converting`.
- Event for an unknown order; event with an amount that differs from the quote.
- Out-of-order event (`Sent` before `Converting`).
- `Converting -> Sent` without the partner reference.
- Transition attempted by a caller without permission.

## Impact

- API: order endpoints and the state list in `contracts/openapi.yaml`; the app
  must handle every state (see `docs/FRONTEND_INTEGRATION.md`). Contract
  change, so STRICT.
- Database: `orders` and `order_events` tables, unique constraint on external
  event ids, check constraint on valid state names. Not created by this draft.
- Permissions: users read only their own orders; only the backend (and a
  reviewer role for `UnderReview`) may transition. Needs the auth design.

## Failure rules

- Timeout or unknown result from the partner never advances the order: it
  moves to `UnderReview`, or stays where it is and is retried idempotently.
- Never mark `Sent` or `ConfirmedOnChain` without a verifiable external
  reference.
- Never retry a money-moving call without an idempotency key.
- Webhooks are accepted only if authenticated by the partner scheme (to be
  designed); unauthenticated events are dropped and logged without body.
- An unrecognized state value read from storage is an error, not a default.
- When in doubt, hold: `UnderReview` is always safer than advancing.

## Implementation order (after approval)

1. Money design implemented and reviewed.
2. Pure state machine with the table and exhaustive tests (no I/O).
3. Contract: order schema, state enum, examples, errors.
4. Database design package, then migration (manual apply).
5. Event log and idempotency.
6. Partner adapter behind a trait, with a fake for tests.
7. Reviewer tooling for `UnderReview`.

## Review history

(empty)
