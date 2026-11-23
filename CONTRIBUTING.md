# Contributing

## Branches

- `main` is protected. It only receives reviewed merges from `dev`.
- `dev` is the integration branch.
- Work happens on `feature/YYYYMMDD-short-name`, created **from `dev`**,
  for example `feature/20261201-health-endpoints`.
- Never force-push a shared branch (`main`, `dev`, or any branch someone else
  uses). Fix mistakes with a new commit.

## Commits

[Conventional Commits](https://www.conventionalcommits.org/), in English:

```text
feat(api): add readiness endpoint
fix(api): return 503 when the database times out
docs: describe the order state machine draft
chore(ci): pin actions to commit SHAs
test(api): cover invalid BIND_ADDR
refactor(api): split config parsing from validation
```

## Pull requests

Before opening a PR, check exactly what it carries:

```bash
git log dev..HEAD --oneline
git diff --stat dev...HEAD
```

Fill in the PR template. Checklist:

- [ ] Risk class and security level declared
- [ ] Only the intended files and commits are included
- [ ] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` pass
- [ ] `contracts/openapi.yaml` changed first if an endpoint changed
- [ ] Money as integer minor units, no floats
- [ ] No secrets or personal data anywhere
- [ ] Docs updated

## Risk classes

| Class | When | Process |
| --- | --- | --- |
| FAST | Small, low-risk, easy to revert (typos, docs, tooling) | Normal PR review |
| STANDARD | Regular features and fixes | PR review plus tests |
| STRICT | Money, auth/permissions, keys, database schema, contracts between services, or more than one repo | An **approved design package and an independent review before any implementation** |

## Security levels

| Level | Scope |
| --- | --- |
| P0 | Money, signing, execution |
| P1 | Login, credentials, account linking |
| P2 | Regular service code |
| P3 | Documentation |

P0 and P1 are always STRICT. When unsure, pick the stricter class.

## Rules of thumb

- Money is integer minor units; never floats.
- The database is never migrated at startup (see `api/migrations/README.md`).
- Generated code is never edited by hand.
- Comments explain *why*. Repository content is in English.
