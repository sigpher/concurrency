use sqlx::{SqlitePool, error::Error, prelude::FromRow, sqlite::SqliteConnectOptions};
use std::env;
use std::{fs, str::FromStr, time::Duration};

#[derive(Debug, FromRow)]
struct User {
    // id: i64,
    name: String,
    age: Option<i32>,
    email: String,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenvy::dotenv().unwrap();

    let db = env::var("DATABASE_URL").expect("database should be set");

    let options = SqliteConnectOptions::from_str(&db)?
        .create_if_missing(true)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
        .synchronous(sqlx::sqlite::SqliteSynchronous::Normal)
        .busy_timeout(Duration::from_secs(5));
    let pool = SqlitePool::connect_with(options).await?;

    let create_db_sql = fs::read_to_string("sql/create_user.sql").unwrap();
    sqlx::query(&create_db_sql).execute(&pool).await?;

    batch_insert_users(&pool).await?;

    Ok(())
}

async fn batch_insert_users(pool: &SqlitePool) -> Result<(), Error> {
    let mut tx = pool.begin().await?;

    let users = vec![
        User {
            // id: 1,
            name: "choi".into(),
            age: Some(30),
            email: "cwy@tdyh.com.cn".into(),
        },
        User {
            // id: 2,
            name: "lora".into(),
            age: Some(26),
            email: "zdq@tdyh.com.cn".into(),
        },
        User {
            // id: 3,
            name: "troy".into(),
            age: Some(6),
            email: "cly@tdyh.com.cn".into(),
        },
        User {
            // id: 4,
            name: "elan".into(),
            age: Some(2),
            email: "cyl@tdyh.com.cn".into(),
        },
    ];

    for user in users.iter() {
        sqlx::query("INSERT OR IGNORE INTO users (name, age, email) VALUES(?, ?, ?)")
            .bind(&user.name)
            .bind(&user.age)
            .bind(&user.email)
            .execute(&mut *tx)
            .await?;
    }

    tx.commit().await?;
    sqlx::query("VACUUM;").execute(pool).await?;

    Ok(())
}
