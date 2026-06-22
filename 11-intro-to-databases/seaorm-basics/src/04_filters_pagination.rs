// 04_filters_pagination — Conditions, ordering, and paginator.
//
// Chained `.filter()` calls are AND. `Condition::any()` is OR, `all()` is AND,
// and they nest. Every operator is typed: comparing `IsActive` (bool) to a
// string simply won't compile. `.paginate(db, page_size)` gives a Paginator
// with `num_pages()` (runs COUNT) and `fetch_page(i)` (0-indexed).
//
// Needs DATABASE_URL + tables. Run with:
//   cargo run --bin 04_filters_pagination

use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, DbErr, EntityTrait, Order, PaginatorTrait,
    QueryFilter, QueryOrder, Set,
};

use seaorm_basics::{entities::user, reset, setup};

#[tokio::main]
async fn main() -> Result<(), DbErr> {
    let db = setup().await?;
    reset(&db).await?;

    let seed = [
        ("admin_neo", "neo@a.com"),
        ("admin_trin", "trin@b.com"),
        ("root_smith", "smith@c.com"),
        ("user_morph", "morph@d.com"),
    ];
    for (username, email) in seed {
        let active = !username.starts_with("user_");
        user::ActiveModel {
            email: Set(email.into()),
            username: Set(username.into()),
            password_hash: Set("h".into()),
            is_active: Set(active),
            ..Default::default()
        }
        .insert(&db)
        .await?;
    }

    // OR via Condition::any()
    let some = user::Entity::find()
        .filter(
            Condition::any() // OR
                .add(user::Column::Email.contains("@a.com"))
                .add(user::Column::Email.contains("@c.com")),
        )
        .all(&db)
        .await?;
    println!("email @a.com or @c.com → {} user(s)", some.len());

    // Combined AND + nested OR, then order newest-first
    let active_admins = user::Entity::find()
        .filter(
            Condition::all() // AND
                .add(user::Column::IsActive.eq(true))
                .add(
                    Condition::any()
                        .add(user::Column::Username.starts_with("admin"))
                        .add(user::Column::Username.starts_with("root")),
                ),
        )
        .order_by(user::Column::CreatedAt, Order::Desc)
        .all(&db)
        .await?;
    println!("active admin/root users:");
    for u in &active_admins {
        println!("   - {}", u.username);
    }

    // Pagination — 2 per page

    // PAGE NUMBER => CLIENT SIDE
    // LIMIT => CLIENT // DEFAULT

    // LIMIT => 10
    // OFFSET => 23


    // 1 ... 100

    // IF PAGE starting from 1
    // start position = (PAGE_NUMBER-1) * LIMIT + 1
    // end position = PAGE_NUMBER * LIMIT

    // PAGE 1 => 

    let paginator = user::Entity::find()
        .order_by(user::Column::CreatedAt, Order::Desc)
        .paginate(&db, 2);
    let total_pages: u64 = paginator.num_pages().await?;
    println!("\ntotal_pages (page_size 2) = {total_pages}");
    for p in 0..total_pages {
        let page: Vec<user::Model> = paginator.fetch_page(p).await?;
        let names: Vec<&str> = page.iter().map(|u| u.username.as_str()).collect();
        println!("page {p}: {names:?}");
    }
    Ok(())
}