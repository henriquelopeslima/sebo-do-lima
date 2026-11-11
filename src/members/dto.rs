use serde::Deserialize;

use crate::{error::AppError, validation::require_text};

pub const DEFAULT_LOAN_LIMIT: i32 = 3;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemberCreate {
    pub name: String,
    pub email: String,
    pub phone: Option<String>,
    pub loan_limit: Option<i32>,
}

impl MemberCreate {
    pub fn validate(&self) -> Result<(), AppError> {
        require_text("name", &self.name)?;
        if !is_email(&self.email) {
            return Err(AppError::Validation("email inválido".into()));
        }
        if self.loan_limit.is_some_and(|limit| limit < 0) {
            return Err(AppError::Validation(
                "loanLimit não pode ser negativo".into(),
            ));
        }
        Ok(())
    }
}

// shortcut: só confere o formato algo@algo, sem espaços; usar confirmação por e-mail se a entrega importar
fn is_email(value: &str) -> bool {
    !value.contains(char::is_whitespace)
        && value.split_once('@').is_some_and(|(local, domain)| {
            !local.is_empty() && !domain.is_empty() && !domain.contains('@')
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valida_email() {
        assert!(is_email("joao@email.com"));
        for invalid in [
            "",
            "joao",
            "@email.com",
            "joao@",
            "jo ao@email.com",
            "a@b@c",
        ] {
            assert!(!is_email(invalid), "{invalid:?} deveria ser inválido");
        }
    }

    #[test]
    fn rejeita_limite_negativo() {
        let member = MemberCreate {
            name: "João Silva".into(),
            email: "joao@email.com".into(),
            phone: None,
            loan_limit: Some(-1),
        };
        assert!(matches!(member.validate(), Err(AppError::Validation(_))));
    }
}
