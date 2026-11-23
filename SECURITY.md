# Security policy

## Reporting a vulnerability

Please report privately. Do **not** open a public issue or PR.

Use GitHub's private vulnerability reporting: open the repository's
**Security** tab and choose **Report a vulnerability**. Include what you found,
how to reproduce it, and the impact. We aim to acknowledge reports within
3 business days.

Please do not access data that is not yours, disrupt the service, or test
against production systems.

## Keys never leave the device

Trevio is self-custodial. Private keys and seed phrases are generated and kept
only on the user's phone, in the device secure storage. They are never sent to
the backend, never logged, never included in analytics, crash reports or
backups. Any code path that would transmit key material is a security bug.

## Secrets policy

- No secrets in the repository: no passwords, API keys, tokens, private keys,
  seed phrases, or real `.env` files. Only `.env.example` with placeholders.
- Secrets reach the application through environment variables at runtime.
- Logs never contain request or response bodies, credentials, or personal data.
- If a secret is committed by mistake, treat it as compromised: rotate it
  first, then clean up. Deleting the commit is not enough.
- Dependencies are checked in CI with `cargo-deny` (advisories, licenses,
  sources).
