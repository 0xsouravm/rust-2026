// Create the posts table with a FK to users (CASCADE on delete).

use sea_orm::ConnectionTrait;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.get_connection().execute_unprepared(
            r#"CREATE TABLE IF NOT EXISTS posts (
                id        SERIAL PRIMARY KEY,
                user_id   INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                title     VARCHAR(255) NOT NULL,
                body      TEXT NOT NULL,
                published BOOLEAN NOT NULL DEFAULT false,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )"#,
        ).await?;
        manager
            .get_connection()
            .execute_unprepared("CREATE INDEX IF NOT EXISTS idx_posts_user_id ON posts(user_id)")
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared("DROP TABLE IF EXISTS posts")
            .await?;
        Ok(())
    }
}