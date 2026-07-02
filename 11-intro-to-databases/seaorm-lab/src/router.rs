// src/router.rs — assemble the application.

use axum::{routing::get, Router};
use tower_http::trace::TraceLayer;

use crate::{handlers, state::AppState};

pub fn build_app(state: AppState) -> Router {
    let users: Router<AppState> = Router::new()
        .route("/", get(handlers::users::list).post(handlers::users::create))
        .route(
            "/{id}",
            get(handlers::users::get)
                .put(handlers::users::update)
                .delete(handlers::users::delete),
        )
        .route("/{id}/posts", get(handlers::users::user_posts));

    Router::new()
        .nest("/users", users)
        .route("/health", get(handlers::health))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}