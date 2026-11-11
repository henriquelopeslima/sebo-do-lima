use axum::{
    Json,
    extract::{
        Path, Query, State,
        rejection::{JsonRejection, QueryRejection},
    },
    http::StatusCode,
};
use sqlx::PgPool;

use super::{
    dto::{BookCreate, BookFilter, BookUpdate},
    model::Book,
    service,
};
use crate::error::AppError;

pub async fn list(
    State(pool): State<PgPool>,
    filter: Result<Query<BookFilter>, QueryRejection>,
) -> Result<Json<Vec<Book>>, AppError> {
    let Query(filter) = filter?;
    Ok(Json(service::list(&pool, &filter).await?))
}

pub async fn create(
    State(pool): State<PgPool>,
    payload: Result<Json<BookCreate>, JsonRejection>,
) -> Result<(StatusCode, Json<Book>), AppError> {
    let Json(input) = payload?;
    let book = service::create(&pool, &input).await?;
    Ok((StatusCode::CREATED, Json(book)))
}

pub async fn find(State(pool): State<PgPool>, Path(id): Path<i64>) -> Result<Json<Book>, AppError> {
    Ok(Json(service::find(&pool, id).await?))
}

pub async fn update(
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
    payload: Result<Json<BookUpdate>, JsonRejection>,
) -> Result<Json<Book>, AppError> {
    let Json(input) = payload?;
    Ok(Json(service::update(&pool, id, &input).await?))
}

pub async fn delete(
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
) -> Result<StatusCode, AppError> {
    service::delete(&pool, id).await?;
    Ok(StatusCode::NO_CONTENT)
}
