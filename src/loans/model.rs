//! Espelha o schema `Loan` de docs/contract-api.yml.

use chrono::NaiveDate;
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, sqlx::Type)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[sqlx(type_name = "text", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LoanStatus {
    Aberto,
    Finalizado,
    Atrasado,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
#[expect(
    clippy::struct_field_names,
    reason = "loanDate é o nome do campo no contrato"
)]
pub struct Loan {
    pub id: i64,
    pub book_id: i64,
    pub member_id: i64,
    pub loan_date: NaiveDate,
    pub due_date: NaiveDate,
    pub return_date: Option<NaiveDate>,
    pub status: LoanStatus,
    // shortcut: f64 porque o contrato define double; trocar por decimal se a multa passar a ser somada/cobrada de verdade
    pub fine: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn json_segue_o_contrato() {
        let loan = Loan {
            id: 990,
            book_id: 55,
            member_id: 101,
            loan_date: NaiveDate::from_ymd_opt(2026, 11, 12).unwrap(),
            due_date: NaiveDate::from_ymd_opt(2026, 11, 19).unwrap(),
            return_date: None,
            status: LoanStatus::Aberto,
            fine: 0.0,
        };
        assert_eq!(
            serde_json::to_value(loan).unwrap(),
            json!({
                "id": 990, "bookId": 55, "memberId": 101,
                "loanDate": "2026-11-12", "dueDate": "2026-11-19",
                "returnDate": null, "status": "ABERTO", "fine": 0.0
            })
        );
    }
}
