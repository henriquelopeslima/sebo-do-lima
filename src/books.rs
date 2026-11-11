mod dto;
mod handlers;
pub(crate) mod model;
pub(crate) mod repository;
mod service;

use axum::{Router, routing::get};
use sqlx::PgPool;

pub fn router() -> Router<PgPool> {
    Router::new()
        .route("/", get(handlers::list).post(handlers::create))
        .route(
            "/{id}",
            get(handlers::find)
                .put(handlers::update)
                .delete(handlers::delete),
        )
}
