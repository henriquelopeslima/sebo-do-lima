use crate::error::AppError;

pub fn require_text(field: &str, value: &str) -> Result<(), AppError> {
    if value.trim().is_empty() {
        return Err(AppError::Validation(format!("{field} não pode ser vazio")));
    }
    Ok(())
}
