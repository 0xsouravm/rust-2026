# Assignment
## crud-starter 

A multi-file CRUD scaffold you can build on while you learn routers,
middleware, and error handling in one sitting. The project **compiles
at the end of every step**, so you can always `cargo run` and see
something working before moving on.

## Directory structure

```
crud-starter/
├── Cargo.toml
└── src/
    ├── main.rs               entry point — tracing init, server boot
    ├── router.rs             build_app() — assembly, the "big picture"
    ├── error.rs              AppError + IntoResponse
    ├── models.rs             User, NewUser, UpdateUser, ListQuery
    ├── state.rs              AppState (Arc<RwLock<HashMap>>)
    ├── routes/
    │   ├── mod.rs            re-exports sub-routers
    │   ├── health.rs         /health
    │   └── users.rs          /users CRUD handlers + sub-router
    ├── middleware/           ← the section you came for
    │   ├── mod.rs
    │   ├── tower_service.rs  Tower::Service + Tower::Layer, un-sugared
    │   ├── timing.rs         custom timing middleware (from_fn)
    │   ├── logger.rs         visual IN/OUT logger (from_fn)
    │   ├── request_id.rs     X-Request-Id (from_fn + Set/Propagate)
    │   ├── auth.rs           X-API-Key (from_fn, selective)
    │   └── rate_limit.rs     token-bucket (from_fn_with_state)
    └── bin/
        └── chain_order.rs    standalone demo: execution order in logs
```

## Part 1 — CRUD / routers / errors (steps 1–5)

### Step 1 — Boot a minimal axum app
**Goal:** confirm the project runs. `routes/health.rs` is the only
real handler. Read `main.rs` top-to-bottom and notice:
- `#[tokio::main]` for the async runtime
- `TcpListener::bind` + `axum::serve`
- The `Router<()>` type returned by `build_app()`

### Step 2 — Build a sub-router (no nesting yet)
**Goal:** see how a handler is a fn and a sub-router is `Router<S>`.
In `routes/users.rs`, the `router()` function returns
`Router<AppState>` and the parent will consume it.

Notice the `Query<ListQuery>` and `Path<u64>` extractors — these are
how handlers read from the request.

### Step 3 — Nest and merge
**Goal:** understand the two composition operators. See `router.rs`:
- `merge` — add more routes at the **current** prefix level
- `nest` — mount a sub-router under a **new** prefix

The whole v1 is mounted at `/api/v1`, so final paths are
`/api/v1/health` and `/api/v1/users/{id}`.

### Step 4 — AppError + IntoResponse (error handling)
**Goal:** every handler returns `Result<T, AppError>` and the `?`
operator just works. The single IntoResponse impl owns the status
mapping AND the JSON shape — handlers never assemble an error
response manually.

### Step 5 — Add a middleware (request-id)
**Goal:** see how a middleware is just an async fn from `Request`
to `Response` that wraps `next.run(req)`.

## Part 2 — Middleware

You already know the CRUD is a `Router` with a state. Now we'll see
how requests flow through layers of code, what those layers are made
of, and how to compose them selectively.

### What a middleware is — three equivalent views

A middleware is:

1. **Conceptually** — code that runs before/after your handler.
   Inbound code → handler → outbound code. The handler is hidden
   inside `next.run(req).await`.
2. **Syntactically** — an `async fn(Request, Next) -> Response`.
   This is the `axum::middleware::from_fn` form.
3. **Under the hood** — a `tower::Layer` wrapping a `tower::Service`.
   See `src/middleware/tower_service.rs` for the un-sugared form.
   You will not write this by hand; `from_fn` is the right answer
   99% of the time. Read it once to see what's happening.

```rust
// Syntactic form
async fn my_middleware(req: Request, next: Next) -> Response {
    // ← INBOUND
    let res = next.run(req).await;
    // ← OUTBOUND
    res
}
```

### Custom timing middleware
**File:** `src/middleware/timing.rs`

```rust
let start = Instant::now();
let response = next.run(req).await;     // handler runs
let elapsed = start.elapsed();          // post-handler
tracing::info!("[timing] {} {} → {} ({} ms)",
    req.method(), req.uri().path(),
    response.status().as_u16(), elapsed.as_millis());
```

Used in `router.rs` on the `/users/{id}` route.

### Tower::Service / Tower::Layer (Raw Form)
**File:** `src/middleware/tower_service.rs`

what IS a layer really?

Layer consists of Two trait impls:
- `Service::call(&mut self, req)` returns a future
- `Layer::layer(self, inner)` wraps a service in another

`from_fn` is sugar for "implement this for me, please."

### Execution order: who runs first?
**File:** `src/bin/chain_order.rs`

This is the most important demo. Run it with `RUST_LOG=info` and tail
the log. Each layer prints `IN` and `OUT`:

```
cargo run --bin chain_order
# in another shell:
curl http://127.0.0.1:3011/api/v1/users/1
```

Logs:

```
[A:trace]      IN  → /api/v1/users/1
[B:cors]       IN  → /api/v1/users/1
[C:request_id] IN  → /api/v1/users/1
[D:timing]     IN  → /api/v1/users/1
[E:auth]       IN  → /api/v1/users/1
[F:rate_limit] IN  → /api/v1/users/1
                ← handler runs →
[F:rate_limit] OUT ← 200
[E:auth]       OUT ← 200
[D:timing]     OUT ← 200
[C:request_id] OUT ← 200
[B:cors]       OUT ← 200
[A:trace]      OUT ← 200
```

The rule: **the LAST `.layer()` is the OUTERMOST**. So the order
in source code is INNERMOST-first, OUTERMOST-last — opposite to how
the layers *execute* on a request.

### TraceLayer
**File:** wired in `main.rs` (`TraceLayer::new_for_http()`)

`TraceLayer` from `tower_http::trace` is the canonical example of a
middleware that is NOT a function. It's a struct, you call
`TraceLayer::new_for_http()`, you get back a Layer. It hooks into
the `tracing` crate and produces structured logs at the
DEBUG/INFO/TRACE level for every request — method, path, status,
latency.

```rust
.layer(TraceLayer::new_for_http())
```

`RUST_LOG=info cargo run` to see the output.

### Selective middleware on nested routers
**File:** `src/router.rs`

This is the most important lesson for real apps. The `/users`
sub-router has `timing` and `auth` applied; the `/health` sub-router
has nothing. So:

```sh
curl http://127.0.0.1:3010/api/v1/health         # 200, no API key needed
curl http://127.0.0.1:3010/api/v1/users/1        # 401, no API key
curl -H 'X-API-Key: letmein' http://127.0.0.1:3010/api/v1/users/1   # 200
```

How does it work? In `router.rs`:

```rust
let users: Router<AppState> = Router::new()
    .route("/{id}", routes::users::read_user_route()
        .layer(middleware::from_fn(timing_middleware))   // only /users/{id}
        .layer(middleware::from_fn(api_key_middleware))) // only /users/{id}
    .route("/", post(routes::users::create_user)
        .layer(middleware::from_fn(timing_middleware))
        .layer(middleware::from_fn(api_key_middleware))
        .layer(middleware::from_fn_with_state(rate_state, rate_limit_middleware)));  // POST only

let health: Router<AppState> = Router::new()
    .route("/health", axum::routing::get(routes::health::health));  // no layers
```

Two patterns in play:
1. **Apply to a sub-router** — the layer only sees the routes
   inside that sub-router. `/health` is in a different sub-router
   so the auth layer never sees it.
2. **Apply to a single route** — `MethodRouter::layer()` (used on
   `routes::users::read_user_route()`) layers the middleware on
   just that route.

The rate-limit middleware is applied only to POST via a different
mechanism — an internal `if req.method() != POST { return next.run(req).await }`
at the top of the function. Both styles are valid; the internal
check is easier when the layer and the rule are 1:1.

### CORS configuration
**File:** the `cors_layer()` fn in `src/router.rs`

CORS is NOT a function. It's a struct from `tower-http`:

```rust
fn cors_layer() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(["http://localhost:3000".parse().unwrap()])
        .allow_methods([Method::GET, Method::POST, ...])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE])
        .max_age(std::time::Duration::from_secs(3600))
}
```

Why does CORS go OUTERMOST? Because a CORS preflight is an
`OPTIONS` request that browsers send before the real request. If a
disallowed origin hits your auth middleware, you've already wasted
work. CORS short-circuits at the edge with a 200 (or 403).

To see preflight in action:

```sh
curl -i -X OPTIONS -H 'Origin: http://localhost:3000' \
     -H 'Access-Control-Request-Method: POST' \
     http://127.0.0.1:3010/api/v1/users
```

### Middleware that needs shared state
**File:** `src/middleware/rate_limit.rs`

Plain `from_fn` can't pass state in. The stateful variant is
`from_fn_with_state`:

```rust
.layer(middleware::from_fn_with_state(rate_state, rate_limit_middleware))
```

The middleware then takes `State<RateLimiterState>` as its first
argument. This is the same `State<T>` extractor handlers use, just
in a middleware.

Important pattern: `from_fn_with_state` requires the state to be
`Clone`. Our `RateLimiterState` is `Arc<Mutex<...>>` so cloning
just bumps a refcount.

The token-bucket algorithm is also worth a 5-minute aside:
- `capacity` is the burst size
- `refill_rate` is the long-term rate
- A request consumes 1 token; empty bucket → 429

## The full chain, end-to-end

For `POST /api/v1/users` (the only request that hits every layer):

```
                                 TraceLayer            ← (tower-http) logs, latency
                                  CORS                  ← (tower-http) OPTIONS preflight
                                   request_id (hand)    ← X-Request-Id
                                    request_id (tower)  ← propagates to response
                                     ── nest /api/v1 ──
                                       /users router
                                        timing           ← measures per request
                                         auth            ← X-API-Key check
                                          rate_limit     ← POST only, token-bucket
                                           handler
```

For `GET /api/v1/health` (the public endpoint), the chain is:

```
TraceLayer → CORS → request_id → health handler
```

The layers between `request_id` and the handler are completely
absent. That's the selective-application lesson.

## Try it all

```sh
# Terminal 1
RUST_LOG=info cargo run

# Terminal 2
curl -i http://127.0.0.1:3010/api/v1/health
curl -i http://127.0.0.1:3010/api/v1/users/1
curl -i -H 'X-API-Key: letmein' http://127.0.0.1:3010/api/v1/users/1
curl -i -X POST -H 'X-API-Key: letmein' -H 'Content-Type: application/json' \
     -d '{"name":"Morpheus","email":"m@zion.io"}' \
     http://127.0.0.1:3010/api/v1/users

# CORS preflight
curl -i -X OPTIONS -H 'Origin: http://localhost:3000' \
     -H 'Access-Control-Request-Method: POST' \
     http://127.0.0.1:3010/api/v1/users

# Execution order demo (separate binary)
cargo run --bin chain_order
curl http://127.0.0.1:3011/api/v1/users/1
```

## Assignment goals

The starter deliberately leaves a few things out. Now that you have
the patterns, finish them:

- `PUT /users/{id}` — full update (mirror PATCH)
- JWT in place of X-API-Key (see `10-axum-deep-dive/middleware/src/04_jwt_auth.rs`)
- Per-user rate limit (key the bucket by the JWT subject (user id), not the IP)
- A separate `tower::Layer` impl of `timing` (mirror
  `src/middleware/tower_service.rs`)
