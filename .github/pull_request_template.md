## Risk

- Class: <!-- FAST / STANDARD / STRICT (see CONTRIBUTING.md) -->
- Security level: <!-- P0 / P1 / P2 / P3 -->
- Approved design package (required for STRICT): <!-- link or "n/a" -->

## What changed

<!-- What and why, in a few lines. -->

## How it was tested

<!-- Commands run and results; new tests added. -->

## Screenshots (UI changes)

<!-- One per state: loading, success, error, empty. Remove if not a UI change. -->

## Checklist

- [ ] Branch is `feature/YYYYMMDD-short-name`, created from `dev`
- [ ] `git log dev..HEAD --oneline` and `git diff --stat dev...HEAD` show only what I intend
- [ ] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` pass (backend)
- [ ] `contracts/openapi.yaml` updated first if an endpoint changed
- [ ] Money is integer minor units; no floats
- [ ] No secrets, keys, tokens or personal data in code, logs, screenshots or tests
- [ ] Docs updated if behavior changed
