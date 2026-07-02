// 06_custom_queries — When the fluent builder isn't enough: select_only +
// aggregation, or drop to raw SQL.
//
// `select_only()` opts out of SELECT *; `.column()` / `.column_as()` pick
// exact columns and aliases; `.group_by()` / `.having()` add aggregation;
// `.into_model::<T>()` deserialises into any struct that derives
// `FromQueryResult`. For the remaining 10%, `Statement::from_sql_and_values`
// is the raw-SQL escape hatch — still parameterised ($1, $2), never string
// interpolation.
//
// Needs DATABASE_URL + tables. Run with:
//   cargo run --bin 06_custom_queries

use sea_orm::{
    ActiveModelTrait, ColumnTrait, DbBackend, DbErr, EntityTrait, FromQueryResult, JoinType,
    QuerySelect, RelationTrait, Set, Statement,
};
use sea_orm::sea_query::ExprTrait;

use seaorm_basics::{entities::{post, user}, reset, setup};

#[derive(Debug, FromQueryResult)]
struct UserSummary {
    id: i32,
    username: String,
    post_count: i64,
}

#[tokio::main]
async fn main() -> Result<(), DbErr> {
    let db = setup().await?;
    reset(&db).await?;

    let neo = user::ActiveModel {
        email: Set("neo@matrix.com".into()),
        username: Set("neo".into()),
        password_hash: Set("h".into()),
        ..Default::default()
    }
    .insert(&db)
    .await?;
    let trin = user::ActiveModel {
        email: Set("trinity@matrix.com".into()),
        username: Set("trinity".into()),
        password_hash: Set("h".into()),
        ..Default::default()
    }
    .insert(&db)
    .await?;
    let _oracle = user::ActiveModel {
        email: Set("oracle@matrix.com".into()),
        username: Set("oracle".into()),
        password_hash: Set("h".into()),
        ..Default::default()
    }
    .insert(&db)
    .await?;

    for (uid, title) in [
        (neo.id, "Red Pill Guide"),
        (neo.id, "Matrix Survival"),
        (trin.id, "Hacking 101"),
    ] {
        post::ActiveModel {
            user_id: Set(uid),
            title: Set(title.into()),
            body: Set("content".into()),
            published: Set(true),
            ..Default::default()
        }
        .insert(&db)
        .await?;
    }

    // Aggregation via the fluent builder: per-user post count, having > 0.
    let summaries: Vec<UserSummary> = user::Entity::find()
        .select_only()
        .column(user::Column::Id)
        .column(user::Column::Username)
        .column_as(post::Column::Id.count(), "post_count")
        .join(JoinType::LeftJoin, user::Relation::Post.def())
        .group_by(user::Column::Id)
        .group_by(user::Column::Username)
        .having(post::Column::Id.count().gt(0))
        .into_model::<UserSummary>()
        .all(&db)
        .await?;
    println!("Fluent aggregation (users with ≥1 post):");
    for s in &summaries {
        println!("   {} has {} posts", s.username, s.post_count);
    }

    // Raw SQL escape hatch — same result, parameterised.
    let raw: Vec<UserSummary> = UserSummary::find_by_statement(Statement::from_sql_and_values(
        DbBackend::Postgres,
        r#"SELECT u.id, u.username, COUNT(p.id) AS post_count
           FROM users u
           LEFT JOIN posts p ON p.user_id = u.id
           WHERE u.is_active = $1
           GROUP BY u.id, u.username
           HAVING COUNT(p.id) > $2
           ORDER BY post_count DESC"#,
        [true.into(), 0i64.into()],
    ))
    .all(&db)
    .await?;
    println!("\nRaw SQL (same shape): {} row(s)", raw.len());
    for s in &raw {
        println!("   {} has {} posts", s.username, s.post_count);
    }
    Ok(())
}