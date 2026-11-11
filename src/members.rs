mod dto;
mod handlers;
mod repository;

use axum::{Router, routing::get};
use sqlx::PgPool;

pub fn router() -> Router<PgPool> {
    Router::new()
        .route("/", get(handlers::list).post(handlers::create))
        .route("/{id}", get(handlers::find))
        .route("/{id}/loans", get(handlers::loans))
}
