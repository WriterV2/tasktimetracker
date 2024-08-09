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

struct AppError(anyhow::Error);

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self.0),
        )
            .into_response()
    }
}

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

#[derive(Clone)]
struct ApiContext {
    pool: SqlitePool,
}

pub async fn serve(pool: SqlitePool) {
    let app = Router::new()
        .route(
            "/api/bookings",
            get(get_bookings).post(post_booking).patch(patch_booking),
        )
        .route("/api/tags", get(get_tags).post(post_tag).patch(patch_tag))
        .layer(ServiceBuilder::new().layer(AddExtensionLayer::new(ApiContext { pool })));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

#[derive(Deserialize, Debug)]
struct BookingPatchQueryParams {
    id: i64,
    startdate: Option<i64>,
    enddate: Option<i64>,
    description: Option<String>,
}

async fn patch_booking(
    ctx: Extension<ApiContext>,
    Query(params): Query<BookingPatchQueryParams>,
) -> Result<impl IntoResponse, AppError> {
    if params.description.is_none() && params.startdate.is_none() && params.enddate.is_none() {
        return Ok(StatusCode::NO_CONTENT.into_response());
    }

    let mut query_builder: QueryBuilder<Sqlite> = QueryBuilder::new("UPDATE booking ");
    query_builder.push("SET ");
    let mut comma_necessary = false;

    if let Some(startdate) = params.startdate {
        query_builder.push("startdate = ").push_bind(startdate);
        comma_necessary = true;
    }

    if let Some(enddate) = params.enddate {
        if comma_necessary {
            query_builder.push(", ");
        }
        query_builder.push("enddate = ").push_bind(enddate);
        comma_necessary = true;
    }

    if let Some(description) = params.description {
        if comma_necessary {
            query_builder.push(", ");
        }
        query_builder.push("des = ").push_bind(description);
    }

    query_builder
        .push(" WHERE id = ")
        .push_bind(params.id)
        .push(" RETURNING id, startdate, enddate, des");

    let booking = query_builder
        .build_query_as::<Booking>()
        .fetch_one(&ctx.pool)
        .await?;

    Ok(Json(booking).into_response())
}

#[derive(Deserialize, Debug)]
struct BookingPostQueryParams {
    enddate: Option<i64>,
    description: Option<String>,
}

async fn post_booking(
    ctx: Extension<ApiContext>,
    Query(params): Query<BookingPostQueryParams>,
) -> Result<impl IntoResponse, AppError> {
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
        .await?;

    Ok((StatusCode::CREATED, Json(booking)).into_response())
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
) -> Result<impl IntoResponse, AppError> {
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
        .await?;

    Ok(Json(bookings))
}

#[derive(Deserialize, Debug)]
struct TagGetQueryParams {
    id: Option<i64>,
    name: Option<String>,
}

async fn get_tags(
    ctx: Extension<ApiContext>,
    Query(params): Query<TagGetQueryParams>,
) -> Result<impl IntoResponse, AppError> {
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
        .await?;

    Ok(Json(tags))
}

#[derive(Deserialize, Debug)]
struct TagPostQueryParams {
    name: String,
}

async fn post_tag(
    ctx: Extension<ApiContext>,
    Query(params): Query<TagPostQueryParams>,
) -> Result<impl IntoResponse, AppError> {
    let tag = sqlx::query_as!(
        Tag,
        "INSERT INTO tag (name) VALUES ($1) RETURNING id, name",
        params.name
    )
    .fetch_one(&ctx.pool)
    .await?;

    Ok((StatusCode::CREATED, Json(tag)).into_response())
}

#[derive(Debug, Deserialize)]
struct TagPatchQueryParams {
    id: i64,
    name: Option<String>,
}

async fn patch_tag(
    ctx: Extension<ApiContext>,
    Query(params): Query<TagPatchQueryParams>,
) -> Result<impl IntoResponse, AppError> {
    if let Some(name) = params.name {
        let mut query_builder: QueryBuilder<Sqlite> = QueryBuilder::new("");
        query_builder
            .push("UPDATE tag SET name = ")
            .push_bind(name)
            .push(" WHERE id = ")
            .push_bind(params.id)
            .push(" RETURNING id, name");
        let tags = query_builder
            .build_query_as::<Tag>()
            .fetch_all(&ctx.pool)
            .await?;

        Ok(Json(tags).into_response())
    } else {
        Ok(StatusCode::NO_CONTENT.into_response())
    }
}

// Tags can be added to a task for categorization and organisation
#[derive(Debug, Serialize, FromRow, Deserialize)]
pub struct Tag {
    id: i64,
    name: String,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Booking {
    id: i64,
    startdate: i64,
    enddate: Option<i64>,
    des: String,
}
