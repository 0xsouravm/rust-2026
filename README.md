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
```
