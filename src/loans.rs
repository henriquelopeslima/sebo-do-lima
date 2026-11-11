mod dto;
mod handlers;
mod repository;

use axum::{
    Router,
    routing::{get, post},
};
use sqlx::PgPool;

pub fn router() -> Router<PgPool> {
    Router::new()
        .route("/", post(handlers::create))
        .route("/overdue", get(handlers::overdue))
        .route("/{id}/return", post(handlers::return_loan))
}
