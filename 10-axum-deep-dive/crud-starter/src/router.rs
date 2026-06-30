// src/router.rs — assembly of the CRUD app.

use axum::{
    http::{header, Method},
    middleware,
    routing::post,
    Router,
};
use tower_http::{
    cors::CorsLayer,
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    trace::TraceLayer,
};

use crate::{
    middleware::{
        auth::jwt_middleware,
        rate_limit::{rate_limit_middleware, RateLimiterState},
        request_id::request_id_middleware,
        tower_service::TimingLayer,
    },
    routes,
    state::AppState,
};

pub fn build_app() -> Router {
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
                .layer(middleware::from_fn(jwt_middleware))
                .layer(
                    middleware::from_fn_with_state(
                        rate_state,
                        rate_limit_middleware,
                    ),
                ),
        )
        .route("/", routes::users::list_route());

    let health: Router<AppState> = Router::new()
        .route(
            "/health",
            axum::routing::get(routes::health::health),
        );

    let auth: Router<AppState> = Router::new()
        .route(
            "/login",
            post(routes::auth::login),
        );

    let v1: Router<AppState> = Router::new()
        .merge(health)
        .merge(auth)
        .nest("/users", users);

    let v1: Router = v1.with_state(AppState::new());

    let x_request_id = "x-request-id";

    Router::new()
        .nest("/api/v1", v1)
        .layer(
            PropagateRequestIdLayer::new(
                x_request_id.parse().unwrap(),
            ),
        )
        .layer(
            SetRequestIdLayer::new(
                x_request_id.parse().unwrap(),
                MakeRequestUuid,
            ),
        )
        .layer(middleware::from_fn(request_id_middleware))
        .layer(cors_layer())
        .layer(TraceLayer::new_for_http())
}

fn cors_layer() -> CorsLayer {
    CorsLayer::new()
        .allow_origin([
            "http://localhost:3000".parse().unwrap(),
        ])
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([
            header::AUTHORIZATION,
            header::CONTENT_TYPE,
        ])
        .max_age(std::time::Duration::from_secs(3600))
}