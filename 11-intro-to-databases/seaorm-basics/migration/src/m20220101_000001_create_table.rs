use sea_orm_migration::{prelude::*, schema::*, sea_orm::Statement};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        let db_con = manager.get_connection();

        db_con.execute_unprepared(
        r#"CREATE TABLE IF NOT EXISTS users (
            id            SERIAL PRIMARY KEY,
            email         VARCHAR(255) NOT NULL UNIQUE,
            username      VARCHAR(255) NOT NULL,
            password_hash VARCHAR(255) NOT NULL,
            is_active     BOOLEAN NOT NULL DEFAULT true,
            created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )"#).await?;

        db_con.execute_unprepared(
        r#"CREATE TABLE IF NOT EXISTS posts (
            id        SERIAL PRIMARY KEY,
            user_id   INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            title     VARCHAR(255) NOT NULL,
            body      TEXT NOT NULL,
            published BOOLEAN NOT NULL DEFAULT false
        )"#)
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db_con = manager.get_connection();

        db_con.execute_unprepared("DELETE FROM posts").await?;
        db_con.execute_unprepared("DELETE FROM users").await?;

        Ok(())

    }
}

#[derive(DeriveIden)]
enum Post {
    Table,
    Id,
    Title,
    Text,
}
