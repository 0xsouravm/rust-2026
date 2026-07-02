// 03_relations — has_many / belongs_to, and loading related rows.
//
// user.rs declares `has_many = "post::Entity"`; post.rs declares `belongs_to`
// from `Column::UserId` to `user::Column::Id`. With that, `find_with_related`
// loads each user together with their posts — using exactly TWO queries
// (users, then posts via an IN clause), not N+1.
//
// Needs DATABASE_URL + tables. Run with:
//   cargo run --bin 03_relations

use sea_orm::{ActiveModelTrait, DbErr, EntityTrait, ModelTrait, Set};

use seaorm_basics::{entities::{post, user}, reset, setup};

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

    for title in ["Red Pill Guide", "Matrix Survival"] {
        post::ActiveModel {
            user_id: Set(neo.id),
            title: Set(title.into()),
            body: Set("content".into()),
            published: Set(true),
            ..Default::default()
        }
        .insert(&db)
        .await?;
    }

    // Oracle has no posts — appears in find_with_related with an empty Vec.
    let _oracle = user::ActiveModel {
        email: Set("oracle@matrix.com".into()),
        username: Set("oracle".into()),
        password_hash: Set("h".into()),
        ..Default::default()
    }
    .insert(&db)
    .await?;

    // find_with_related → Vec<(user::Model, Vec<post::Model>)>. Two queries.
    let rows: Vec<(user::Model, Vec<post::Model>)> =
        user::Entity::find().find_with_related(post::Entity).all(&db).await?;
    for (u, posts) in &rows {
        println!("{} has {} posts", u.username, posts.len());
        for p in posts {
            println!("   - {}", p.title);
        }
    }

    // 1 User ID - #10
    // 100 Posts + 1 User 

    // For Users with name starting from "Rohan" get all the posts
    // 1st Query => SELECT * FROM users WHERE username ILIKE "%rohan%"; -> N Users
    // for user in result    
        // Nth Queries => SELECT * FROM posts WHERE user_id = neo.id 


    // Lazily load related rows from an existing Model.
    let neo_loaded = user::Entity::find_by_id(neo.id).one(&db).await?.unwrap();
    let neo_posts: Vec<post::Model> = neo_loaded.find_related(post::Entity).all(&db).await?;
    println!("\nneo.find_related → {} posts", neo_posts.len());

    Ok(())
}