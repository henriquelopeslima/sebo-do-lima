//! Espelha o schema `Member` de docs/contract-api.yml.

use chrono::NaiveDate;
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, sqlx::Type)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[sqlx(type_name = "text", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MemberStatus {
    Ativo,
    Suspenso,
    Inativo,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Member {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub phone: Option<String>,
    pub status: MemberStatus,
    pub loan_limit: i32,
    pub registration_date: NaiveDate,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn status_segue_o_contrato() {
        assert_eq!(json!(MemberStatus::Suspenso), json!("SUSPENSO"));
    }
}
