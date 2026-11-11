//! Espelha o schema `Book` de docs/contract-api.yml.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn status_segue_o_contrato() {
        assert_eq!(json!(BookStatus::Manutencao), json!("MANUTENCAO"));
    }
}
