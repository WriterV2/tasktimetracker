use sqlx::sqlite::SqlitePool;
use std::env;

#[derive(Debug)]
struct Tag {
    id: i64,
    name: String,
}

impl Tag {
    async fn add(&self, pool: &SqlitePool) -> anyhow::Result<()> {
        sqlx::query!("INSERT INTO tag (name) VALUES ( $1 )", self.name)
            .execute(pool)
            .await?;

        Ok(())
    }

    async fn from_id(id: i64, pool: &SqlitePool) -> anyhow::Result<Option<Self>> {
        let tag = sqlx::query_as!(Tag, "SELECT * FROM tag WHERE id = $1", id)
            .fetch_optional(pool)
            .await?;

        Ok(tag)
    }

    async fn from_name(name: &str, pool: &SqlitePool) -> anyhow::Result<Option<Self>> {
        let tag = sqlx::query_as!(Tag, "SELECT * FROM tag WHERE name = $1", name)
            .fetch_optional(pool)
            .await?;

        Ok(tag)
    }

    async fn update_name(&self, pool: &SqlitePool) -> anyhow::Result<()> {
        sqlx::query!("UPDATE tag SET name = $1 WHERE id = $2", self.name, self.id)
            .execute(pool)
            .await?;
        Ok(())
    }

    async fn delete(&self, pool: &SqlitePool) -> anyhow::Result<()> {
        sqlx::query!("DELETE FROM tag WHERE id = $1", self.id)
            .execute(pool)
            .await?;
        Ok(())
    }

    fn set_name(mut self, name: &str) -> Self {
        self.name = String::from(name);
        self
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let pool = SqlitePool::connect(&env::var("DATABASE_URL")?).await?;
    Ok(())
}
