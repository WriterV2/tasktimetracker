use sqlx::sqlite::SqlitePool;
use std::env;

trait CommonRecord: std::fmt::Debug {
    async fn db_add(self, pool: &SqlitePool) -> anyhow::Result<Self>
    where
        Self: Sized;

    async fn from_id(id: i64, pool: &SqlitePool) -> anyhow::Result<Option<Self>>
    where
        Self: Sized;
    async fn all(pool: &SqlitePool) -> anyhow::Result<Vec<Self>>
    where
        Self: Sized;

    async fn db_update(self, pool: &SqlitePool) -> anyhow::Result<Self>
    where
        Self: Sized;

    async fn db_delete(self, pool: &SqlitePool) -> anyhow::Result<Self>
    where
        Self: Sized;
}

#[derive(Debug, Default)]
struct Task {
    id: i64,
    name: String,
    des: String,
    done: bool,
    time: i64,
    iid: i64,
}

impl CommonRecord for Task {
    async fn db_add(mut self, pool: &SqlitePool) -> anyhow::Result<Self> {
        self.id = sqlx::query!(
            "INSERT INTO task (name, des, done, time, iid) VALUES ($1, $2, $3, $4, $5)",
            self.name,
            self.des,
            self.done,
            self.time,
            self.iid
        )
        .execute(pool)
        .await?
        .last_insert_rowid();

        Ok(self)
    }

    async fn from_id(id: i64, pool: &SqlitePool) -> anyhow::Result<Option<Self>> {
        let importance = sqlx::query_as!(Self, "SELECT * FROM task WHERE id = $1", id)
            .fetch_optional(pool)
            .await?;

        Ok(importance)
    }

    async fn all(pool: &SqlitePool) -> anyhow::Result<Vec<Self>> {
        let vec = sqlx::query_as!(Self, "SELECT * FROM task")
            .fetch_all(pool)
            .await?;
        Ok(vec)
    }

    async fn db_delete(self, pool: &SqlitePool) -> anyhow::Result<Self> {
        sqlx::query!("DELETE FROM task WHERE id = $1", self.id)
            .execute(pool)
            .await?;
        Ok(self)
    }

    async fn db_update(self, pool: &SqlitePool) -> anyhow::Result<Self> {
        sqlx::query!(
            "UPDATE task SET name = $1, des = $2, done = $3, time = $4, iid = $5 WHERE id = $6",
            self.name,
            self.des,
            self.done,
            self.time,
            self.iid,
            self.id
        )
        .execute(pool)
        .await?;
        Ok(self)
    }
}

impl Task {
    fn set_name(mut self, name: &str) -> Self {
        self.name = String::from(name);
        self
    }

    fn set_description(mut self, description: &str) -> Self {
        self.des = String::from(description);
        self
    }

    fn set_done(mut self, done: bool) -> Self {
        self.done = done;
        self
    }

    fn toggle_done(mut self) -> Self {
        self.done = !self.done;
        self
    }

    fn set_time(mut self, time_in_sec: i64) -> Self {
        self.time = time_in_sec;
        self
    }

    fn set_importance(mut self, importance: Importance) -> Self {
        self.iid = importance.id;
        self
    }

    async fn add_tag(self, tag: Tag, pool: &SqlitePool) -> anyhow::Result<Self> {
        sqlx::query!(
            "INSERT INTO tagassignment (tkid, tgid) VALUES ($1, $2)",
            self.id,
            tag.id
        )
        .execute(pool)
        .await?;

        Ok(self)
    }

    async fn assigned_tags(&self, pool: &SqlitePool) -> anyhow::Result<Vec<Tag>> {
        let tags = sqlx::query_as!(
            Tag,
            "SELECT tg.id, tg.name FROM tag tg INNER JOIN tagassignment ta ON tg.id = ta.tgid WHERE ta.tkid = $1",
            self.id
        )
        .fetch_all(pool)
        .await?;

        Ok(tags)
    }

    async fn remove_tag(self, tag: Tag, pool: &SqlitePool) -> anyhow::Result<Self> {
        sqlx::query!(
            "DELETE FROM tagassignment WHERE tkid = $1 AND tgid = $2",
            self.id,
            tag.id
        )
        .execute(pool)
        .await?;

        Ok(self)
    }
}

#[derive(Debug, Default)]
struct Importance {
    id: i64,
    name: String,
    val: i64,
}

impl CommonRecord for Importance {
    async fn db_add(mut self, pool: &SqlitePool) -> anyhow::Result<Self> {
        self.id = sqlx::query!(
            "INSERT INTO importance (name, val) VALUES ($1, $2)",
            self.name,
            self.val
        )
        .execute(pool)
        .await?
        .last_insert_rowid();

        Ok(self)
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

    async fn db_delete(self, pool: &SqlitePool) -> anyhow::Result<Self> {
        sqlx::query!("DELETE FROM importance WHERE id = $1", self.id)
            .execute(pool)
            .await?;
        Ok(self)
    }

    async fn db_update(self, pool: &SqlitePool) -> anyhow::Result<Self> {
        sqlx::query!(
            "UPDATE importance SET name = $1, val = $2 WHERE id = $3",
            self.name,
            self.val,
            self.id
        )
        .execute(pool)
        .await?;
        Ok(self)
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
    async fn db_add(mut self, pool: &SqlitePool) -> anyhow::Result<Self> {
        self.id = sqlx::query!("INSERT INTO tag (name) VALUES ( $1 )", self.name)
            .execute(pool)
            .await?
            .last_insert_rowid();

        Ok(self)
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

    async fn db_update(self, pool: &SqlitePool) -> anyhow::Result<Self> {
        sqlx::query!("UPDATE tag SET name = $1 WHERE id = $2", self.name, self.id)
            .execute(pool)
            .await?;

        Ok(self)
    }
    async fn db_delete(self, pool: &SqlitePool) -> anyhow::Result<Self> {
        sqlx::query!("DELETE FROM tag WHERE id = $1", self.id)
            .execute(pool)
            .await?;
        Ok(self)
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

    let mut task = Task::default()
        .set_importance(
            Importance::default()
                .set_name("Lowest")
                .set_value(-1000)
                .db_add(&pool)
                .await?,
        )
        .set_name("Nothing")
        .set_description("Absolutely nothing!")
        .set_done(false)
        .set_time(0)
        .db_add(&pool)
        .await?;

    task = task
        .add_tag(Tag::default().set_name("Dev").db_add(&pool).await?, &pool)
        .await?
        .add_tag(Tag::default().set_name("Misc").db_add(&pool).await?, &pool)
        .await?
        .add_tag(
            Tag::default().set_name("Writing").db_add(&pool).await?,
            &pool,
        )
        .await?
        .remove_tag(Tag::from_name("Writing", &pool).await?.unwrap(), &pool)
        .await?;

    let assigned_tags = &task.assigned_tags(&pool).await?;

    println!("{:#?}", task);
    println!("{:#?}", assigned_tags);

    Ok(())
}
