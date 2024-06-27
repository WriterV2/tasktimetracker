use anyhow::Context;
use sqlx::sqlite::SqlitePoolOptions;
use std::env;

use tasktimetracker::{self, records};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // let pool = SqlitePool::connect(&env::var("DATABASE_URL")?).await?;
    let pool = SqlitePoolOptions::new()
        .connect(&env::var("DATABASE_URL")?)
        .await
        .context("Failed to connect to database")?;

    sqlx::migrate!().run(&pool).await?;
    records::serve(pool).await?;

    Ok(())
}
