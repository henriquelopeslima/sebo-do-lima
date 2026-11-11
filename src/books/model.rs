//! Espelha o schema `Book` de docs/contract-api.yml.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::error::AppError;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[sqlx(type_name = "text", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BookStatus {
    Disponivel,
    Emprestado,
    Manutencao,
    Perdido,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Book {
    pub id: i64,
    pub title: String,
    pub isbn: String,
    pub author: String,
    pub category: String,
    pub status: BookStatus,
    pub publish_year: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Book {
    /// Só um livro `DISPONIVEL` pode ser emprestado.
    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "passa a ser usado pelo serviço de empréstimos")
    )]
    pub fn ensure_available(&self) -> Result<(), AppError> {
        if self.status == BookStatus::Disponivel {
            Ok(())
        } else {
            Err(AppError::Validation("Livro indisponível".to_owned()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn sample() -> Book {
        Book {
            id: 55,
            title: "Dom Casmurro".into(),
            isbn: "978-85-359-0277-8".into(),
            author: "Machado de Assis".into(),
            category: "Ficção".into(),
            status: BookStatus::Disponivel,
            publish_year: Some(1899),
            created_at: DateTime::UNIX_EPOCH,
            updated_at: DateTime::UNIX_EPOCH,
        }
    }

    #[test]
    fn so_livro_disponivel_pode_ser_emprestado() {
        let book = |status| Book { status, ..sample() };
        assert!(book(BookStatus::Disponivel).ensure_available().is_ok());
        for status in [
            BookStatus::Emprestado,
            BookStatus::Manutencao,
            BookStatus::Perdido,
        ] {
            assert!(matches!(
                book(status).ensure_available(),
                Err(AppError::Validation(_))
            ));
        }
    }

    #[test]
    fn status_segue_o_contrato() {
        assert_eq!(json!(BookStatus::Manutencao), json!("MANUTENCAO"));
    }
}
