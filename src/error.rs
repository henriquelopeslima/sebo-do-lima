use axum::{
    Json,
    extract::rejection::{JsonRejection, QueryRejection},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;

#[derive(Debug)]
pub enum AppError {
    NotFound(&'static str),
    Validation(String),
    Database(sqlx::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Self::NotFound(message) => (StatusCode::NOT_FOUND, message.to_owned()),
            Self::Validation(message) => (StatusCode::BAD_REQUEST, message),
            Self::Database(error) => {
                // o detalhe fica só no log: mensagens do banco não devem vazar para o cliente
                eprintln!("erro de banco de dados: {error}");
                (StatusCode::INTERNAL_SERVER_ERROR, "Erro interno".to_owned())
            }
        };
        (status, Json(json!({ "error": message }))).into_response()
    }
}

impl From<sqlx::Error> for AppError {
    fn from(error: sqlx::Error) -> Self {
        Self::Database(error)
    }
}

// O padrão do axum para corpo/query inválidos é 422; o contrato pede 400.
impl From<JsonRejection> for AppError {
    fn from(rejection: JsonRejection) -> Self {
        Self::Validation(rejection.body_text())
    }
}

impl From<QueryRejection> for AppError {
    fn from(rejection: QueryRejection) -> Self {
        Self::Validation(rejection.body_text())
    }
}
