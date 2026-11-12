//! Métricas HTTP no formato Prometheus: contador e histograma de latência por rota.

use std::{sync::OnceLock, time::Instant};

use axum::{
    extract::{MatchedPath, Request},
    middleware::Next,
    response::Response,
};
use metrics::{counter, histogram};
use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};

static HANDLE: OnceLock<PrometheusHandle> = OnceLock::new();

/// Instala o recorder global (uma única vez). Sem ele, o middleware não registra nada.
///
/// # Panics
///
/// Se outro código já instalou um recorder de métricas no processo.
pub fn install() {
    HANDLE.get_or_init(|| {
        PrometheusBuilder::new()
            .install_recorder()
            .expect("recorder de métricas já instalado por outro código")
    });
}

pub(crate) async fn render() -> String {
    HANDLE
        .get()
        .map(PrometheusHandle::render)
        .unwrap_or_default()
}

/// Conta a requisição e mede a duração. Usa o padrão da rota (`/api/books/{id}`), não a URI.
pub(crate) async fn track(request: Request, next: Next) -> Response {
    let method = request.method().to_string();
    let route = request.extensions().get::<MatchedPath>().map_or_else(
        || "desconhecida".to_owned(),
        |path| path.as_str().to_owned(),
    );
    let start = Instant::now();

    let response = next.run(request).await;

    let labels = [
        ("method", method),
        ("route", route),
        ("status", response.status().as_u16().to_string()),
    ];
    counter!("http_requests_total", &labels).increment(1);
    histogram!("http_request_duration_seconds", &labels).record(start.elapsed().as_secs_f64());
    response
}
