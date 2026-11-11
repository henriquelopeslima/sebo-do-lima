use axum::{
    Json,
    extract::{Path, State, rejection::JsonRejection},
    http::StatusCode,
};
use sqlx::PgPool;

use super::{dto::MemberCreate, model::Member, service};
use crate::{error::AppError, loans::model::Loan};

pub async fn list(State(pool): State<PgPool>) -> Result<Json<Vec<Member>>, AppError> {
    Ok(Json(service::list(&pool).await?))
}

pub async fn create(
    State(pool): State<PgPool>,
    payload: Result<Json<MemberCreate>, JsonRejection>,
) -> Result<(StatusCode, Json<Member>), AppError> {
    let Json(input) = payload?;
    let member = service::create(&pool, &input).await?;
    Ok((StatusCode::CREATED, Json(member)))
}

pub async fn find(
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
) -> Result<Json<Member>, AppError> {
    Ok(Json(service::find(&pool, id).await?))
}

pub async fn loans(
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
) -> Result<Json<Vec<Loan>>, AppError> {
    Ok(Json(service::loans(&pool, id).await?))
}
