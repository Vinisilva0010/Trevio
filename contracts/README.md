# Contracts

`openapi.yaml` is the **source of truth** for the HTTP API (OpenAPI 3.1).

- The contract changes **first**. An endpoint is added here, with request,
  response and error examples, before any server or app code is written.
- The server must match it. `api/tests/openapi.rs` fails when a route is
  served but not documented, or documented but not served.
- The app is built against a mock server and a client generated from it
  (see `docs/FRONTEND_INTEGRATION.md`).
- **Generated code is never edited by hand.** If the generated output is
  wrong, fix the contract and regenerate.
- Changing a contract between services is STRICT work (see `CONTRIBUTING.md`).
- Errors use RFC 9457 `application/problem+json`; the `Problem` schema is shared.
