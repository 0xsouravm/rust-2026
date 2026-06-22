// src/db/mod.rs — the data-access layer (a.k.a. repository).
//
// Keeping all SQL in `db/` (not in handlers) is what makes handlers testable
// and keeps the HTTP layer ignorant of SQL. Your repository functions go in
// `src/db/users.rs` — open it for the TODO.

pub mod users;