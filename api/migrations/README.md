# Database migrations

**Status: no migrations exist yet, on purpose.**

Anything that touches the database schema is STRICT work (see
`CONTRIBUTING.md`). A migration needs an approved design package and an
independent review before it is written.

## Rules

1. The server **never** migrates at startup. A crashed or misconfigured
   deploy must not be able to alter the schema.
2. Migrations are applied by a person, manually, with a documented command.
3. Files are forward-only and named `NNNN_short_description.sql`
   (for example `0001_create_orders.sql`). Never edit a migration that has
   been applied anywhere; add a new one.
4. Each migration is reviewed in its own PR, with a note on how to roll
   forward if it fails.

## Command (once the first migration exists)

Requires `sqlx-cli` (a human installs it, it is not part of this repo):

```bash
cargo install sqlx-cli --no-default-features --features rustls,postgres
export DATABASE_URL='postgres://USER:PASSWORD@HOST:5432/DBNAME'
sqlx migrate run --source api/migrations
```

Use `sqlx migrate info --source api/migrations` first to see what is pending.
Never paste a real `DATABASE_URL` into an issue, PR or chat.
