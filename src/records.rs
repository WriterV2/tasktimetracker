use anyhow::Ok;
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
