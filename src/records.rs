use anyhow::Context;
use sqlx::SqlitePool;

// Corresponds to a table record
pub trait CommonRecord: std::fmt::Debug {
    // Add this record to the database
    async fn db_add(self, pool: &SqlitePool) -> anyhow::Result<Self>
    where
        Self: Sized;

    // Get this data from an existing database record with the given id
    async fn from_id(id: i64, pool: &SqlitePool) -> anyhow::Result<Option<Self>>
    where
        Self: Sized;

    // Crate records from all database records of the respective table
    async fn all(pool: &SqlitePool) -> anyhow::Result<Vec<Self>>
    where
        Self: Sized;

    // Update the corresponding existing database record to this state (excluding id)
    async fn db_update(self, pool: &SqlitePool) -> anyhow::Result<Self>
    where
        Self: Sized;

    // Delete the corresponding existing database record
    async fn db_delete(self, pool: &SqlitePool) -> anyhow::Result<Self>
    where
        Self: Sized;
}

// Add a task to track the time spent on it until it's done
#[derive(Debug, Default)]
pub struct Task {
    id: i64,
    name: String,
    des: String,
    done: bool,
    iid: i64,
}

impl CommonRecord for Task {
    // Add this task to the database and update the id to the database id
    async fn db_add(mut self, pool: &SqlitePool) -> anyhow::Result<Self> {
        self.id = sqlx::query!(
            "INSERT INTO task (name, des, done, iid) VALUES ($1, $2, $3, $4)",
            self.name,
            self.des,
            self.done,
            self.iid
        )
        .execute(pool)
        .await?
        .last_insert_rowid();

        Ok(self)
    }

    // Get a task from an corresponding existing database record with the given id
    async fn from_id(id: i64, pool: &SqlitePool) -> anyhow::Result<Option<Self>> {
        let importance = sqlx::query_as!(Self, "SELECT * FROM task WHERE id = $1", id)
            .fetch_optional(pool)
            .await?;

        Ok(importance)
    }

    // Get all tasks from the database
    async fn all(pool: &SqlitePool) -> anyhow::Result<Vec<Self>> {
        let vec = sqlx::query_as!(Self, "SELECT * FROM task")
            .fetch_all(pool)
            .await?;
        Ok(vec)
    }

    // Delete the corresponding existing database record
    async fn db_delete(self, pool: &SqlitePool) -> anyhow::Result<Self> {
        sqlx::query!("DELETE FROM task WHERE id = $1", self.id)
            .execute(pool)
            .await?;
        Ok(self)
    }

    // Update the corresponding existing database record to this state
    async fn db_update(self, pool: &SqlitePool) -> anyhow::Result<Self> {
        sqlx::query!(
            "UPDATE task SET name = $1, des = $2, done = $3, iid = $5 WHERE id = $6",
            self.name,
            self.des,
            self.done,
            self.iid,
            self.id
        )
        .execute(pool)
        .await?;
        Ok(self)
    }
}

impl Task {
    // Set the name/title of the task
    pub fn set_name(mut self, name: &str) -> Self {
        self.name = String::from(name);
        self
    }

    // Set the description of the task
    pub fn set_description(mut self, description: &str) -> Self {
        self.des = String::from(description);
        self
    }

    // Mark this task as done
    pub fn set_done(mut self, done: bool) -> Self {
        self.done = done;
        self
    }

    // Set the importance to the given importance level (see Importance struct)
    pub fn set_importance(mut self, importance: Importance) -> Self {
        self.iid = importance.id;
        self
    }

    // Assign the given tag to this task (see Tag struct)
    pub async fn add_tag(self, tag: Tag, pool: &SqlitePool) -> anyhow::Result<Self> {
        sqlx::query!(
            "INSERT INTO tagassignment (tkid, tgid) VALUES ($1, $2)",
            self.id,
            tag.id
        )
        .execute(pool)
        .await?;

        Ok(self)
    }

    // Get all tags assigned to this task
    pub async fn assigned_tags(&self, pool: &SqlitePool) -> anyhow::Result<Vec<Tag>> {
        let tags = sqlx::query_as!(
            Tag,
            "SELECT tg.id, tg.name FROM tag tg INNER JOIN tagassignment ta ON tg.id = ta.tgid WHERE ta.tkid = $1",
            self.id
        )
        .fetch_all(pool)
        .await?;

        Ok(tags)
    }

    // Remove an assigned tag
    pub async fn remove_tag(self, tag: Tag, pool: &SqlitePool) -> anyhow::Result<Self> {
        sqlx::query!(
            "DELETE FROM tagassignment WHERE tkid = $1 AND tgid = $2",
            self.id,
            tag.id
        )
        .execute(pool)
        .await?;

        Ok(self)
    }

    pub async fn get_real_id(&self, pool: &SqlitePool) -> anyhow::Result<i64> {
        if let Some(task) = Self::from_id(self.id, pool).await? {
            Ok(task.id)
        } else {
            Err(sqlx::Error::RowNotFound).with_context(|| {
                format!(
                    "Task with temporary id {} is not in database and thus has no real id",
                    self.id
                )
            })
        }
    }
}

// Different levels of importance can be specified and named
#[derive(Debug, Default)]
pub struct Importance {
    id: i64,
    name: String,
    val: i64,
}

impl CommonRecord for Importance {
    // Add this importance level to the database and update the id to the database id
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

    // Get an importance level from an corresponding existing database record with the given id
    async fn from_id(id: i64, pool: &SqlitePool) -> anyhow::Result<Option<Self>> {
        let importance = sqlx::query_as!(Self, "SELECT * FROM importance WHERE id = $1", id)
            .fetch_optional(pool)
            .await?;

        Ok(importance)
    }

    // Get all importance levels from the database
    async fn all(pool: &SqlitePool) -> anyhow::Result<Vec<Self>> {
        let vec = sqlx::query_as!(Self, "SELECT * FROM importance")
            .fetch_all(pool)
            .await?;
        Ok(vec)
    }

    // Delete the corresponding existing database record
    async fn db_delete(self, pool: &SqlitePool) -> anyhow::Result<Self> {
        sqlx::query!("DELETE FROM importance WHERE id = $1", self.id)
            .execute(pool)
            .await?;
        Ok(self)
    }

    // Update the corresponding existing database record to this state
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
    // Get an importance level from the database with the given name
    async fn from_name(name: &str, pool: &SqlitePool) -> anyhow::Result<Option<Self>> {
        let importance = sqlx::query_as!(Self, "SELECT * FROM importance WHERE name = $1", name)
            .fetch_optional(pool)
            .await?;

        Ok(importance)
    }

    // Get an importance level from the database with the given value
    async fn from_value(val: i64, pool: &SqlitePool) -> anyhow::Result<Option<Self>> {
        let importance = sqlx::query_as!(Self, "SELECT * FROM importance WHERE val = $1", val)
            .fetch_optional(pool)
            .await?;

        Ok(importance)
    }

    // Set the name of this importance level
    pub fn set_name(mut self, name: &str) -> Self {
        self.name = String::from(name);
        self
    }

    // Set the value of this importance level
    pub fn set_value(mut self, value: i64) -> Self {
        self.val = value;
        self
    }
}

// Tags can be added to a task for categorization and organisation
#[derive(Debug, Default)]
pub struct Tag {
    id: i64,
    name: String,
}

impl CommonRecord for Tag {
    // Add this tag to the database and update the id to the database id
    async fn db_add(mut self, pool: &SqlitePool) -> anyhow::Result<Self> {
        self.id = sqlx::query!("INSERT INTO tag (name) VALUES ( $1 )", self.name)
            .execute(pool)
            .await?
            .last_insert_rowid();

        Ok(self)
    }

    // Get a tag from an existing record with the given id
    async fn from_id(id: i64, pool: &SqlitePool) -> anyhow::Result<Option<Self>> {
        let tag = sqlx::query_as!(Self, "SELECT * FROM tag WHERE id = $1", id)
            .fetch_optional(pool)
            .await?;

        Ok(tag)
    }

    // Get tags from all existing tag records
    async fn all(pool: &SqlitePool) -> anyhow::Result<Vec<Self>> {
        let vec = sqlx::query_as!(Self, "SELECT * FROM tag")
            .fetch_all(pool)
            .await?;
        Ok(vec)
    }

    // Update the corresponding existing database record to this state
    async fn db_update(self, pool: &SqlitePool) -> anyhow::Result<Self> {
        sqlx::query!("UPDATE tag SET name = $1 WHERE id = $2", self.name, self.id)
            .execute(pool)
            .await?;

        Ok(self)
    }

    // Delete the corresponding existing database record
    async fn db_delete(self, pool: &SqlitePool) -> anyhow::Result<Self> {
        sqlx::query!("DELETE FROM tag WHERE id = $1", self.id)
            .execute(pool)
            .await?;
        Ok(self)
    }
}

impl Tag {
    // Get a tag from an existing record with the given name
    pub async fn from_name(name: &str, pool: &SqlitePool) -> anyhow::Result<Option<Self>> {
        let tag = sqlx::query_as!(Self, "SELECT * FROM tag WHERE name = $1", name)
            .fetch_optional(pool)
            .await?;

        Ok(tag)
    }

    // Set the name of this tag
    pub fn set_name(mut self, name: &str) -> Self {
        self.name = String::from(name);
        self
    }
}

#[derive(Debug)]
pub struct Booking {
    id: i64,
    tid: i64,
    startdate: i64,
    enddate: Option<i64>,
}

impl CommonRecord for Booking {
    async fn db_add(mut self, pool: &SqlitePool) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        self.id = sqlx::query!(
            "INSERT INTO booking (tid, startdate, enddate) VALUES ($1, $2, $3)",
            self.tid,
            self.startdate,
            self.enddate
        )
        .execute(pool)
        .await?
        .last_insert_rowid();

        Ok(self)
    }

    async fn from_id(id: i64, pool: &SqlitePool) -> anyhow::Result<Option<Self>> {
        let booking = sqlx::query_as!(Self, "SELECT * FROM booking WHERE id = $1", id)
            .fetch_optional(pool)
            .await?;

        Ok(booking)
    }

    // Get all tasks from the database
    async fn all(pool: &SqlitePool) -> anyhow::Result<Vec<Self>> {
        let vec = sqlx::query_as!(Self, "SELECT * FROM booking")
            .fetch_all(pool)
            .await?;
        Ok(vec)
    }

    // Delete the corresponding existing database record
    async fn db_delete(self, pool: &SqlitePool) -> anyhow::Result<Self> {
        sqlx::query!("DELETE FROM booking WHERE id = $1", self.id)
            .execute(pool)
            .await?;
        Ok(self)
    }

    // Update the corresponding existing database record to this state
    async fn db_update(self, pool: &SqlitePool) -> anyhow::Result<Self> {
        sqlx::query!(
            "UPDATE booking SET tid = $1, startdate = $2, enddate = $3 WHERE id = $4",
            self.tid,
            self.startdate,
            self.enddate,
            self.id
        )
        .execute(pool)
        .await?;
        Ok(self)
    }
}

impl Booking {
    pub async fn set_task(mut self, task_id: i64, pool: &SqlitePool) -> anyhow::Result<Self> {
        if let Some(task) = Task::from_id(task_id, pool).await? {
            self.tid = task.id;
            Ok(self)
        } else {
            Err(sqlx::Error::RowNotFound)
                .with_context(|| format!("Task with id {} not found", task_id))
        }
    }

    pub fn finish_task(mut self) -> anyhow::Result<Self> {
        let time = std::time::SystemTime::now();
        let time = time
            .duration_since(std::time::UNIX_EPOCH)
            .expect("System time must be after 1970-01-01 00:00:00 UTC")
            .as_millis() as i64;
        self.enddate = Some(time);
        Ok(self)
    }
}

impl Default for Booking {
    fn default() -> Self {
        let time = std::time::SystemTime::now();
        let time = time
            .duration_since(std::time::UNIX_EPOCH)
            .expect("System time must be after 1970-01-01 00:00:00 UTC")
            .as_millis() as i64;
        Self {
            id: 0,
            tid: 0,
            startdate: time,
            enddate: None,
        }
    }
}
