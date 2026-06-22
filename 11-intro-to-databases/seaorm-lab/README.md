# seaorm-lab — an Axum + SeaORM Users + Posts API

Replace in-memory state with a real PostgreSQL database using SeaORM, with
relations, pagination, migrations on startup, and a clean `AppError` → HTTP
mapping.

## Directory structure

```
seaorm-lab/
├── Cargo.toml
├── .env.example
├── seed.sql          demo users + posts (idempotent; load before a live demo)
└── src/
    ├── main.rs           startup: env → connect → Migrator::up → serve
    ├── router.rs         build_app(): /users sub-router + /health, TraceLayer
    ├── state.rs          AppState { db } (Clone — DatabaseConnection is Arc-wrapped)
    ├── error.rs          AppError + IntoResponse (unique → 409, not-found → 404)
    ├── entities/
    │   ├── mod.rs
    │   ├── user.rs        User entity (has_many Post); password_hash is serde(skip)
    │   └── post.rs        Post entity (belongs_to User)
    ├── migration/         in-crate SeaORM migrations (raw DDL in up()/down())
    │   ├── mod.rs         Migrator
    │   ├── m20250101000001_create_users.rs
    │   └── m20250101000002_create_posts.rs
    └── handlers/
        ├── mod.rs         re-exports + health
        └── users.rs       list/get/create/update/delete/user_posts
```

## Setup & run

```sh
# 1. Postgres (Docker)
docker run --name pgdev -e POSTGRES_PASSWORD=secret -p 5432:5432 -d postgres:16

# 2. Configure + run (migrations apply automatically on startup — no CLI needed)
cd 11-intro-to-databases/seaorm-lab
cp .env.example .env            # edit password/db name
export DATABASE_URL=postgresql://postgres:secret@localhost:5432/seaorm_lab
cargo run

# (optional) Load demo users + posts so GET /users and /users/:id/posts return data:
psql "$DATABASE_URL" -f seed.sql
```

## Try the API

```sh
# Create a user
curl -i -X POST http://127.0.0.1:3013/users \
  -H 'Content-Type: application/json' \
  -d '{"email":"neo@matrix.com","username":"neo","password":"redpill"}'

# Paginated list
curl -i 'http://127.0.0.1:3013/users?page=0&page_size=10'

# Get one (use the id from create)
curl -i http://127.0.0.1:3013/users/1

# Partial update
curl -i -X PUT http://127.0.0.1:3013/users/1 \
  -H 'Content-Type: application/json' \
  -d '{"is_active":false}'

# The related-posts endpoint (find_related) — empty until you add posts
curl -i http://127.0.0.1:3013/users/1/posts

# Delete
curl -i -X DELETE http://127.0.0.1:3013/users/1

# Duplicate email → 409
curl -i -X POST http://127.0.0.1:3013/users \
  -H 'Content-Type: application/json' \
  -d '{"email":"neo@matrix.com","username":"neo2","password":"x"}'

# Missing user → 404
curl -i http://127.0.0.1:3013/users/999999
```

## What this??

- **AppState integration** — `DatabaseConnection` is Arc-wrapped internally, so
  `AppState` derives `Clone` without an extra `Arc`.
- **Model / ActiveModel / Entity** — `Set` for writes, `.into()` from a fetched
  `Model` for partial updates (only `Set` columns are emitted in the UPDATE).
- **Relations** — `has_many` (user) / `belongs_to` (post), and the
  `/users/:id/posts` endpoint via lazy `find_related`.
- **Pagination** — `paginate(db, page_size)` + `num_pages()` + `fetch_page()`,
  capped at 100 per page.
- **Migrations as code** — `Migrator::up(&db, None)` on startup; the binary is
  self-migrating (great for containers). Each migration runs once, tracked in
  `seaql_migrations`.
- **Clean error handling** — one `AppError` + `IntoResponse`; `?` in handlers
  converts `DbErr`; unique violations → 409, not-found → 404, else → 500 with a
  sanitized message (raw error only in logs).

## Stretch goals

- `POST /users/:id/posts` to create a post for a user
- `GET /users` with `find_with_related(post::Entity)` to embed posts per user
- Email format validation before insert → 422
