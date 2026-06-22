// src/migration/mod.rs — SeaORM migrations, kept in-crate (no separate crate).
//
// Each migration is a struct implementing `MigrationTrait` with `up()`/`down()`.
// We write the DDL as plain SQL via `execute_unprepared` — clearer than the
// builder DSL for CREATE TABLE, and easy to read in code review. SeaORM still
// tracks which migrations have run in the `seaql_migrations` table, so
// `Migrator::up(&db, None)` is a no-op once everything is applied.

pub use sea_orm_migration::prelude::*;

mod m20250101000001_create_users;
mod m20250101000002_create_posts;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250101000001_create_users::Migration),
            Box::new(m20250101000002_create_posts::Migration),
        ]
    }
}