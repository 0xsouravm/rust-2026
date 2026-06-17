// src/middleware/mod.rs — re-exports of every middleware module.
//
// Order in this file is **read-order**, not execution order. The
// execution order is determined by where you call `.layer()` in
// `src/router.rs`. That's the most common point of confusion.

pub mod auth;
pub mod logger;
pub mod rate_limit;
pub mod request_id;
pub mod timing;
pub mod tower_service;
