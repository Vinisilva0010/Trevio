# Design: money and amounts

- Class: STRICT. Security level: P0
- Status: **HOLD** (awaiting human confirmation and independent review)
- Date: 2026-11-22
- No code may be written from this draft until the status is "Approved".

## Goal

One safe way to represent, add, subtract, convert and round money across the
backend, the database and the API, so a rounding or overflow bug cannot create
or destroy value.

Out of scope: order flow (see the order state machine draft), fees policy
(business decision), the partner API, signing, any database table.

## Sources and assumptions that may be wrong

Sources: product description (BRL via Pix, USDC/USDT on Solana); SPL token
standard (USDC and USDT use 6 decimals on Solana).

Assumptions, each possibly wrong:

1. BRL has 2 decimals (centavos) everywhere, including at the partner.
2. USDC and USDT both have 6 decimals on the Solana mint we use. Verify the
   mint metadata, do not hardcode blindly per chain.
3. Amounts in this domain are never negative; direction is modeled separately
   (credit/debit), not by sign.
4. The partner quotes a rate we can express as an integer with a fixed scale,
   and quotes carry an expiry we can trust.
5. `u64` is large enough: 2^64 base units of a 6-decimal token is about
   1.8e13 whole tokens.
6. Rounding direction preferences below (always against Trevio's exposure)
   match what the partner contract and legal advice require.

## Smallest change that solves it

A small `money` module in the backend (own crate or module, no I/O):

- `Centavos(u64)` for BRL and `BaseUnits6(u64)` for USDC/USDT, with an
  explicit `Stablecoin` enum (`Usdc`, `Usdt`). Private inner field; no
  `From<f64>`, no `Into<f64>`, no `Deref`. Never mix types by accident.
- Arithmetic only through `checked_add`, `checked_sub`, `checked_mul`
  returning `Result<_, MoneyError>`. Overflow and underflow are errors, never
  wrap, never saturate silently.
- `Rate`: an integer price, "centavos per 1 whole stablecoin" scaled by 10^6
  (`rate_micro: u64`), so no float is involved. Conversion uses `u128`
  intermediates:
  - BRL -> stablecoin: `base_units = centavos * 10^12 / rate_micro`,
    **rounded down** (user never receives more than
    was paid for).
  - stablecoin -> BRL: `centavos = base_units * rate_micro / 10^12`, **rounded
    down** (never pay out more than the stablecoin is worth).
  - Fees are **rounded up** (never under-charge).
  - The exact scale constants and formulas are fixed in the review, with
    worked examples as test vectors.
- `Quote { id, pair, side, amount_in, amount_out, rate, fee, created_at,
  expires_at }`. A quote is immutable. Using a quote after `expires_at`, or
  with a mismatching amount/pair, is an error. Expiry is checked against the
  server clock at the moment of use.
- Wire format: JSON integers in minor units plus a currency code, for example
  `{"amount": 12345, "currency": "BRL"}`. Amounts above 2^53 - 1 are sent as
  decimal strings because JavaScript numbers lose precision beyond that. The
  contract states this explicitly.
- Database (when approved): `BIGINT` with `CHECK (amount >= 0)`; conversion
  `u64 <-> i64` is checked, and values above `i64::MAX` are rejected.

## Tests that prove it

Must pass:

- Add/sub/mul round trips on typical values.
- Documented conversion vectors (BRL -> USDC and back) with the exact
  expected results and rounding direction.
- Rounding: amount where the remainder is 0, below half, exactly half, above
  half, 1 minor unit; result is always the floor for conversions and the ceil
  for fees.
- Property test: converting there and back never returns more than the
  starting amount.

Must fail (each returns a typed error):

- `u64::MAX + 1`, `0 - 1`, multiplication overflow.
- Conversion with `rate_micro = 0`.
- Adding BRL to USDC (does not compile; a compile-fail test documents it).
- Using an expired quote; using a quote for another pair or amount.
- JSON with a float (`12.34`), a negative number, a string for a small
  amount, a number above 2^53 - 1 sent as a number, or a missing currency.
- Database value above `i64::MAX`.

## Impact

- API: every amount in `contracts/openapi.yaml` uses one shared `Money`
  schema (integer or decimal string + currency). Contract change, so STRICT.
- Database: no table is created by this design; it only fixes column types
  for later designs.
- Permissions: none by itself. Quotes will be created only by authenticated
  users once auth exists.

## Failure rules

- Any arithmetic error aborts the operation; nothing is partially applied.
- A partner rate that is missing, zero, stale, or does not parse is "no
  quote": fail closed, show the user an error, never fall back to a cached or
  default rate.
- A timeout or unknown result when fetching a rate means no quote.
- Clock problems (expiry in the past at creation, absurd far future) reject
  the quote.

## Implementation order (after approval)

1. `money` module with types, errors and unit tests (no I/O).
2. Property tests and conversion vectors agreed in review.
3. `Money` schema in `contracts/openapi.yaml` with examples.
4. JSON serialization/deserialization with the failing cases above.
5. Quote type and expiry logic.
6. Only then: database column conventions in the order design.

## Review history

(empty)
