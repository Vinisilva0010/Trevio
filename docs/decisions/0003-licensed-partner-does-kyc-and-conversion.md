# 0003. A licensed partner (PSAV) does KYC and BRL <-> stablecoin conversion

- Status: Proposed
- Date: 2026-11-22

## Context

Receiving and paying with Pix and converting BRL to and from stablecoins are
regulated activities, as is identity verification. Trevio does not hold the
licence and never holds user funds.

## Decision (proposed)

Integrate with a licensed partner (PSAV) that performs KYC, issues and receives
Pix, and converts BRL <-> USDC/USDT. Trevio orchestrates the order and shows
status; the partner holds the fiat leg.

## Open points

- The partner is not chosen and no contract exists, so API shape, quote
  validity, settlement times and refund mechanics are unknown.
- Who the legal counterparty to the user is must be confirmed with legal advice.
- The order state machine draft depends on these answers.

## Consequences (if accepted)

- Partner integration is STRICT (money, contracts between services).
- Trevio must handle partner outages, timeouts and unknown results by failing
  closed.
- The partner's availability and terms become a business dependency.
