# Advancing with Rust (2026)

Hands-on repository for the **Advancing with Rust** internship at **Silicon** (2026).

This repo collects the code written while working through the Rust fundamentals,
one numbered module at a time. Each module is self-contained so it can be built
and run on its own.

## Prerequisites

- [Rust toolchain](https://www.rust-lang.org/tools/install) (`rustc` + `cargo`), edition 2024
  - Built against `rustc 1.93.0` / `cargo 1.93.0`

Check your install:

```sh
rustc --version
cargo --version
```

## Modules

| # | Module | What it covers |
|---|--------|----------------|
| 00 | [`00-cargo-and-rustc`](./00-cargo-and-rustc) | Compiling directly with `rustc` vs. managing a project with Cargo; crates and `Cargo.toml` |
| 01 | [`01-variables-and-data-types`](./01-variables-and-data-types) | `let`/`mut`, constants, scalar types (integers, floats, bool, char), and `as` type casting |
| 02 | [`02-functions-and-control-flow`](./02-functions-and-control-flow) | Defining functions and return values; `if`/`else` as statements and expressions; `match`; `loop`/`while`/`for` |
| 03 | [`03-ownership_borrowing`](./03-ownership_borrowing) | The three ownership rules, moves vs. the `Copy` trait, transferring ownership in/out of functions, and `clone` |
| 04 | [`04-collections`](./04-collections) | Fixed-size arrays and slices, `Vec` with `push`/`insert`/`remove`, `String` vs. `&str`, tuples (including destructuring and matching), and `HashMap` with insert/get/contains_key/remove |
| 05 | [`05-structs-and-enums`](./05-structs-and-enums) | Named-field, tuple, and unit structs; `impl` blocks with associated functions and `&self`/`&mut self`/`self` methods; enums with unit, tuple, and struct-like variants; pattern matching enum data; `impl` on enums |
| 06 | [`06-traits-and-generics`](./06-traits-and-generics/traits) | Defining traits with required and default methods; `impl Trait for Type` for your own and foreign types; the orphan rule; associated functions, methods, and `Self` parameters; disambiguating same-named methods across traits with fully qualified `<Type as Trait>::method` syntax |
| 07 | [`07-error-and-file-ops`](./07-error-and-file-ops) | Error handling with `panic!`, the `Result<T, E>` enum (`Ok`/`Err`), the `Option<T>` enum (`Some`/`None`), propagating errors with the `?` operator, and `unwrap` / `expect` / `unwrap_or` |
| 08 | [`08-async-tokio`](./08-async-tokio) | The C10K problem and why async exists, `Future`s (lazy `poll()`/`Poll::{Ready,Pending}`), `async`/`.await` with `tokio::join!` and `?` error handling, the Tokio runtime (executor + reactor + work-stealing scheduler), `tokio::spawn` and `JoinHandle` vs `join!`, concurrency vs parallelism, common pitfalls (blocking the executor, forgetting `.await`, holding `std::sync::Mutex` across `.await`) |
| 09 | [`09-http-hyper-axum`](./09-http-hyper-axum) | Building a raw Hyper 1.x server (`Request<Incoming>` / `Response<Full<Bytes>>` / `TokioIo`), Hyper vs Axum side-by-side, the Axum `Router` with method chaining (`get().post()`), `Path` / `Query` / `Json` / `State` extractors, response shapes (`&str`, `(StatusCode, T)`, `Json<T>`, `Html<T>`, `Response::builder()`), sharing state with `Arc<RwLock<T>>`, nested routers with `.nest()`, and a custom `AppError` enum implementing `IntoResponse` with in-memory User CRUD (`POST` / `GET` / `PUT` / `PATCH` / `DELETE` / `?search=`) |
| 10 | [`10-axum-deep-dive`](./10-axum-deep-dive) | Four Axum 0.8 projects: `routers` (method chaining + auto 404/405/OPTIONS, `{id}` path params with typed `Path<T>`/`Query<T>`/`Json<T>`, `.nest()`/`.merge()`, `Arc<RwLock<T>>` state via `State<T>`, `IntoResponse`), `error-handling`, `middleware` (`from_fn` `(Request, Next) -> Response`, `TraceLayer`, request-id extensions, JWT auth with `jsonwebtoken`, `CorsLayer` + token-bucket rate limit via `from_fn_with_state`), and `crud-starter` — a multi-file CRUD assignment wiring it all together with selective middleware on nested routers, CORS outermost, and a `chain_order` demo of IN/OUT execution order |
| 11 | [`11-intro-to-databases`](./11-intro-to-databases) | Four PostgreSQL projects: `db-foundations` (sqlx 0.8 compile-time-verified `query!`/`query_as!` macros — connection pooling with `PgPoolOptions`, `FromRow` models, UUID/`TIMESTAMPTZ`/`CHECK`/FK `ON DELETE CASCADE`, ACID transactions with Rust Drop-driven ROLLBACK, and `sqlx::migrate!()`), `sqlx-lab` (Axum 0.8 + sqlx Users API with `COALESCE` partial updates and `thiserror` `AppError`→HTTP mapping), `seaorm-basics` (SeaORM 1.1 — `DeriveEntityModel`, the Model/ActiveModel/Entity trinity, `has_many`/`belongs_to` + `find_with_related`, `Condition::any/all`, `PaginatorTrait`, raw SQL, and `MockDatabase` tests with no DB needed), and `seaorm-lab` (Axum 0.8 + SeaORM Users/Posts API with in-crate `Migrator` self-migrating on startup) |

### 00: Cargo and rustc

- `main.rs`: a standalone file compiled directly with `rustc`.
- `silicon/`: the first Cargo project (`cargo new`).

### 01: Variables and data types

- `vars/`: Cargo project exploring immutability and `mut`, the `const PI`,
  integer/float/bool/char types, byte representation of characters, and numeric
  casting with `as` (including overflow wrap-around and float precision).

### 02: Functions and control flow

- Cargo project covering custom functions with typed parameters and return
  values (and early `return`), `if`/`else` chains and `if` as an expression,
  `match` (catch-all `_`, or-patterns, ranges, block arms), and the three loop
  forms (`loop`, `while`, `for`) with `break`, `continue`, `step_by`, `rev`,
  and `enumerate`. Many variants are kept commented out as a learning trail.

### 03: Ownership and borrowing

- `03-ownership_borrowing/`: Cargo project working through Rust's three ownership
  rules (single owner, drop on scope exit, ownership transfer), how moves
  invalidate the source variable, returning ownership back from a function, the
  `Copy` trait for stack types like integers, and `clone` for duplicating heap
  data such as `String`.

### 04: Collections

- `04-collections/`: Cargo project exploring Rust's main collection types —
  fixed-size arrays (literals, `[value; len]`, indexing, `.len()`) and slices
  (`&arr[a..b]`, `arr.get(range)` returning `Option`); `Vec<T>` built with
  `Vec::new()` and `vec![]`, mutated with `push`/`insert`/`remove` and walked by
  reference; `String` vs. `&str`, `+` concatenation, and iterating `.chars()`;
  tuples — declaration, positional access, destructuring with `let (.., x)`,
  pattern matching on shape, and using tuples as multi-value return types; and
  `HashMap<K, V>` with `insert`/`get`/`contains_key`/`remove` and iteration
  over keys and `(key, value)` pairs.

### 05: Structs and enums

- `05-structs-and-enums/`: Cargo project covering all three struct shapes
  (named-field `Person { ... }`, tuple structs `Meter(f64)`, and unit structs
  `Empty`), accessing and mutating fields, `impl` blocks that hold associated
  functions (called via `::`, like `Rectangle::new`) and methods taking
  `&self` / `&mut self` / owned `self`, splitting logic across multiple `impl`
  blocks, and the enums side: unit-only enums (`TrafficLight`), variants that
  carry tuple or struct-like data (`Move::Walk(i32, i32)`,
  `Move::SwordColor { r, g, b }`), and matching on those variants to pull the
  inner values out — including `impl` blocks on enums that dispatch behaviour
  by variant.

### 06: Traits and generics

- `06-traits-and-generics/traits/`: Cargo project introducing traits as
  shared behaviour contracts — defining a trait with required associated
  functions, methods, and a default-implemented method; implementing one trait
  for several of your own types and for foreign types (e.g. `Size for i32`,
  `Size for HashMap<...>`); the orphan rule that gates the latter; traits that
  return `Self` and take `&Self`; two traits defining a method with the same
  name and disambiguating the call with fully qualified
  `<Type as Trait>::method()` syntax; and `enum_revision.rs`, a side file
  revisiting an enum with an `impl` block to contrast methods (taking `&self`)
  with associated functions (taking the type by value).

### 07: Error and file operations

- `07-error-and-file-ops/`: Cargo project covering Rust's three main error
  channels — `panic!` for unrecoverable failures, the `Result<T, E>` enum
  (with `Ok`/`Err` variants and `match`-based handling) for recoverable
  failures, and the `Option<T>` enum (with `Some`/`None`) for values that may
  be absent (e.g. `Vec::get`); propagating errors from a callee to a caller
  with the `?` operator; and the quick-exit helpers `unwrap`, `expect`, and
  `unwrap_or`.

### 08: Async with Tokio

- `08-async-tokio/`: Cargo project introducing asynchronous Rust on top of
  the Tokio runtime. Starts with the **C10K problem** and why
  thread-per-request doesn't scale, then explains what a **`Future`** is
  (lazy, driven by `poll()` returning `Poll::Ready` / `Poll::Pending`),
  walks through `async` / `.await` syntax, **concurrent** execution with
  `tokio::join!`, and error handling with `?`. Goes inside the **Tokio
  runtime** (executor + reactor + work-stealing scheduler) and contrasts
  `tokio::spawn` (independent task with a `JoinHandle`) with `join!`
  (cooperative concurrency on a single task). Clarifies the **concurrency
  vs. parallelism** distinction, and closes with the most common **async
  pitfalls**: blocking the executor, forgetting `.await`, and holding a
  `std::sync::Mutex` across `.await` (and the `tokio::sync::Mutex` fix).

### 09: HTTP, Hyper, and Axum 0.8

- `09-http-hyper-axum/`: Cargo project that goes from raw HTTP in Hyper 1.x to Axum 0.8.

### 10: Axum deep dive — routers, middleware, and error handling

- `10-axum-deep-dive/`: Four Cargo projects that take Axum 0.8 apart one
  concern at a time. **`routers/`** is five standalone binaries covering
  method chaining (`.get().post().delete()`) and Axum's free
  auto-responses (OPTIONS → 200 + `Allow`, unknown method → 405,
  unknown path → 404), `{id}` path params with typed `Path<T>` (auto-422
  on bad input) and tuple extraction, `Query<T>` with optional fields,
  `Json<T>` body extraction, organising routes by domain with `.nest()`
  (adds a prefix) vs `.merge()` (same level), `Arc<RwLock<T>>` state via
  `.with_state()` and the `State<T>` extractor, and the `IntoResponse`
  trait (`&str`, `(StatusCode, T)`, `Json`, `Html`, custom headers,
  `Result<T, E>`). **`error-handling/`** is five binaries: a hand-rolled
  `AppError` enum whose single `IntoResponse` impl owns the
  status-to-JSON mapping so `?` just works, `thiserror`
  (`#[derive(Error)]`, `#[error("...")]`, `#[from]` for auto `From`,
  `#[error(transparent)]`), `anyhow` (`.with_context()`, `bail!()`, the
  cause chain via `{:#}`), a three-layer error model
  (`RepositoryError` → `ServiceError` → `AppError` with `From` impls as
  the layer boundaries), and panic safety via `panic::set_hook` +
  `CatchPanicLayer`. **`middleware/`** is five binaries: `from_fn`
  middleware as an `async fn(Request, Next) -> Response` with the
  `next.run(req).await` pass-through, `TraceLayer` +
  `tracing-subscriber`/`RUST_LOG`, a UUID request-id middleware writing
  to and reading from request extensions, JWT auth with `jsonwebtoken`
  (Bearer token → `Claims` in extensions, public vs protected routers),
  and `CorsLayer` plus a per-IP token-bucket rate limiter using
  `from_fn_with_state` — with the key rule that the **last** `.layer()`
  is the **outermost**. **`crud-starter/`** is a multi-file CRUD
  assignment (`main`/`router`/`error`/`models`/`state`, `routes/`, a
  `middleware/` tree including a raw `tower::Service`/`tower::Layer`
  impl, and a `chain_order` binary logging the IN/OUT execution order)
  that stitches the three together and demonstrates **selective**
  middleware on nested routers (auth + timing on `/users`, nothing on
  `/health`), CORS placed outermost so preflight short-circuits, and a
  stateful rate limiter keyed by client IP.

### 11: Intro to databases — sqlx, SeaORM, and PostgreSQL

- `11-intro-to-databases/`: **`db-foundations/`** is six standalone binaries built on
  sqlx 0.8's compile-time-verified `query!`/`query_as!` macros: it opens
  with a flat-file lost-update race (motivating ACID), then walks through
  raw SQL fundamentals, connection pooling with `PgPoolOptions`
  (`max_connections`/`min_connections`/`acquire_timeout`), `FromRow`
  structs over UUID PKs / `TIMESTAMPTZ` / `CHECK` / FK `ON DELETE
  CASCADE`, full CRUD with `RETURNING`, atomic transfers in a
  transaction where early-return drops the `Transaction` and triggers
  ROLLBACK via `Drop` (no explicit rollback needed), and schema
  evolution with `sqlx::migrate!()` + an embedded migrations folder.
  **`sqlx-lab/`** is the sqlx lab — an Axum 0.8 Users API over sqlx with
  `COALESCE`-based partial `UPDATE`s and a single `thiserror` `AppError`
  whose `IntoResponse` maps unique-violation→409 and `RowNotFound`→404.
  **`seaorm-basics/`** is seven binaries on SeaORM 1.1 — `connect`/
  `setup`/`reset` helpers, `DeriveEntityModel` and the Model /
  ActiveModel / Entity trinity, `has_many`/`belongs_to` relations with
  `find_with_related` / `find_also_related`, filtering with
  `Condition::any`/`all`, `PaginatorTrait` pagination, joins via
  `QuerySelect`, `ColumnTrait` + `sea_query::ExprTrait` custom queries,
  and `MockDatabase`-based unit tests that run with **no database
  running**. **`seaorm-lab/`** is the SeaORM lab — an Axum 0.8 + SeaORM
  Users/Posts API whose binary is **self-migrating** (`Migrator::up` on
  startup, raw DDL in `up()`/`down()`, tracked in `seaql_migrations`),
  with `find_related`, paginated responses, and an `AppError` → HTTP
  mapping. 


  **Note:** the sqlx projects use `query!`/`query_as!` macros
  that are verified at *compile* time, so they need a live Postgres and
  `DATABASE_URL` to build (Docker one-liner in each README); the SeaORM
  projects compile with no database at all.

## Running the code

**Cargo projects** (e.g. `silicon`, `vars`). `cd` into the project directory:

```sh
cd 01-variables-and-data-types/vars
cargo run
```

**Standalone files** compiled with `rustc`:

```sh
cd 00-cargo-and-rustc
rustc main.rs && ./main
```

**Module 06 lives one level deeper** than the other modules — it has its own
sub-crate `traits/`:

```sh
cd 06-traits-and-generics/traits
cargo run
```

**Module 11 needs a database.** The `db-foundations` and `sqlx-lab`
projects use sqlx's compile-time `query!`/`query_as!` macros, which
verify SQL against a *live* Postgres at build time — so `cargo build`
fails with `error: set DATABASE_URL` until a database is reachable.
Quick start (full details in each project's README):

```sh
# Postgres via Docker
docker run --name pgdev -e POSTGRES_PASSWORD=secret -p 5432:5432 -d postgres:16
export DATABASE_URL=postgresql://postgres:secret@localhost:5432/<db_name>

cd 11-intro-to-databases/db-foundations
cargo run --bin 02_sql_fundamentals    # sqlx: needs DATABASE_URL at build time
```

## Layout

```
rust-2026/
├── 00-cargo-and-rustc/
│   ├── main.rs              # compiled with rustc directly
│   └── silicon/             # cargo project
├── 01-variables-and-data-types/
│   └── vars/                # cargo project
├── 02-functions-and-control-flow/   # cargo project
├── 03-ownership_borrowing/         # cargo project
├── 04-collections/                 # cargo project
├── 05-structs-and-enums/           # cargo project
├── 06-traits-and-generics/
│   └── traits/             # cargo project
├── 07-error-and-file-ops/         # cargo project
├── 08-async-tokio/                # cargo project
├── 09-http-hyper-axum/            # cargo project
├── 10-axum-deep-dive/
│   ├── routers/            # cargo project — routing examples
│   ├── error-handling/     # cargo project — error-handling examples
│   ├── middleware/         # cargo project — middleware examples
│   └── crud-starter/       # cargo project — multi-file CRUD assignment
└── 11-intro-to-databases/
    ├── db-foundations/     # cargo project — sqlx query!/query_as! examples
    ├── sqlx-lab/           # cargo project — Axum + sqlx Users API lab
    ├── seaorm-basics/      # cargo project — SeaORM entity/relation/query examples
    └── seaorm-lab/         # cargo project — Axum + SeaORM Users/Posts API lab
```
