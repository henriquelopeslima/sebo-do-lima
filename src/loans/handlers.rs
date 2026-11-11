use axum::{
    Json,
    extract::{Path, State, rejection::JsonRejection},
    http::StatusCode,
};
use sqlx::PgPool;

use super::{
    dto::{LoanCreate, LoanReturn},
    model::Loan,
    service,
};
use crate::error::AppError;

pub async fn create(
    State(pool): State<PgPool>,
    payload: Result<Json<LoanCreate>, JsonRejection>,
) -> Result<(StatusCode, Json<Loan>), AppError> {
    let Json(input) = payload?;
    let loan = service::create(&pool, &input).await?;
    Ok((StatusCode::CREATED, Json(loan)))
}

pub async fn return_loan(
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
) -> Result<Json<LoanReturn>, AppError> {
    Ok(Json(service::return_loan(&pool, id).await?))
}

pub async fn overdue(State(pool): State<PgPool>) -> Result<Json<Vec<Loan>>, AppError> {
    Ok(Json(service::overdue(&pool).await?))
}
