// src/router.rs — assembly of the CRUD app.
//
//. Three things to point out:
//
//   1. CHAIN ORDER — layers are executed OUTER-FIRST. A `.layer()`
//      added later wraps everything added earlier.
//
//   2. SELECTIVE MIDDLEWARE — `timing` and `auth` are added to the
//      `users` sub-router ONLY, not to `/health`. That's why
//      /health is reachable without an API key.
//
//   3. NESTED ROUTER + STATE — the sub-router is `Router<AppState>`.
//      `.with_state(...)` consumes the state and returns a
//      `Router<()>`, which can be freely merged/nested above.
//
// Chain (outer → inner):
//
//   TraceLayer           (tower-http — method, path, status, latency)
//     CORS               (tower-http — answers preflight first)
//       request_id       (hand-rolled — X-Request-Id + log line)
//         ── nest /api/v1 ──
//           /users   (with timing + auth layers)
//             POST   (extra: rate_limit via from_fn_with_state)
//             GET/PATCH/DELETE
//           /health
//             GET    (no layers)

use axum::{
    http::{header, Method, StatusCode},
    middleware,
    routing::{get, post},
    Router,
    Json,
};
use tower_http::{
    cors::CorsLayer,
    trace::TraceLayer,
};

use crate::{
    middleware::{
        auth::{jwt_middleware, Claims},
        rate_limit::{rate_limit_middleware, RateLimiterState},
        tower_service::TimingLayer,
    },
    routes,
    state::AppState,
};

async fn login_handler(Json(body): Json<serde_json::Value>) -> Result<Json<serde_json::Value>, StatusCode> {
    let username = body.get("username").and_then(|v| v.as_str()).unwrap_or("anonymous");
    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "secure-local-dev-key".to_string());

    // calculate token expiration (current unix time + 3600 secs/1 hr)
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
    let claims = Claims { sub: username.to_string(), exp: (now + 3600) as usize, role: "user".to_string() };
    
    // cryptographically sign the token using the secret key
    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(secret.as_bytes()),
    ).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // send the token back to the user
    Ok(Json(serde_json::json!({ "token": token })))
}

pub fn build_app() -> Router {
    // ── /users sub-router ──────────────────────────────────────────────
    // Stack: timing → auth → rate_limit.  Rate limit fires only on POST
    // (see rate_limit_middleware's internal check).
    let rate_state = RateLimiterState::new();

    let users: Router<AppState> = Router::new()
        .route(
            "/{id}",
            routes::users::read_user_route()
                .layer(TimingLayer)
                .layer(middleware::from_fn(jwt_middleware)),
        )
        .route(
            "/",
            post(routes::users::create_user)
                .layer(TimingLayer)
                .layer(
                    middleware::from_fn_with_state(rate_state, rate_limit_middleware),
                )
                .layer(middleware::from_fn(jwt_middleware))
        )
        .route("/", routes::users::list_route());

    // ── /health sub-router ─────────────────────────────────────────────
    // No layers — public.
    let health: Router<AppState> = Router::new()
        .route("/health", get(routes::health::health));

    let public_auth: Router<AppState> = Router::new()
        .route("/login", post(login_handler));  // public endpoint to get a token

    // ── v1 composition ─────────────────────────────────────────────────
    let v1: Router<AppState> = Router::new()
        .merge(health)
        .merge(public_auth)
        .nest("/users", users);

    // Drop the state type so we can layer freely above.
    let v1: Router = v1.with_state(AppState::new());

    // ── Outer layers (last added = outermost) ──────────────────────────
    // request → TraceLayer → CORS → request_id → v1 → handler
    // let x_request_id = "x-request-id";

    Router::new()
        .nest("/api/v1", v1)
        .layer(cors_layer())
        .layer(TraceLayer::new_for_http())
        .layer(TimingLayer)
}

// ── CORS ──────────────────────────────────────────────────────────────
//
// Not a function. CORS in axum is a `tower_http::cors::CorsLayer`
// configured inline. It handles preflight `OPTIONS` requests before
// they reach your handlers, which is why it goes OUTERMOST (so a
// disallowed origin never even reaches auth).
fn cors_layer() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(["http://localhost:3000".parse().unwrap()])
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE])
        .max_age(std::time::Duration::from_secs(3600))
}
