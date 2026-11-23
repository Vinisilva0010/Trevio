# app

The Trevio mobile app (Expo / React Native) lives here. It is **not scaffolded
yet**: the designer will turn the Figma screens into code and add it to this
folder. Start with `docs/FRONTEND_INTEGRATION.md`.

## Proposed structure

```text
app/
  src/
    screens/        One folder per screen, built from the Figma frames
    components/     Shared UI pieces
    api/
      generated/    TypeScript client generated from contracts/openapi.yaml.
                    NEVER edited by hand.
    wallet/         Key generation and storage, signing. Always reviewed by @Vinisilva0010
    auth/           Login and session. Always reviewed by @Vinisilva0010
```

Rules that apply to everything here:

- Only the generated client talks to the backend.
- The app never computes money; it only formats amounts for display.
- Keys and secrets live only in the device secure storage.
- `src/wallet/`, `src/auth/` and `src/api/` require review by @Vinisilva0010
  (see `.github/CODEOWNERS`).
