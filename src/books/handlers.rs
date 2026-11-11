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
    repository,
};
use crate::{entities::Book, error::AppError};

const NOT_FOUND: AppError = AppError::NotFound("Livro não encontrado");

pub async fn list(
    State(pool): State<PgPool>,
    filter: Result<Query<BookFilter>, QueryRejection>,
) -> Result<Json<Vec<Book>>, AppError> {
    let Query(filter) = filter?;
    Ok(Json(repository::list(&pool, &filter).await?))
}

pub async fn create(
    State(pool): State<PgPool>,
    payload: Result<Json<BookCreate>, JsonRejection>,
) -> Result<(StatusCode, Json<Book>), AppError> {
    let Json(input) = payload?;
    input.validate()?;
    let book = repository::create(&pool, &input).await?;
    Ok((StatusCode::CREATED, Json(book)))
}

pub async fn find(State(pool): State<PgPool>, Path(id): Path<i64>) -> Result<Json<Book>, AppError> {
    let book = repository::find(&pool, id).await?.ok_or(NOT_FOUND)?;
    Ok(Json(book))
}

pub async fn update(
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
    payload: Result<Json<BookUpdate>, JsonRejection>,
) -> Result<Json<Book>, AppError> {
    let Json(input) = payload?;
    input.validate()?;
    let book = repository::update(&pool, id, &input)
        .await?
        .ok_or(NOT_FOUND)?;
    Ok(Json(book))
}

pub async fn delete(
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
) -> Result<StatusCode, AppError> {
    match repository::delete(&pool, id).await {
        Ok(true) => Ok(StatusCode::NO_CONTENT),
        Ok(false) => Err(NOT_FOUND),
        // o histórico de empréstimos referencia o livro; apagar o livro o destruiria
        Err(error) if is_foreign_key_violation(&error) => {
            Err(AppError::Conflict("Livro possui empréstimos registrados"))
        }
        Err(error) => Err(error.into()),
    }
}

fn is_foreign_key_violation(error: &sqlx::Error) -> bool {
    error
        .as_database_error()
        .is_some_and(sqlx::error::DatabaseError::is_foreign_key_violation)
}
