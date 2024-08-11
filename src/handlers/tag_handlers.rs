use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Extension, Json};
use axum_extra::extract::Query;
use serde::Deserialize;
use sqlx::{QueryBuilder, Sqlite};

use crate::error::AppError;
use crate::tag::Tag;

use super::ApiContext;

#[derive(Deserialize, Debug)]
pub struct TagGetQueryParams {
    id: Option<i64>,
    name: Option<String>,
}

pub async fn get_tags(
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
pub struct TagPostQueryParams {
    name: String,
}

pub async fn post_tag(
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
pub struct TagPatchQueryParams {
    id: i64,
    name: Option<String>,
}

pub async fn patch_tag(
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

#[derive(Deserialize, Debug)]
pub struct TagDeleteQueryParams {
    id: i64,
}

pub async fn delete_tag(
    ctx: Extension<ApiContext>,
    Query(params): Query<TagDeleteQueryParams>,
) -> Result<impl IntoResponse, AppError> {
    sqlx::query!("DELETE FROM tag WHERE id = $1", params.id)
        .execute(&ctx.pool)
        .await?;
    Ok(())
}
