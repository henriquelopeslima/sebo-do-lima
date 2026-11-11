use serde::Deserialize;

use crate::{entities::BookStatus, error::AppError};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BookCreate {
    pub title: String,
    pub isbn: String,
    pub author: String,
    pub category: String,
    pub publish_year: Option<i32>,
}

impl BookCreate {
    pub fn validate(&self) -> Result<(), AppError> {
        require_text("title", &self.title)?;
        require_text("isbn", &self.isbn)?;
        require_text("author", &self.author)?;
        require_text("category", &self.category)
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BookUpdate {
    pub title: Option<String>,
    pub isbn: Option<String>,
    pub author: Option<String>,
    pub category: Option<String>,
    pub status: Option<BookStatus>,
    pub publish_year: Option<i32>,
}

impl BookUpdate {
    pub fn validate(&self) -> Result<(), AppError> {
        let fields = [
            ("title", &self.title),
            ("isbn", &self.isbn),
            ("author", &self.author),
            ("category", &self.category),
        ];
        fields
            .into_iter()
            .filter_map(|(field, value)| Some((field, value.as_deref()?)))
            .try_for_each(|(field, value)| require_text(field, value))
    }
}

#[derive(Debug, Default, Deserialize)]
pub struct BookFilter {
    pub status: Option<BookStatus>,
    pub category: Option<String>,
    pub search: Option<String>,
}

fn require_text(field: &str, value: &str) -> Result<(), AppError> {
    if value.trim().is_empty() {
        return Err(AppError::Validation(format!("{field} não pode ser vazio")));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejeita_texto_em_branco() {
        let blank_title = BookUpdate {
            title: Some("  ".into()),
            ..Default::default()
        };
        assert!(matches!(
            blank_title.validate(),
            Err(AppError::Validation(_))
        ));
        assert!(BookUpdate::default().validate().is_ok());
    }
}
