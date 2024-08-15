use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Extension, Json};
use axum_extra::extract::Query;
use serde::Deserialize;
use sqlx::{QueryBuilder, Sqlite};

use crate::error::AppError;
use crate::tagassignment::TagAssignment;

use super::ApiContext;

#[derive(Deserialize, Debug)]
pub struct TagAssignmentPostQueryParams {
    tag_id: Vec<i64>,
    booking_id: i64,
}

pub async fn post_tagassignment(
    ctx: Extension<ApiContext>,
    Query(params): Query<TagAssignmentPostQueryParams>,
) -> Result<impl IntoResponse, AppError> {
    let mut tx = ctx.pool.begin().await?;
    let mut query_builder: QueryBuilder<Sqlite> =
        QueryBuilder::new("INSERT INTO tagassignment (tgid, bid) VALUES");

    for (i, tid) in params.tag_id.iter().enumerate() {
        query_builder.push(" (");
        query_builder.push_bind(tid);
        query_builder.push(", ");
        query_builder.push_bind(params.booking_id);
        query_builder.push(")");
        if params.tag_id.len() - 1 > i {
            query_builder.push(",");
        }
    }
    query_builder.push(" RETURNING tgid, bid;");
    let result: Vec<TagAssignment> = query_builder.build_query_as().fetch_all(&mut *tx).await?;
    tx.commit().await?;
    Ok((StatusCode::CREATED, Json(result)))
}
