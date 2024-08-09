use sqlx::sqlite::SqlitePoolOptions;
use std::env;

use tasktimetracker::{self, records};

#[tokio::main]
async fn main() {
    let pool = SqlitePoolOptions::new()
        .connect(
            &env::var("DATABASE_URL").expect("Failed to get environment variable DATABASE_URL"),
        )
        .await
        .expect("Failed to connect to database");

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to run migrations");
    records::serve(pool).await;
}
