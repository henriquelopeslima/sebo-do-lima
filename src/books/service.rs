use sqlx::PgPool;

use super::{
    dto::{BookCreate, BookFilter, BookUpdate},
    model::Book,
    repository,
};
use crate::error::AppError;

const NOT_FOUND: AppError = AppError::NotFound("Livro não encontrado");

pub async fn list(pool: &PgPool, filter: &BookFilter) -> Result<Vec<Book>, AppError> {
    Ok(repository::list(pool, filter).await?)
}

pub async fn find(pool: &PgPool, id: i64) -> Result<Book, AppError> {
    repository::find(pool, id).await?.ok_or(NOT_FOUND)
}

pub async fn create(pool: &PgPool, input: &BookCreate) -> Result<Book, AppError> {
    input.validate()?;
    Ok(repository::create(pool, input).await?)
}

pub async fn update(pool: &PgPool, id: i64, input: &BookUpdate) -> Result<Book, AppError> {
    input.validate()?;
    repository::update(pool, id, input).await?.ok_or(NOT_FOUND)
}

pub async fn delete(pool: &PgPool, id: i64) -> Result<(), AppError> {
    match repository::delete(pool, id).await {
        Ok(true) => Ok(()),
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
