//! API da biblioteca: livros, membros e empréstimos.
//!
//! Cada funcionalidade (`books`, `members`, `loans`) é um módulo dividido em camadas:
//!
//! | Camada       | Pode                                                             | Não pode                              |
//! |--------------|------------------------------------------------------------------|---------------------------------------|
//! | `handlers`   | extrair dados da requisição, chamar o `service`, escolher status | SQL, regra de negócio                 |
//! | `service`    | validar, aplicar regras, abrir transação, chamar repositórios    | tipos do axum, SQL                    |
//! | `repository` | SQL                                                              | regra de negócio, `AppError`          |
//! | `model`      | tipos e funções puras                                            | I/O                                   |
//!
//! Um módulo só acessa a tabela de outro através do `repository` desse outro módulo.

mod books;
pub mod config;
mod error;
mod loans;
mod members;
mod validation;

use axum::{Router, routing::get};
use sqlx::PgPool;
use tokio::signal::unix::{SignalKind, signal};
use tower_http::trace::TraceLayer;

use config::Config;

/// Router completo da API, com todas as rotas sob `/api`.
pub fn app(pool: PgPool) -> Router {
    let api = Router::new()
        .route("/hello", get(async || "Hello, world!"))
        .nest("/books", books::router())
        .nest("/members", members::router())
        .nest("/loans", loans::router());
    Router::new()
        .nest("/api", api)
        .layer(TraceLayer::new_for_http())
        .with_state(pool)
}

/// Conecta ao banco, aplica as migrations e serve a API até receber SIGTERM ou Ctrl-C.
///
/// # Errors
///
/// Falha se o banco está inacessível, uma migration não aplica ou a porta não pode ser aberta.
pub async fn run(config: Config) -> Result<(), Box<dyn std::error::Error>> {
    let pool = PgPool::connect(&config.database_url).await?;
    sqlx::migrate!().run(&pool).await?;

    let listener = tokio::net::TcpListener::bind(("0.0.0.0", config.port)).await?;
    tracing::info!(port = config.port, "servidor iniciado");
    let mut terminate = signal(SignalKind::terminate())?;
    axum::serve(listener, app(pool))
        .with_graceful_shutdown(async move {
            tokio::select! {
                _ = tokio::signal::ctrl_c() => {}
                _ = terminate.recv() => {}
            }
        })
        .await?;
    Ok(())
}
