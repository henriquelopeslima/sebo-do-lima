use axum::{
    Json,
    extract::{Path, State, rejection::JsonRejection},
    http::StatusCode,
};
use sqlx::PgPool;

use super::{
    dto::{LoanCreate, LoanReturn},
    repository,
};
use crate::{error::AppError, loans::model::Loan};

pub async fn create(
    State(pool): State<PgPool>,
    payload: Result<Json<LoanCreate>, JsonRejection>,
) -> Result<(StatusCode, Json<Loan>), AppError> {
    let Json(input) = payload?;
    let loan = repository::create(&pool, &input).await?;
    Ok((StatusCode::CREATED, Json(loan)))
}

pub async fn return_loan(
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
) -> Result<Json<LoanReturn>, AppError> {
    Ok(Json(repository::return_loan(&pool, id).await?))
}

pub async fn overdue(State(pool): State<PgPool>) -> Result<Json<Vec<Loan>>, AppError> {
    Ok(Json(repository::overdue(&pool).await?))
}
