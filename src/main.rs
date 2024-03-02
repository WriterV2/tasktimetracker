use sqlx::sqlite::SqlitePool;
use std::env;

trait CommonRecord: std::fmt::Debug {
    async fn db_add(&self, pool: &SqlitePool) -> anyhow::Result<()>;
    async fn from_id(id: i64, pool: &SqlitePool) -> anyhow::Result<Option<Self>>
    where
        Self: Sized;
    async fn all(pool: &SqlitePool) -> anyhow::Result<Vec<Self>>
    where
        Self: Sized;

    async fn db_update(&self, pool: &SqlitePool) -> anyhow::Result<()>;
    async fn db_delete(&self, pool: &SqlitePool) -> anyhow::Result<()>;
}

#[derive(Debug, Default)]
struct Importance {
    id: i64,
    name: String,
    val: i64,
}

impl CommonRecord for Importance {
    async fn db_add(&self, pool: &SqlitePool) -> anyhow::Result<()> {
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
        let importance = sqlx::query_as!(Self, "SELECT * FROM importance WHERE id = $1", id)
            .fetch_optional(pool)
            .await?;

        Ok(importance)
    }

    async fn all(pool: &SqlitePool) -> anyhow::Result<Vec<Self>> {
        let vec = sqlx::query_as!(Self, "SELECT * FROM importance")
            .fetch_all(pool)
            .await?;
        Ok(vec)
    }

    async fn db_delete(&self, pool: &SqlitePool) -> anyhow::Result<()> {
        sqlx::query!("DELETE FROM importance WHERE id = $1", self.id)
            .execute(pool)
            .await?;
        Ok(())
    }

    async fn db_update(&self, pool: &SqlitePool) -> anyhow::Result<()> {
        sqlx::query!(
            "UPDATE importance SET name = $1, val = $2 WHERE id = $3",
            self.name,
            self.val,
            self.id
        )
        .execute(pool)
        .await?;
        Ok(())
    }
}

impl Importance {
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

    fn set_name(mut self, name: &str) -> Self {
        self.name = String::from(name);
        self
    }

    fn set_value(mut self, value: i64) -> Self {
        self.val = value;
        self
    }
}

#[derive(Debug, Default)]
struct Tag {
    id: i64,
    name: String,
}

impl CommonRecord for Tag {
    async fn db_add(&self, pool: &SqlitePool) -> anyhow::Result<()> {
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

    async fn all(pool: &SqlitePool) -> anyhow::Result<Vec<Self>> {
        let vec = sqlx::query_as!(Self, "SELECT * FROM tag")
            .fetch_all(pool)
            .await?;
        Ok(vec)
    }

    async fn db_update(&self, pool: &SqlitePool) -> anyhow::Result<()> {
        sqlx::query!("UPDATE tag SET name = $1 WHERE id = $2", self.name, self.id)
            .execute(pool)
            .await?;

        Ok(())
    }
    async fn db_delete(&self, pool: &SqlitePool) -> anyhow::Result<()> {
        sqlx::query!("DELETE FROM tag WHERE id = $1", self.id)
            .execute(pool)
            .await?;
        Ok(())
    }
}

impl Tag {
    async fn from_name(name: &str, pool: &SqlitePool) -> anyhow::Result<Option<Self>> {
        let tag = sqlx::query_as!(Self, "SELECT * FROM tag WHERE name = $1", name)
            .fetch_optional(pool)
            .await?;

        Ok(tag)
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
