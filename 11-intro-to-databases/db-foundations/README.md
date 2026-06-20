# sqlx

## Setup (one time)

Install Postgres [directly here](https://www.enterprisedb.com/downloads/postgres-postgresql-downloads) or follow docker method below and after that, install [PGAdmin](https://www.pgadmin.org/download/) to view your database in a GUI.


```sh
# 1. Postgres via Docker
docker run --name pgdev -e POSTGRES_PASSWORD=secret -p 5432:5432 -d postgres:18

# 2. sqlx CLI (for `database create` if you don't use the embedded migrate!())
cargo install sqlx-cli --no-default-features --features native-tls,postgres

# 3. Connection string + database
cd 11-intro-to-databases/db-foundations
cp .env.example .env            # edit password/db name to match your Postgres
export DATABASE_URL=postgresql://postgres:secret@localhost:5432/test_db
# (the migrate!() helper creates the schema, but the DB itself must exist:)
sqlx database create
```

> `db_foundations::connect()` (in `src/lib.rs`) loads `.env`, opens a tuned
> `PgPool`, and runs `sqlx::migrate!("./migrations")` for you — so once the
> **database** exists, every `cargo run --bin NN_…` is self-contained.


## Examples

| Bin | Concept | Needs DB? |
|-----|---------|-----------|
| `01_why_databases` | The file-storage problem + ACID (a lost-update race, no DB) | no |
| `02_sql_fundamentals` | SELECT / WHERE / ORDER / LIMIT / LIKE / COUNT / INSERT / UPDATE / DELETE / RETURNING / INNER & LEFT JOIN | yes |
| `03_pool_and_models` | `PgPoolOptions`, `FromRow`, `fetch_one`/`fetch_optional`/`fetch_all` | yes |
| `04_crud` | Full Create/Read/Update/Delete as repository functions | yes |
| `05_transactions` | Atomic transfer with Rust `Drop`-driven rollback | yes |
| `06_migrations` | `sqlx::migrate!()`, the `_sqlx_migrations` table, CLI vs embedded | yes |

Run one:

```sh
cargo run --bin 02_sql_fundamentals
```

## Schema

`migrations/` holds two files applied in order:

- `20250101000001_create_users_and_posts.sql` — `users` (UUID PK, UNIQUE email,
  nullable `bio`) + `posts` (FK → `users` with `ON DELETE CASCADE`), plus
  indexes on the columns we filter/sort by.
- `20250101000002_create_accounts.sql` — `accounts(user_id, credits)` with a
  `CHECK (credits >= 0)` so the transaction example can demonstrate a transfer
  that *fails* instead of silently going negative.

## Compile-time verification

The `query_as!` macros connect to `DATABASE_URL` during `cargo build` and check
every query against the live schema. If you rename a column and forget to
update a query, you get a **compile error**, not a runtime crash.

For CI / machines without Postgres, cache the query metadata once:

```sh
cargo install sqlx-cli
cargo sqlx prepare        # writes a .sqlx/ directory
SQLX_OFFLINE=true cargo build
```

Commit the `.sqlx/` directory and CI builds without a database.