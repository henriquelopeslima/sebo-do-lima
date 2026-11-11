//! Testes HTTP: exercitam o router completo contra um banco real.

use axum::{
    body::Body,
    http::{Method, Request, StatusCode, header},
};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use sqlx::PgPool;
use tower::ServiceExt;

/// Corpo vazio vira `Value::Null`; corpo que não é JSON vira `Value::String`.
async fn request(
    pool: &PgPool,
    method: Method,
    uri: &str,
    body: Option<Value>,
) -> (StatusCode, Value) {
    let builder = Request::builder().method(method).uri(uri);
    let request = match body {
        Some(body) => builder
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(body.to_string())),
        None => builder.body(Body::empty()),
    }
    .unwrap();

    let response = sebo_do_lima::app(pool.clone())
        .oneshot(request)
        .await
        .unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes)
            .unwrap_or_else(|_| Value::String(String::from_utf8_lossy(&bytes).into_owned()))
    };
    (status, body)
}

#[sqlx::test]
async fn hello(pool: PgPool) {
    let (status, body) = request(&pool, Method::GET, "/api/hello", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, json!("Hello, world!"));
}
