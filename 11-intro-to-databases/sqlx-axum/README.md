# sqlx-axum — an Axum + sqlx Users CRUD API (starter)

A **scaffold**: the database + sqlx wiring is done for you; the Axum CRUD
layer is yours to build. This is the same shape as the finished
[`../sqlx-lab/`](../sqlx-lab) reference — implement it here first, then peek
at the reference to compare.

## What's already done

- **Schema** — `migrations/20250101000001_create_users.sql` (UUID PK, unique
  email, `updated_at` trigger).
- **Pool + migrations on startup** — `src/main.rs` connects a `PgPool` and
  runs `sqlx::migrate!()` on boot.
- **Shared state** — `src/state.rs` (`AppState { pool }`, `Clone`).
- **Error plumbing** — `src/error.rs` (`AppError` + `IntoResponse`, maps
  `sqlx::Error` → 404/409/500).
- **Row model** — `src/models.rs` (`User`, `FromRow + Serialize`).
- **A sample sqlx query** — `GET /health` runs `SELECT COUNT(*) FROM users`
  against the pool. This is your proof the wiring works.

## What's left for you (TODO)

- [ ] `src/models.rs` — add the request/query DTOs (`CreateUserRequest`,
      `UpdateUserRequest`, `ListQuery`). Suggested shapes are in a comment.
- [ ] `src/db/users.rs` — implement the repository: `create_user`, `get_user`,
      `list_users`, `update_user`, `delete_user`. Use `query_as!` / `query!`.
      Suggested signatures are in a comment.
- [ ] `src/routes/users.rs` — implement the five handlers (`create`, `get`,
      `list`, `update`, `delete`). Keep them thin: extract → call one repo
      function → map to `AppError`. The contract for each is in a comment.
- [ ] `src/router.rs` — uncomment the `/users` sub-router block in
      `build_app` once the handlers compile.
- [ ] Verify with the `curl` commands below.

## Build setup

`query_as!` checks your SQL against the live schema **at compile time**, so
the DB must exist and have migrations applied before `cargo build` will
succeed once you've added macro queries. (The starter as-shipped compiles
without a DB because `/health` uses the runtime `sqlx::query_as`.)

```sh
# 1. Postgres (Docker)
docker run --name pgdev -e POSTGRES_PASSWORD=secret -p 5432:5432 -d postgres:16

# 2. sqlx CLI
cargo install sqlx-cli --no-default-features --features native-tls,postgres

# 3. Configure + create the database
cd 11-intro-to-databases/sqlx-axum
cp .env.example .env            # edit password/db name
export DATABASE_URL=postgresql://postgres:secret@localhost:5432/sqlx_axum
sqlx database create
sqlx migrate run                # applies ./migrations/*.sql

# 4. (optional) load demo users so GET /users returns data right away
psql "$DATABASE_URL" -f seed.sql

# 5. Build + run (migrations also re-apply automatically on startup)
cargo run
```

The server listens on **http://127.0.0.1:3014** (offset from `sqlx-lab`:3012
and `seaorm-lab`:3013 so they can run at the same time).

## Verify the wiring first

Before writing any CRUD, confirm the done parts work:

```sh
cargo run
# another shell:
curl -i http://127.0.0.1:3014/health
# → { "status": "ok", "service": "sqlx-axum", "users": 6 }   (6 if you seeded)
```

If `/health` shows a count, the pool + migrations + model all work. If you
skip `seed.sql`, `"users"` will be `0` — still a valid green light.

## Try the API (once you finish the TODO)

```sh
# Create
curl -i -X POST http://127.0.0.1:3014/users \
  -H 'Content-Type: application/json' \
  -d '{"name":"Neo","email":"neo@matrix.com"}'

# List (paginated)
curl -i 'http://127.0.0.1:3014/users?limit=10&offset=0'

# Get one (paste the id from the create response)
curl -i http://127.0.0.1:3014/users/<id>

# Partial update
curl -i -X PUT http://127.0.0.1:3014/users/<id> \
  -H 'Content-Type: application/json' \
  -d '{"is_active":false}'

# Delete
curl -i -X DELETE http://127.0.0.1:3014/users/<id>

# Duplicate email → 409
curl -i -X POST http://127.0.0.1:3014/users \
  -H 'Content-Type: application/json' \
  -d '{"name":"Neo2","email":"neo@matrix.com"}'

# Missing user → 404
curl -i http://127.0.0.1:3014/users/00000000-0000-0000-0000-000000000000
```

## Suggested completion order

1. **Data Models** (`models.rs`) — quick win; unblocks the handlers' extractors.
2. **`create_user` + `create` handler** — gets a row in so you have an id to
   test the rest against. Wire `POST /users` in `router.rs`, `cargo run`,
   `curl` a create. Watch the unique-violation → 409 path too.
3. **`get_user` + `get`** — the canonical `fetch_optional` → 404 shape.
4. **`list_users` + `list`** — `fetch_all` + clamp `limit`/`offset`.
5. **`update_user` + `update`** — `COALESCE` partial update, `RETURNING`.
6. **`delete_user` + `delete`** — `rows_affected()` → 204 or 404.

After each handler, uncomment just its route line in `router.rs` and `curl`
it. By the end you've built the whole `sqlx-lab` reference yourself.

## Stretch goals

- Soft delete (`is_active = false`) instead of hard `DELETE`.
- Email format validation before insert → 422.
- A `POST`/`accounts` transfer endpoint wired to a transaction (see
  `../db-foundations/src/05_transactions.rs`).

## Reference

The finished version of exactly this API lives in
[`../sqlx-lab/`](../sqlx-lab). Try the lab yourself first; use the reference
to unblock yourself or to compare your style when you're done.