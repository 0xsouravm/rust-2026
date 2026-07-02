// 07_mock_tests — Unit-test the data layer WITHOUT a database, using
// MockDatabase.
//
// `append_query_results` pre-programs what SELECTs return (as real Model
// values). `append_exec_results` pre-programs INSERT/UPDATE/DELETE results.
// Calls are consumed in order. After running, `into_transaction_log()` gives
// you every SQL statement generated — assert on it to catch regressions.
//
// No DATABASE_URL needed — that's the whole point. Run with:
//   cargo run --bin 07_mock_tests

use sea_orm::{
    ActiveModelTrait, DatabaseBackend, EntityTrait, MockDatabase, MockExecResult, Set,
};

use seaorm_basics::entities::user;

fn sample_user(id: i32) -> user::Model {
    user::Model {
        id,
        email: "alice@example.com".into(),
        username: "alice".into(),
        password_hash: "hashed".into(),
        is_active: true,
        created_at: chrono::DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z").unwrap(),
    }
}

#[tokio::main]
async fn main() {
    // ── find_by_id returns the seeded row ──────────────────────────────────
    let db = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results(vec![vec![sample_user(1)]])
        .into_connection();

    let u = user::Entity::find_by_id(1)
        .one(&db)
        .await
        .expect("query should succeed");
    assert!(u.is_some(), "expected a user");
    let u = u.unwrap();
    assert_eq!(u.username, "alice");
    assert_eq!(u.email, "alice@example.com");

    let log = db.into_transaction_log();
    assert_eq!(log.len(), 1, "expected exactly one transaction");
    println!("find_by_id → returned alice, generated 1 statement ✅");

    // ── create returns the inserted model ────────────────────────────────
    let db = MockDatabase::new(DatabaseBackend::Postgres)
        // Postgres uses INSERT … RETURNING (a query), so it consumes a query result.
        .append_query_results(vec![vec![sample_user(42)]])
        // MySQL/SQLite would use an exec result — harmless to include for Postgres.
        .append_exec_results(vec![MockExecResult {
            last_insert_id: 42,
            rows_affected: 1,
        }])
        .into_connection();

    let new_user = user::ActiveModel {
        email: Set("alice@example.com".into()),
        username: Set("alice".into()),
        password_hash: Set("hash".into()),
        is_active: Set(true),
        ..Default::default()
    };
    let created = new_user
        .insert(&db)
        .await
        .expect("insert should succeed");
    assert_eq!(created.id, 42, "mock should return id 42");
    assert_eq!(created.email, "alice@example.com");
    println!("create   → mock returned id=42 ✅");

    println!("\nAll MockDatabase assertions passed ✅");
}