// In-memory User CRUD

// Create inside /users prefix
// 1. List USers
// 2. Create User -> POST
// 3. Get User by ID -> GET
// 4. Update User by ID -> PUT
// 5. Delete User by ID -> DELETE
// 6. Patch User by ID -> PATCH

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, RwLock},
};
use tokio::net::TcpListener;

// AppError Enum
// error.rs
#[derive(Debug)]
enum AppError {
    NotFound(String),
    BadRequest(String),
    Internal(String)
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::NotFound(err) => (StatusCode::NOT_FOUND, err),
            AppError::BadRequest(err) => (StatusCode::BAD_REQUEST, err),
            AppError::Internal(err) => (StatusCode::INTERNAL_SERVER_ERROR, err)
        };

        (status, Json(json!(
            {
                "error": message
            }
        ))).into_response()
    }
}


// APp State and User Structs
#[derive(Serialize, Clone)]
struct User {
    id: u64,
    name: String,
    email: String
}

// state.rs
#[derive(Clone)]
struct AppState {
    db: Arc<RwLock<HashMap<u64, User>>>,
    next_id: Arc<RwLock<u64>>
}

impl AppState {
    fn new() -> Self {
        let mut db = HashMap::new();
        db.insert(1, User {
            id: 1,
            name: "Roshan".into(),
            email: "roshan@gmail.com".into() 
        });
        db.insert(2, User {
            id: 2,
            name: "Sachidananda".into(),
            email: "sd@gmail.com".into() 
        });

        Self {
            db: Arc::new(RwLock::new(db)),
            next_id: Arc::new(RwLock::new(3))
        }
    }

    fn increase_id(&self) -> u64 {
        let mut next = self.next_id.write().unwrap();
        let id = *next;
        *next += 1;
        id
    }

}

// BASE_URL + /health
async fn health() -> Json<Value> {
    Json(
        json!(
            {
                "status": "ok",
                "uptime": 200
            }
        )
    )
}

#[derive(Deserialize)]
struct NewUser {
    name: String,
    email: String
}

#[derive(Deserialize)]
struct UpdateUser {
    name: Option<String>,
    email: Option<String>
}


#[derive(Deserialize)]
struct ListQuery {
    search: Option<String>,
    limit: Option<usize>
}

// Handler Functions controllers.
async fn list_users(
    State(state): State<AppState>,
    Query(params): Query<ListQuery>
) -> Json<Vec<User>> {
    let db = state.db.read().unwrap();
    let mut users: Vec<User> = db.values().cloned().collect();

    if let Some(q) = params.search.as_deref().map(str::to_lowercase) {
        users.retain(|u| u.name.to_lowercase().contains(&q));
    }

    if let Some(limit) = params.limit {
        users.truncate(limit);
    }

    Json(users)
}

// POST /users
async fn create_user(
    State(state): State<AppState>,
    Json(payload): Json<NewUser>
) -> Result<(StatusCode, Json<User>), AppError> {
    let name = payload.name.clone().trim().to_string();
    let email = payload.email.clone().trim().to_string();

    let id = state.increase_id();
    let user = User { id, name, email };

    state.db.write().unwrap().insert(id, user.clone());

    Ok((StatusCode::CREATED, Json(user)))

}

// GET /users/{id}
async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<u64>
) -> Result<Json<User>, AppError> {
    let db = state.db.read().unwrap();
    
    db.get(&id)
        .cloned()
        .map(Json)
        .ok_or_else(|| AppError::NotFound(format!("user {id} not found")))
    
    // Equivalent:
    // let user_data = db.get(&id).cloned();
    // match user_data {
    //     Some(data) => {
    //         Ok(Json(data))
    //     },
    //     None => {
    //         Err(AppError::NotFound(format!("user {id} not found")))
    //     }
    // }
}

// FULL REPLACE
async fn update_user(
    State(state): State<AppState>,
    Path(id): Path<u64>,
    Json(payload): Json<NewUser>
) -> Result<Json<User>, AppError> {
    let name = payload.name.clone().trim().to_string();
    let email = payload.email.clone().trim().to_string();

    let mut db = state.db.write().unwrap();
    let user = db.get_mut(&id).ok_or_else(
        || AppError::NotFound(format!("user {id} not found")))?;

    user.name = name;
    user.email = email;

    Ok(Json(user.clone()))

    // Equivalent:
    // let user = db.get_mut(&id).ok_or_else(|| AppError::NotFound(format!("user {id} not found")));
    // match user {
    //     Ok(data) => {
    //         data.name = name;
    //         data.email = email;
    //         Ok(Json(data.clone()))
    //     },
    //     _ => {
    //         Err(AppError::NotFound(format!("user {id} not found")))
    //     }
    // }
}

// PARTIAL UPDATE
async fn patch_user(
    State(state): State<AppState>,
    Path(id): Path<u64>,
    Json(payload): Json<UpdateUser>
) -> Result<Json<User>, AppError> {
    let name = payload.name.clone().unwrap_or("".to_string()).trim().to_string();
    let email = payload.email.clone().unwrap_or("".to_string()).trim().to_string();

    let mut db = state.db.write().unwrap();
    let user = db.get_mut(&id).ok_or_else(
        || AppError::NotFound(format!("user {id} not found")))?;

    if !name.is_empty() { 
        user.name = name 
    };
    
    if !email.is_empty() { 
        user.email = email 
    };

    Ok(Json(user.clone()))

    // Equivalent:
    //     let user = db.get_mut(&id).ok_or_else(
    //    || AppError::NotFound(format!("user {id} not found")));
    // match user {
    //     Ok(data) => {
    //         if !name.is_empty() {
    //             data.name = name;
    //         }
    //         if !email.is_empty() {
    //             data.email = email;
    //         }
    //         Ok(Json(data.clone()))
    //     },
    //     _ => {
    //         Err(AppError::NotFound(format!("user {id} not found")))
    //     }
    // }

}

async fn delete_user(
    State(state): State<AppState>,
    Path(id): Path<u64>
) -> Result<StatusCode, AppError> {
    let removed_user = state.db.write().unwrap().remove(&id);
    match removed_user {
        Some(_) => Ok(StatusCode::NO_CONTENT),
        None => Err(AppError::NotFound(format!("user {id} not found")))
    }
}

// Routers router.rs

fn users_router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_users).post(create_user))
       .route("/{id}", 
get(get_user)
            .put(update_user)
            .patch(patch_user)
            .delete(delete_user)
        )
}

fn build_app() -> Router {
    Router::new()
        .route("/health", get(health))
        .nest("/users", users_router())
        .with_state(AppState::new())
}


// main.rs
#[tokio::main]
async fn main() {
    let addr: SocketAddr = "127.0.0.1:3005".parse().unwrap();
    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, build_app()).await.unwrap();
}
