# Frontend integration guide

For the designer who turns the Figma screens into the app. No backend
knowledge needed. Everything the app needs from the backend is described in
one file: `contracts/openapi.yaml`.

## 1. How we work: contract first

1. **Backend writes the endpoint in `contracts/openapi.yaml` first**, with
   example requests, responses and errors.
2. **You build the screen against a mock server** generated from that file.
   You do not wait for the real backend.
3. **The real backend must match the mock.** If it does not, that is a backend
   bug, not something to work around in the app.
4. If you need a new field or endpoint, ask for a contract change. Do not
   invent one.

## 2. Requirements

Node.js (LTS) and npm must be installed. They are not installed on every
machine: get them from https://nodejs.org/. Check with `node --version`.

## 3. Run the mock server

Prism reads the contract and answers with its examples. Tool version checked
on the npm registry: `@stoplight/prism-cli` 5.16.0.

```bash
npx --yes @stoplight/prism-cli@5.16.0 mock contracts/openapi.yaml --port 4010
```

Then call it, for example:

```bash
curl -i http://127.0.0.1:4010/health
```

Point the app's base URL to `http://127.0.0.1:4010` while developing (on an
Android emulator use `http://10.0.2.2:4010`; on a physical phone use your
computer's local IP and run Prism with `--host 0.0.0.0`).

## 4. Generate the TypeScript client

Tool version checked on the npm registry: `openapi-typescript` 7.13.0. It
generates types from the contract; use a small typed fetch wrapper on top.

```bash
npx --yes openapi-typescript@7.13.0 contracts/openapi.yaml -o app/src/api/generated/schema.d.ts
```

Re-run it whenever `contracts/openapi.yaml` changes, and commit the result.
**Never edit files in `app/src/api/generated/` by hand.** If the output is
wrong, the contract is wrong: ask for a contract fix and regenerate.

Note: these commands were written but not executed on the machine that
bootstrapped the repo (Node was missing). If one fails, report it instead of
working around it.

## 5. Rules

**Money**

- The app **never computes money**. Amounts arrive from the API as integers
  in minor units with a currency (BRL in centavos; USDC/USDT in 6-decimal base
  units). The app only formats them for display.
- No floats, no `parseFloat`, no `Number` arithmetic on amounts. Do not add,
  convert or round amounts in the app. If a screen needs a total or a
  conversion, the API must provide it.
- Large amounts may arrive as strings; keep them as strings or `BigInt`.

**Secrets and keys**

- Keys, seed phrases and tokens live only in the device secure storage
  (Keychain / Keystore). Never in AsyncStorage, files, state that is
  persisted, URLs, or the repo.
- Code for keys and login lives only in `app/src/wallet/` and `app/src/auth/`
  and is reviewed by @Vinisilva0010.

**Logging**

- Never log personal data (CPF, name, e-mail, phone, Pix keys, addresses
  tied to a person), tokens, keys, or request/response bodies. Remove debug
  logs before opening a PR.

## 6. Showing API errors

Errors come as `application/problem+json`:

```json
{
  "type": "urn:trevio:problem:not-ready",
  "title": "Service Not Ready",
  "status": 503,
  "detail": "A required dependency is unavailable. Try again later."
}
```

- Decide what to do from `type` and `status`, not from the text.
- `detail` is safe to show to the user, but prefer your own friendly,
  translated copy chosen by `type`.
- Network failure or timeout (no response) is its own state: "no connection,
  try again". Never assume success.
- 5xx: generic "something went wrong, try again" with a retry button.
- Always keep the `x-request-id` response header available for support, but
  do not show it unless a screen design calls for it.

## 7. Order states the UI must handle

These come from the order design draft
(`docs/design/20261122-order-state-machine/design.md`, status HOLD, so names
may change after review). Every state needs a design in Figma.

| State | Meaning | What the user should see |
| --- | --- | --- |
| Created | Order made, payment not ready | "Preparing your payment..." with a spinner |
| AwaitingPix | Waiting for the user's Pix | Pix code/QR, amount, countdown to expiry, copy button |
| PixReceived | Pix arrived | "Payment received. Checking..." |
| Converting | Converting BRL to stablecoin | "Converting your money..." progress |
| Sent | Stablecoin sent, not final yet | "On its way to your wallet..." |
| ConfirmedOnChain | Done | Success screen, final amount, receipt |
| Expired | Not paid in time | "This payment expired" with a button to start a new one |
| UnderReview | A person is checking the order | "We are reviewing your order. This can take a while." No action to take; do not promise a time |
| Held24h | Held for up to 24 hours | "For your security, this will complete within 24 hours" |
| Rejected | Could not be completed | "We could not complete this order" and how to get help |
| Refunded | Money returned | "Your payment was refunded" |

An **unknown state** from the server (new state the app does not know) must
show a neutral "Check this order later" screen, never crash and never pretend
it succeeded.

## 8. Screen to endpoint map (template)

Copy this table into the PR description or a file in `app/` and fill it in.

| Screen | Endpoint | Loading | Success | Error | Empty | Figma frame |
| --- | --- | --- | --- | --- | --- | --- |
| _example: Status check_ | `GET /health` | spinner | "OK" | retry banner | n/a | _link to frame_ |
| | | | | | | |

Every screen needs all four states designed: loading, success, error, empty.

## 9. Handoff checklist (every PR)

- [ ] Branch created from `dev`: `feature/YYYYMMDD-short-name`
- [ ] Only touches `app/` (anything else was agreed with the backend first)
- [ ] Uses only the generated client; no hand-written calls to the API
- [ ] `app/src/api/generated/` regenerated, not edited by hand
- [ ] Screenshots of each state: loading, success, error, empty
- [ ] No secrets, keys, tokens or personal data in code, logs or screenshots
- [ ] No money arithmetic in the app
- [ ] Screen-to-endpoint table updated
- [ ] CI is green
