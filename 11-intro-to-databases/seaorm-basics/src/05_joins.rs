// 05_joins — Loading related data without writing JOIN SQL.
//
// `find_with_related`  — many related rows per parent  → Vec<(Model, Vec<Related>)>
// `find_also_related`  — one related row per parent   → Vec<(Model, Option<Related>)>
// `.join(JoinType, Relation::X.def())` — drop down toward SQL when you need to
// filter on the *joined* table (e.g. "users who wrote a post about Rust").
//
// Needs DATABASE_URL + tables. Run with:
//   cargo run --bin 05_joins

use sea_orm::{
    ActiveModelTrait, ColumnTrait, DbErr, EntityTrait, JoinType, QueryFilter, QuerySelect,
    RelationTrait, Set,
};

use seaorm_basics::{entities::{post, user}, reset, setup};

#[tokio::main]
async fn main() -> Result<(), DbErr> {
    let db = setup().await?;
    reset(&db).await?;

    let neo: user::Model = user::ActiveModel {
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
        (trin.id, "Rust Guide"),
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

    // find_also_related — for each user, the first matching post (Option).
    let rows: Vec<(user::Model, Option<post::Model>)> =
        user::Entity::find().find_also_related(post::Entity).all(&db).await?;
    println!("find_also_related:");
    for (u, p) in &rows {
        println!("   {} → {:?}", u.username, p.as_ref().map(|p| p.title.as_str()));
    }

    // .join() + filter on the related table: users who wrote a "Rust" post.
    let authors = user::Entity::find()
        .join(JoinType::InnerJoin, user::Relation::Post.def())
        .filter(post::Column::Title.contains("Rust"))
        .all(&db)
        .await?;
    println!("\nauthors with a 'Rust' post: {}", authors.len());
    for u in &authors {
        println!("   - {}", u.username);
    }
    Ok(())
}