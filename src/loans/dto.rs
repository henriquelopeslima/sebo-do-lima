use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::{error::AppError, loans::model::LoanStatus};

pub const DEFAULT_DAYS_TO_RETURN: i32 = 7;
const MAX_DAYS_TO_RETURN: i32 = 365;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoanCreate {
    pub member_id: i64,
    pub book_id: i64,
    pub days_to_return: Option<i32>,
}

impl LoanCreate {
    /// Prazo em dias, já com o padrão aplicado e validado.
    pub fn days_to_return(&self) -> Result<i32, AppError> {
        let days = self.days_to_return.unwrap_or(DEFAULT_DAYS_TO_RETURN);
        if (1..=MAX_DAYS_TO_RETURN).contains(&days) {
            Ok(days)
        } else {
            Err(AppError::Validation(format!(
                "daysToReturn deve estar entre 1 e {MAX_DAYS_TO_RETURN}"
            )))
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoanReturn {
    pub loan_id: i64,
    pub status: LoanStatus,
    pub return_date: NaiveDate,
    pub fine: f64,
    pub message: String,
}

impl LoanReturn {
    pub fn new(loan_id: i64, return_date: NaiveDate, fine: f64, days_late: i64) -> Self {
        let message = if days_late > 0 {
            format!("Devolução realizada com {days_late} dia(s) de atraso; multa de R$ {fine:.2}")
        } else {
            "Devolução realizada sem multa".to_owned()
        };
        Self {
            loan_id,
            status: LoanStatus::Finalizado,
            return_date,
            fine,
            message,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prazo_padrao_e_limites() {
        let loan = |days_to_return| LoanCreate {
            member_id: 1,
            book_id: 1,
            days_to_return,
        };
        assert_eq!(loan(None).days_to_return().unwrap(), DEFAULT_DAYS_TO_RETURN);
        assert_eq!(loan(Some(30)).days_to_return().unwrap(), 30);
        assert!(loan(Some(0)).days_to_return().is_err());
        assert!(loan(Some(MAX_DAYS_TO_RETURN + 1)).days_to_return().is_err());
    }
}
