//! Espelha o schema `Member` de docs/contract-api.yml.

use chrono::NaiveDate;
use serde::Serialize;

use crate::error::AppError;

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

impl Member {
    /// Só membro ativo e abaixo do limite de empréstimos em aberto pode emprestar.
    pub fn ensure_can_borrow(&self, open_loans: i64) -> Result<(), AppError> {
        if self.status != MemberStatus::Ativo {
            return Err(AppError::Validation("Membro não está ativo".to_owned()));
        }
        if open_loans >= i64::from(self.loan_limit) {
            return Err(AppError::Validation(
                "Membro atingiu o limite de empréstimos".to_owned(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn sample() -> Member {
        Member {
            id: 101,
            name: "João Silva".into(),
            email: "joao@email.com".into(),
            phone: None,
            status: MemberStatus::Ativo,
            loan_limit: 3,
            registration_date: NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
        }
    }

    #[test]
    fn regras_para_emprestar() {
        let member = |status, loan_limit| Member {
            status,
            loan_limit,
            ..sample()
        };
        assert!(member(MemberStatus::Ativo, 3).ensure_can_borrow(2).is_ok());
        for (status, limit, open) in [
            (MemberStatus::Suspenso, 3, 0),
            (MemberStatus::Inativo, 3, 0),
            (MemberStatus::Ativo, 3, 3),
            (MemberStatus::Ativo, 0, 0),
        ] {
            assert!(matches!(
                member(status, limit).ensure_can_borrow(open),
                Err(AppError::Validation(_))
            ));
        }
    }

    #[test]
    fn status_segue_o_contrato() {
        assert_eq!(json!(MemberStatus::Suspenso), json!("SUSPENSO"));
    }
}
