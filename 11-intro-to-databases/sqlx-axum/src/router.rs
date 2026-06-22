// src/router.rs — assemble the application. (PARTIALLY DONE.)
//
// `/health` is wired and works today. The `/users` sub-router is commented
// out because its handlers are TODO stubs — uncomment it in `build_app` once
// you've implemented `src/routes/users.rs` and `src/db/users.rs`.

use axum::{routing::get, Router};
use tower_http::trace::TraceLayer;

use crate::routes;
use crate::state::AppState;

pub fn build_app(state: AppState) -> Router {
    // TODO(you): once routes::users handlers compile, uncomment this block:
    //
    // let users: Router<AppState> = Router::new()
    //     .route("/", get(routes::users::list).post(routes::users::create))
    //     .route(
    //         "/:id",
    //         get(routes::users::get)
    //             .put(routes::users::update)
    //             .delete(routes::users::delete),
    //     );

    Router::new()
        .route("/health", get(routes::health))
        // .nest("/users", users)
        .layer(TraceLayer::new_for_http()) // request/status/latency logs
        .with_state(state)
}