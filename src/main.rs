use sqlx::sqlite::SqlitePool;
use std::env;

#[derive(Debug)]
struct Importance {
    id: i64,
    name: String,
    val: i64,
}

impl Importance {
    async fn add(&self, pool: &SqlitePool) -> anyhow::Result<()> {
        sqlx::query!(
            "INSERT INTO importance (name, val) VALUES ($1, $2)",
            self.name,
            self.val
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    async fn from_id(id: i64, pool: &SqlitePool) -> anyhow::Result<Option<Self>> {
        let importance = sqlx::query_as!(Importance, "SELECT * FROM importance WHERE id = $1", id)
            .fetch_optional(pool)
            .await?;

        Ok(importance)
    }

    async fn from_name(name: &str, pool: &SqlitePool) -> anyhow::Result<Option<Self>> {
        let importance = sqlx::query_as!(Self, "SELECT * FROM importance WHERE name = $1", name)
            .fetch_optional(pool)
            .await?;

        Ok(importance)
    }

    async fn from_value(val: i64, pool: &SqlitePool) -> anyhow::Result<Option<Self>> {
        let importance = sqlx::query_as!(Self, "SELECT * FROM importance WHERE val = $1", val)
            .fetch_optional(pool)
            .await?;

        Ok(importance)
    }

    async fn update_name(&self, pool: &SqlitePool) -> anyhow::Result<()> {
        sqlx::query!(
            "UPDATE importance SET name = $1 WHERE id = $2",
            self.name,
            self.id
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    async fn update_value(&self, pool: &SqlitePool) -> anyhow::Result<()> {
        sqlx::query!(
            "UPDATE importance SET val = $1 WHERE id = $2",
            self.val,
            self.id
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    async fn delete(&self, pool: &SqlitePool) -> anyhow::Result<()> {
        sqlx::query!("DELETE FROM importance WHERE id = $1", self.id)
            .execute(pool)
            .await?;
        Ok(())
    }

    fn set_name(mut self, name: &str) -> Self {
        self.name = String::from(name);
        self
    }

    fn set_value(mut self, value: i64) -> Self {
        self.val = value;
        self
    }
}

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
        let tag = sqlx::query_as!(Self, "SELECT * FROM tag WHERE id = $1", id)
            .fetch_optional(pool)
            .await?;

        Ok(tag)
    }

    async fn from_name(name: &str, pool: &SqlitePool) -> anyhow::Result<Option<Self>> {
        let tag = sqlx::query_as!(Self, "SELECT * FROM tag WHERE name = $1", name)
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
