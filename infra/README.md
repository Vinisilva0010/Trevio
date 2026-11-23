# infra

Local development infrastructure. **Not** for staging or production.

## Start Postgres

```bash
cp .env.example .env        # then edit the password; .env is git-ignored
docker compose --env-file .env -f infra/docker-compose.yml up -d
docker compose -f infra/docker-compose.yml ps   # wait for "healthy"
```

Postgres listens on `127.0.0.1:5432` only. Compose refuses to start if
`POSTGRES_USER`, `POSTGRES_PASSWORD` or `POSTGRES_DB` is unset.

## Stop

```bash
docker compose --env-file .env -f infra/docker-compose.yml down      # keep data
docker compose --env-file .env -f infra/docker-compose.yml down -v   # delete data
```

## Run the real-database test

```bash
TREVIO_TEST_DATABASE_URL='postgres://USER:PASSWORD@127.0.0.1:5432/DBNAME' \
  cargo test -p trevio-api --test database -- --ignored
```

## Notes

- No schema exists yet; migrations are manual (see `api/migrations/README.md`).
- Postgres 18 stores data in a version-specific subdirectory, which is why the
  volume is mounted at `/var/lib/postgresql` and not `.../data`.
