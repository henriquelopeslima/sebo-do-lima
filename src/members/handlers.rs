use axum::{
    Json,
    extract::{Path, State, rejection::JsonRejection},
    http::StatusCode,
};
use sqlx::PgPool;

use super::{dto::MemberCreate, repository};
use crate::{error::AppError, loans::model::Loan, members::model::Member};

const NOT_FOUND: AppError = AppError::NotFound("Membro não encontrado");

pub async fn list(State(pool): State<PgPool>) -> Result<Json<Vec<Member>>, AppError> {
    Ok(Json(repository::list(&pool).await?))
}

pub async fn create(
    State(pool): State<PgPool>,
    payload: Result<Json<MemberCreate>, JsonRejection>,
) -> Result<(StatusCode, Json<Member>), AppError> {
    let Json(input) = payload?;
    input.validate()?;
    let member = repository::create(&pool, &input).await.map_err(|error| {
        let duplicate = error
            .as_database_error()
            .is_some_and(sqlx::error::DatabaseError::is_unique_violation);
        if duplicate {
            // o contrato só prevê 400 para o cadastro
            AppError::Validation("email já cadastrado".into())
        } else {
            error.into()
        }
    })?;
    Ok((StatusCode::CREATED, Json(member)))
}

pub async fn find(
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
) -> Result<Json<Member>, AppError> {
    let member = repository::find(&pool, id).await?.ok_or(NOT_FOUND)?;
    Ok(Json(member))
}

pub async fn loans(
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
) -> Result<Json<Vec<Loan>>, AppError> {
    repository::find(&pool, id).await?.ok_or(NOT_FOUND)?;
    Ok(Json(repository::loans(&pool, id).await?))
}
