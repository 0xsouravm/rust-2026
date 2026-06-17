// src/models.rs — request inputs, internal storage, response outputs.

use serde::{Deserialize, Serialize};

#[derive(Serialize, Clone)]
pub struct User {
    pub id:    u64,
    pub name:  String,
    pub email: String,
}

#[derive(Deserialize)]
pub struct NewUser {
    pub name:  String,
    pub email: String,
}

#[derive(Deserialize)]
pub struct UpdateUser {
    pub name:  Option<String>,
    pub email: Option<String>,
}

#[derive(Deserialize)]
pub struct ListQuery {
    pub search: Option<String>,
    pub limit:  Option<usize>,
}
