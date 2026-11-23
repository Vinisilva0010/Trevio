# Tech debt

| ID | Item | Priority | Status |
| --- | --- | --- | --- |
| TD-001 | No LICENSE file: code ownership is not decided yet. Decide with the investor/partners, then add it | High | Open |
| TD-002 | Configure branch protection on GitHub for `main` and `dev` (required PR, required CI, no force-push) | High | Open |
| TD-003 | Enable GitHub private vulnerability reporting (SECURITY.md relies on it) | High | Open |
| TD-004 | `cargo-deny` is not installed locally, so `deny.toml` was never executed; the first CI run may need allow-list adjustments | Medium | Open |
| TD-005 | `contracts/openapi.yaml` lint (Redocly via npx in CI) was never run locally because Node is not installed; first CI run may need rule tweaks in `contracts/redocly.yaml` | Medium | Open |
| TD-006 | Frontend owner (designer) GitHub handle missing in `.github/CODEOWNERS` | Medium | Open |
| TD-007 | No Dockerfile or deploy pipeline for the API; no staging environment | Medium | Open |
| TD-008 | No CI for `app/` (lint, typecheck, generated-client freshness) until the app code arrives | Medium | Open |
| TD-009 | Server speaks plain HTTP; TLS termination and trusted-proxy handling must be decided before any non-local deploy | Medium | Open |
| TD-010 | No rate limiting, authentication or authorization (all STRICT, waiting for design review) | High | Blocked: design |
| TD-011 | No metrics or distributed tracing export; only JSON logs | Low | Open |
| TD-012 | `postgres:18` image is pinned by tag, not by digest (local dev only) | Low | Open |
| TD-013 | Toolchain pinned to Rust 1.100.0; define a periodic bump routine | Low | Open |
| TD-014 | Dev-dependency `serde_norway` parses the OpenAPI file in tests; revisit if a better-maintained YAML crate appears | Low | Open |
