mod books;
mod entities;
mod error;
mod loans;
mod members;
mod validation;

use axum::{Router, routing::get};
use sqlx::PgPool;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let database_url = std::env::var("DATABASE_URL").map_err(|_| "DATABASE_URL não definida")?;
    let pool = PgPool::connect(&database_url).await?;
    sqlx::migrate!().run(&pool).await?;

    let api = Router::new()
        .route("/hello", get(async || "Hello, world!"))
        .nest("/books", books::router())
        .nest("/members", members::router())
        .nest("/loans", loans::router());
    let app = Router::new().nest("/api", api).with_state(pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;
    Ok(())
}
