use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Extension, Json};
use axum_extra::extract::Query;
use serde::Deserialize;

use crate::error::AppError;
use crate::tagassignment::TagAssignment;

use super::ApiContext;

#[derive(Deserialize, Debug)]
pub struct TagAssignmentPostQueryParams {
    tag_id: i64,
    booking_id: i64,
}

pub async fn post_tagassignment(
    ctx: Extension<ApiContext>,
    Query(params): Query<TagAssignmentPostQueryParams>,
) -> Result<impl IntoResponse, AppError> {
    let tagassignment = sqlx::query_as!(
        TagAssignment,
        "INSERT INTO tagassignment (tgid, bid) VALUES ($1, $2) RETURNING tgid, bid",
        params.tag_id,
        params.booking_id,
    )
    .fetch_one(&ctx.pool)
    .await?;
    Ok((StatusCode::CREATED, Json(tagassignment)))
}
