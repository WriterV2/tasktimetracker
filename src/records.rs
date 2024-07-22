use anyhow::{Context, Ok};
use async_trait::async_trait;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Extension, Json, Router};
use axum_extra::extract::Query;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use sqlx::{QueryBuilder, Sqlite, SqlitePool};
use tower::ServiceBuilder;
use tower_http::add_extension::AddExtensionLayer;

#[derive(Clone)]
struct ApiContext {
    pool: SqlitePool,
}

pub async fn serve(pool: SqlitePool) -> anyhow::Result<()> {
    let app = Router::new()
        .route("/api/bookings", get(get_bookings).post(post_booking))
        .route("/api/tags", get(get_tags))
        .layer(ServiceBuilder::new().layer(AddExtensionLayer::new(ApiContext { pool })));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app.into_make_service()).await?;
    Ok(())
}

#[derive(Deserialize, Debug)]
struct BookingPostQueryParams {
    enddate: Option<i64>,
    description: Option<String>,
}

async fn post_booking(
    ctx: Extension<ApiContext>,
    Query(params): Query<BookingPostQueryParams>,
) -> impl IntoResponse {
    let mut query_builder: QueryBuilder<Sqlite> =
        QueryBuilder::new("INSERT INTO BOOKING (startdate");
    let time = std::time::SystemTime::now();
    let startdate = time
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64;

    if params.enddate.is_some() {
        query_builder.push(", enddate");
    }

    if params.description.is_some() {
        query_builder.push(", des");
    }

    query_builder.push(") VALUES (").push_bind(startdate);

    if let Some(enddate) = params.enddate {
        query_builder.push(", ").push_bind(enddate);
    }

    if let Some(description) = params.description {
        query_builder.push(", ").push_bind(description);
    }

    query_builder.push(") RETURNING id, startdate, enddate, des");

    let booking = query_builder
        .build_query_as::<Booking>()
        .fetch_one(&ctx.pool)
        .await
        .unwrap();

    (StatusCode::CREATED, Json(booking))
}

#[derive(Deserialize, Debug)]
struct BookingGetQueryParams {
    id: Option<i64>,
    startdate_min: Option<i64>,
    startdate_max: Option<i64>,
    enddate_min: Option<i64>,
    enddate_max: Option<i64>,
    tag: Option<Vec<String>>,
    description_contains: Option<String>,
}

async fn get_bookings(
    ctx: Extension<ApiContext>,
    Query(params): Query<BookingGetQueryParams>,
) -> impl IntoResponse {
    let mut query_builder: QueryBuilder<Sqlite> = QueryBuilder::new("SELECT * FROM booking");

    if params.tag.is_some() {
        query_builder.push(
            " b INNER JOIN tagassignment tg ON b.id = tg.bid INNER JOIN tag t ON t.id = tg.tgid",
        );
    }
    query_builder.push(" WHERE TRUE");

    if let Some(id) = params.id {
        query_builder.push(" AND ");
        if params.tag.is_some() {
            query_builder.push("b.");
        }
        query_builder.push("id = ").push_bind(id);
    }

    if let Some(startdate_min) = params.startdate_min {
        query_builder
            .push(" AND startdate > ")
            .push_bind(startdate_min);
    }

    if let Some(startdate_max) = params.startdate_max {
        query_builder
            .push(" AND startdate < ")
            .push_bind(startdate_max);
    }

    if let Some(enddate_min) = params.enddate_min {
        query_builder.push(" AND enddate > ").push_bind(enddate_min);
    }

    if let Some(enddate_max) = params.enddate_max {
        query_builder.push(" AND enddate > ").push_bind(enddate_max);
    }

    if let Some(description_contains) = params.description_contains {
        query_builder
            .push(" AND des LIKE CONCAT('%', ")
            .push_bind(description_contains)
            .push(", '%')");
    }

    if let Some(tags) = params.tag {
        query_builder.push(" AND t.name IN (");
        let tags_len = tags.len();
        for (index, tag) in tags.into_iter().enumerate() {
            query_builder.push_bind(tag);
            if index < tags_len - 1 {
                query_builder.push(",");
            }
        }
        query_builder.push(")");
    }

    let bookings = query_builder
        .build_query_as::<Booking>()
        .fetch_all(&ctx.pool)
        .await
        .unwrap();

    Json(bookings)
}

#[derive(Deserialize, Debug)]
struct TagGetQueryParams {
    id: Option<i64>,
    name: Option<String>,
}

async fn get_tags(
    ctx: Extension<ApiContext>,
    Query(params): Query<TagGetQueryParams>,
) -> impl IntoResponse {
    let mut query_builder: QueryBuilder<Sqlite> = QueryBuilder::new("SELECT * FROM tag WHERE TRUE");

    if let Some(id) = params.id {
        query_builder.push(" AND id = ").push_bind(id);
    }

    if let Some(name) = params.name {
        query_builder.push(" AND name = ").push_bind(name);
    }

    let tags = query_builder
        .build_query_as::<Tag>()
        .fetch_all(&ctx.pool)
        .await
        .unwrap();

    Json(tags)
}

#[async_trait]
pub trait Record: std::fmt::Debug {
    type Existing: ExistingRecord;
    // Add this record to the database or update it
    async fn save(self, pool: &SqlitePool) -> anyhow::Result<Self::Existing>
    where
        Self: Sized;
}

// Record that does not exist in the database (yet)
#[async_trait]
pub trait NewRecord: Record + Default {
    async fn new() -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        Ok(Self::default())
    }
}

// Record from the database
#[async_trait]
pub trait ExistingRecord: Record + serde::Serialize {
    async fn from_id(id: i64, pool: &SqlitePool) -> anyhow::Result<Self>
    where
        Self: Sized;

    async fn delete(self, pool: &SqlitePool) -> anyhow::Result<Self>
    where
        Self: Sized;
}

// Tags can be added to a task for categorization and organisation
#[derive(Debug, Serialize, FromRow, Deserialize)]
pub struct Tag {
    id: i64,
    name: String,
}

#[derive(Debug, Default)]
pub struct NewTag {
    name: String,
}

#[async_trait]
impl Record for NewTag {
    type Existing = Tag;
    async fn save(self, pool: &SqlitePool) -> anyhow::Result<Self::Existing>
    where
        Self: Sized,
    {
        let id = sqlx::query!("INSERT INTO tag (name) VALUES ( $1 )", self.name)
            .execute(pool)
            .await?
            .last_insert_rowid();

        Ok(Tag {
            id,
            name: self.name,
        })
    }
}

#[async_trait]
impl Record for Tag {
    type Existing = Self;
    async fn save(self, pool: &SqlitePool) -> anyhow::Result<Self::Existing>
    where
        Self: Sized,
    {
        sqlx::query!("UPDATE tag SET name = $1 WHERE id = $2", self.name, self.id)
            .execute(pool)
            .await?;
        Ok(self)
    }
}

#[async_trait]
impl ExistingRecord for Tag {
    async fn delete(self, pool: &SqlitePool) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        sqlx::query!("DELETE FROM tag WHERE id = $1", self.id)
            .execute(pool)
            .await?;
        Ok(self)
    }

    async fn from_id(id: i64, pool: &SqlitePool) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let tag = sqlx::query_as!(Self, "SELECT * FROM tag WHERE id = $1", id)
            .fetch_optional(pool)
            .await?;

        if let Some(t) = tag {
            Ok(t)
        } else {
            Err(sqlx::Error::RowNotFound)
                .with_context(|| format!("Tag with id {} is not in database", id))
        }
    }
}

#[async_trait]
impl NewRecord for NewTag {
    async fn new() -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        Ok(Self {
            name: String::new(),
        })
    }
}

impl Tag {
    pub async fn from_name(name: &str, pool: &SqlitePool) -> anyhow::Result<Self> {
        let tag = sqlx::query_as!(Self, "SELECT * FROM tag WHERE name = $1", name)
            .fetch_optional(pool)
            .await?;

        if let Some(t) = tag {
            Ok(t)
        } else {
            Err(sqlx::Error::RowNotFound)
                .with_context(|| format!("Tag with name {} is not in database", name))
        }
    }

    pub fn set_name(mut self, name: &str) -> Self {
        self.name = String::from(name);
        self
    }
}

impl NewTag {
    pub fn set_name(mut self, name: &str) -> Self {
        self.name = String::from(name);
        self
    }
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Booking {
    id: i64,
    startdate: i64,
    enddate: Option<i64>,
    des: String,
}

#[derive(Debug, Default)]
pub struct NewBooking {
    startdate: i64,
    enddate: Option<i64>,
    des: String,
}

#[async_trait]
impl Record for Booking {
    type Existing = Self;
    async fn save(self, pool: &SqlitePool) -> anyhow::Result<Self::Existing>
    where
        Self: Sized,
    {
        sqlx::query!(
            "UPDATE booking SET startdate = $1, enddate = $2, des = $3 WHERE id = $4",
            self.startdate,
            self.enddate,
            self.des,
            self.id
        )
        .execute(pool)
        .await?;
        Ok(self)
    }
}

#[async_trait]
impl Record for NewBooking {
    type Existing = Booking;
    async fn save(self, pool: &SqlitePool) -> anyhow::Result<Self::Existing>
    where
        Self: Sized,
    {
        let id = sqlx::query!(
            "INSERT INTO booking (startdate, enddate, des) VALUES ($1, $2, $3)",
            self.startdate,
            self.enddate,
            self.des
        )
        .execute(pool)
        .await?
        .last_insert_rowid();

        Ok(Booking {
            id,
            startdate: self.startdate,
            enddate: self.enddate,
            des: self.des,
        })
    }
}

#[async_trait]
impl ExistingRecord for Booking {
    async fn delete(self, pool: &SqlitePool) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        sqlx::query!("DELETE FROM booking WHERE id = $1", self.id)
            .execute(pool)
            .await?;
        Ok(self)
    }

    async fn from_id(id: i64, pool: &SqlitePool) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let booking = sqlx::query_as!(Self, "SELECT * FROM booking WHERE id = $1", id)
            .fetch_optional(pool)
            .await?;

        if let Some(b) = booking {
            Ok(b)
        } else {
            Err(sqlx::Error::RowNotFound)
                .with_context(|| format!("Booking with id {} is not in database", id))
        }
    }
}

impl Booking {
    pub fn finish(mut self) -> anyhow::Result<Self> {
        if !self.is_finished() {
            let time = std::time::SystemTime::now();
            let time = time
                .duration_since(std::time::UNIX_EPOCH)
                .expect("System time must be after 1970-01-01 00:00:00 UTC")
                .as_millis() as i64;
            self.enddate = Some(time);
            Ok(self)
        } else {
            Err(anyhow::anyhow!("Booking is already finished"))
        }
    }

    pub fn is_finished(&self) -> bool {
        self.enddate.is_some()
    }

    pub fn set_description(mut self, description: &str) -> Self {
        self.des = String::from(description);
        self
    }

    pub async fn add_tag(self, tag: Tag, pool: &SqlitePool) -> anyhow::Result<Self> {
        sqlx::query!(
            "INSERT INTO tagassignment (bid, tgid) VALUES ($1, $2)",
            self.id,
            tag.id
        )
        .execute(pool)
        .await?;

        Ok(self)
    }

    pub async fn assigned_tags(&self, pool: &SqlitePool) -> anyhow::Result<Vec<Tag>> {
        let tags = sqlx::query_as!(
            Tag,
            "SELECT tg.id, tg.name FROM tag tg INNER JOIN tagassignment ta ON tg.id = ta.tgid WHERE ta.bid = $1",
            self.id
        )
        .fetch_all(pool)
        .await?;

        Ok(tags)
    }

    pub async fn remove_tag(self, tag: Tag, pool: &SqlitePool) -> anyhow::Result<Self> {
        sqlx::query!(
            "DELETE FROM tagassignment WHERE bid = $1 AND tgid = $2",
            self.id,
            tag.id
        )
        .execute(pool)
        .await?;

        Ok(self)
    }
}

impl NewBooking {
    pub fn set_description(mut self, description: &str) -> Self {
        self.des = String::from(description);
        self
    }
}

#[async_trait]
impl NewRecord for NewBooking {
    async fn new() -> anyhow::Result<Self> {
        let time = std::time::SystemTime::now();
        let time = time.duration_since(std::time::UNIX_EPOCH)?.as_millis() as i64;

        Ok(NewBooking {
            startdate: time,
            enddate: None,
            des: String::new(),
        })
    }
}
