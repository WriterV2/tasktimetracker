use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Extension, Json};
use axum_extra::extract::Query;
use serde::Deserialize;
use sqlx::{QueryBuilder, Sqlite};

use crate::booking::Booking;
use crate::error::AppError;

use super::ApiContext;

#[derive(Deserialize, Debug)]
pub struct BookingDeleteQueryParams {
    id: i64,
}

pub async fn delete_booking(
    ctx: Extension<ApiContext>,
    Query(params): Query<BookingDeleteQueryParams>,
) -> Result<impl IntoResponse, AppError> {
    sqlx::query!("DELETE FROM booking WHERE id = $1", params.id)
        .execute(&ctx.pool)
        .await?;
    Ok(())
}

#[derive(Deserialize, Debug)]
pub struct BookingPatchQueryParams {
    id: i64,
    startdate: Option<i64>,
    enddate: Option<i64>,
    description: Option<String>,
}

pub async fn patch_booking(
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
pub struct BookingPostQueryParams {
    enddate: Option<i64>,
    description: Option<String>,
}

pub async fn post_booking(
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
pub struct BookingGetQueryParams {
    id: Option<i64>,
    startdate_min: Option<i64>,
    startdate_max: Option<i64>,
    enddate_min: Option<i64>,
    enddate_max: Option<i64>,
    tag: Option<Vec<String>>,
    description_contains: Option<String>,
}

pub async fn get_bookings(
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

    let bookings: Vec<Booking> = query_builder.build_query_as().fetch_all(&ctx.pool).await?;

    Ok(Json(bookings))
}
