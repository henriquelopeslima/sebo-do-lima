use sqlx::PgPool;

use super::{dto::MemberCreate, model::Member, repository};
use crate::{error::AppError, loans};

const NOT_FOUND: AppError = AppError::NotFound("Membro não encontrado");

pub async fn list(pool: &PgPool) -> Result<Vec<Member>, AppError> {
    Ok(repository::list(pool).await?)
}

pub async fn find(pool: &PgPool, id: i64) -> Result<Member, AppError> {
    repository::find(pool, id).await?.ok_or(NOT_FOUND)
}

pub async fn create(pool: &PgPool, input: &MemberCreate) -> Result<Member, AppError> {
    input.validate()?;
    repository::create(pool, input).await.map_err(|error| {
        let duplicate = error
            .as_database_error()
            .is_some_and(sqlx::error::DatabaseError::is_unique_violation);
        if duplicate {
            // o contrato só prevê 400 para o cadastro
            AppError::Validation("email já cadastrado".into())
        } else {
            error.into()
        }
    })
}

/// Histórico de empréstimos do membro.
pub async fn loans(pool: &PgPool, id: i64) -> Result<Vec<loans::model::Loan>, AppError> {
    find(pool, id).await?;
    Ok(loans::repository::list_by_member(pool, id).await?)
}
