// 02_entities_crud — Model / ActiveModel / Entity: the CRUD trinity.
//
// Model       — read-only, what queries return (a DB row).
// ActiveModel — write shape; each field is Set(x) or NotSet (for INSERT/UPDATE).
// Entity      — query builder entry point (Entity::find, Entity::find_by_id, …).
//
// Needs DATABASE_URL + tables (setup() creates them). Run with:
//   cargo run --bin 02_entities_crud

use sea_orm::{ActiveModelTrait, ColumnTrait, DbErr, EntityTrait, ModelTrait, PaginatorTrait, QueryFilter, QuerySelect, Set, sqlx::types::Decimal};

use seaorm_basics::{entities::user, reset, setup};

#[tokio::main]
async fn main() -> Result<(), DbErr> {
    let db = setup().await?;
    reset(&db).await?;

    // CREATE — ActiveModel with Set(...). id/created_at stay NotSet (DB-generated).
    let new_user = user::ActiveModel {
        email: Set("neo@matrix.com".into()),
        username: Set("neo".into()),
        password_hash: Set("hashed".into()),
        is_active: Set(true),
        ..Default::default()
    };
    let created = new_user.insert(&db).await?;
    println!("Created: id={} username={}", created.id, created.username);

    // let res = created.delete(&db).await?;

    // READ by primary key
    let found = user::Entity::find_by_id(created.id).one(&db).await?;
    println!("find_by_id → present? {}", found.is_some());

    // READ by a type-safe column expression (compiler knows Email is String)
    let by_email = user::Entity::find()
        .filter(user::Column::Email.eq("neo@matrix.com"))
        .one(&db)
        .await?;
    println!("filter(email=…) → present? {}", by_email.is_some());

    // let active = user::Entity::find()
    //     .filter(user::Column::IsActive.eq(true))
    //     .column_as(user::Column::Amount.sum(), "total_amount")
    //     .into_tuple::<Option<Decimal>>()
    //     .one(&db)
    //     .await?;

    // SELECT SUM(amount) AS "total_amount" FROM users WHERE is_active = TRUE;

    // println!("count(is_active=true) = {active}");

    // UPDATE — fetch the row, convert into an ActiveModel, Set only the field(s)
    // to change, then update(). SeaORM emits UPDATE … SET username=$1 WHERE id=$2.
    let mut am: user::ActiveModel = found.unwrap().into();
    am.username = Set("thomas".into());
    let updated = am.update(&db).await?;
    println!("Updated username → {}", updated.username);

    // DELETE — by id; exec returns rows_affected.
    let res = user::Entity::delete_by_id(updated.id).exec(&db).await?;
    println!("Deleted rows_affected = {}", res.rows_affected);

    Ok(())
}