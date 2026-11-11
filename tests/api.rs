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

async fn get(pool: &PgPool, uri: &str) -> (StatusCode, Value) {
    request(pool, Method::GET, uri, None).await
}

async fn post(pool: &PgPool, uri: &str, body: Value) -> (StatusCode, Value) {
    request(pool, Method::POST, uri, Some(body)).await
}

fn id_of(value: &Value) -> i64 {
    value["id"].as_i64().unwrap()
}

async fn member_id(pool: &PgPool, email: &str) -> i64 {
    let (_, members) = get(pool, "/api/members").await;
    let member = members
        .as_array()
        .unwrap()
        .iter()
        .find(|member| member["email"] == email)
        .unwrap();
    id_of(member)
}

/// Ids dos livros com o status dado, em ordem crescente.
async fn book_ids(pool: &PgPool, status: &str) -> Vec<i64> {
    let (_, books) = get(pool, &format!("/api/books?status={status}")).await;
    books.as_array().unwrap().iter().map(id_of).collect()
}

#[sqlx::test]
async fn livros_crud(pool: PgPool) {
    let (status, book) = post(
        &pool,
        "/api/books",
        json!({
            "title": "Dom Casmurro", "isbn": "978-85-359-0277-8",
            "author": "Machado de Assis", "category": "Ficção", "publishYear": 1899
        }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(book["status"], "DISPONIVEL");
    assert_eq!(book["publishYear"], 1899);
    let uri = format!("/api/books/{}", id_of(&book));

    assert_eq!(get(&pool, &uri).await.0, StatusCode::OK);

    let (status, updated) = request(
        &pool,
        Method::PUT,
        &uri,
        Some(json!({ "status": "MANUTENCAO" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(updated["status"], "MANUTENCAO");
    assert_eq!(updated["title"], "Dom Casmurro");

    let (status, found) = get(&pool, "/api/books?search=machado").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(found.as_array().unwrap().len(), 1);

    let (status, body) = request(&pool, Method::DELETE, &uri, None).await;
    assert_eq!((status, body), (StatusCode::NO_CONTENT, Value::Null));
    let (status, body) = request(&pool, Method::DELETE, &uri, None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"], "Livro não encontrado");
    assert_eq!(get(&pool, &uri).await.0, StatusCode::NOT_FOUND);
    let (status, _) = request(&pool, Method::PUT, &uri, Some(json!({}))).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

// Review Focus 5: entrada inválida é 400, nunca o 422 padrão do axum
#[sqlx::test]
async fn livros_entradas_invalidas(pool: PgPool) {
    let (status, _) = post(&pool, "/api/books", json!({ "title": "x" })).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    let (status, body) = post(
        &pool,
        "/api/books",
        json!({ "title": " ", "isbn": "1", "author": "a", "category": "c" }),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"], "title não pode ser vazio");

    let (status, _) = post(&pool, "/api/books", json!("não é objeto")).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    let (status, _) = request(
        &pool,
        Method::PUT,
        "/api/books/1",
        Some(json!({ "title": "" })),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    for uri in ["/api/books?status=FOO", "/api/books/abc"] {
        assert_eq!(get(&pool, uri).await.0, StatusCode::BAD_REQUEST, "{uri}");
    }
}

#[sqlx::test(fixtures(path = "../seeds", scripts("books", "members", "loans")))]
async fn livro_com_emprestimo_nao_e_removido(pool: PgPool) {
    let lent = book_ids(&pool, "EMPRESTADO").await[0];
    let (status, body) = request(&pool, Method::DELETE, &format!("/api/books/{lent}"), None).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["error"], "Livro possui empréstimos registrados");
}

#[sqlx::test]
async fn membros(pool: PgPool) {
    let (status, member) = post(
        &pool,
        "/api/members",
        json!({ "name": "João Silva", "email": "joao@email.com" }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(member["status"], "ATIVO");
    assert_eq!(member["loanLimit"], 3);
    assert_eq!(member["phone"], Value::Null);
    let uri = format!("/api/members/{}", id_of(&member));

    let invalid = [
        (
            json!({ "name": "Outro", "email": "JOAO@email.com" }),
            Some("email já cadastrado"),
        ),
        (
            json!({ "name": "Outro", "email": "joao" }),
            Some("email inválido"),
        ),
        (
            json!({ "name": "Outro", "email": "o@e.com", "loanLimit": -1 }),
            None,
        ),
        (json!({ "name": "Outro" }), None),
    ];
    for (payload, error) in invalid {
        let (status, body) = post(&pool, "/api/members", payload.clone()).await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{payload}");
        if let Some(error) = error {
            assert_eq!(body["error"], error);
        }
    }

    let (status, members) = get(&pool, "/api/members").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(members.as_array().unwrap().len(), 1);
    assert_eq!(get(&pool, &uri).await.0, StatusCode::OK);

    let (status, body) = get(&pool, "/api/members/999999").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"], "Membro não encontrado");

    assert_eq!(
        get(&pool, &format!("{uri}/loans")).await,
        (StatusCode::OK, json!([]))
    );
    assert_eq!(
        get(&pool, "/api/members/999999/loans").await.0,
        StatusCode::NOT_FOUND
    );
}

#[sqlx::test(fixtures(path = "../seeds", scripts("books", "members", "loans")))]
async fn historico_mostra_atrasado(pool: PgPool) {
    let maria = member_id(&pool, "maria@email.com").await;
    let (status, loans) = get(&pool, &format!("/api/members/{maria}/loans")).await;
    assert_eq!(status, StatusCode::OK);
    let loans = loans.as_array().unwrap();
    assert_eq!(loans.len(), 1);
    assert_eq!(loans[0]["status"], "ATRASADO");
}

#[sqlx::test(fixtures(path = "../seeds", scripts("books", "members")))]
async fn emprestimo_e_devolucao(pool: PgPool) {
    let joao = member_id(&pool, "joao@email.com").await;
    let maria = member_id(&pool, "maria@email.com").await;
    let ana = member_id(&pool, "ana@email.com").await;
    let available = book_ids(&pool, "DISPONIVEL").await;
    let (book, other) = (available[0], available[1]);
    let book_uri = format!("/api/books/{book}");

    let (status, loan) = post(
        &pool,
        "/api/loans",
        json!({ "memberId": joao, "bookId": book }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(loan["status"], "ABERTO");
    assert_eq!(loan["returnDate"], Value::Null);
    assert_eq!(loan["fine"], 0.0);
    assert_eq!(get(&pool, &book_uri).await.1["status"], "EMPRESTADO");

    let rejected = [
        (
            json!({ "memberId": maria, "bookId": book }),
            Some("Livro indisponível"),
        ),
        (
            json!({ "memberId": ana, "bookId": other }),
            Some("Membro não está ativo"),
        ),
        (
            json!({ "memberId": 999_999, "bookId": other }),
            Some("Membro não encontrado"),
        ),
        (
            json!({ "memberId": joao, "bookId": 999_999 }),
            Some("Livro não encontrado"),
        ),
        (
            json!({ "memberId": joao, "bookId": other, "daysToReturn": 0 }),
            None,
        ),
        (json!({ "memberId": joao }), None),
    ];
    for (payload, error) in rejected {
        let (status, body) = post(&pool, "/api/loans", payload.clone()).await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{payload}");
        if let Some(error) = error {
            assert_eq!(body["error"], error);
        }
    }

    let id = id_of(&loan);
    let return_uri = format!("/api/loans/{id}/return");
    let (status, returned) = request(&pool, Method::POST, &return_uri, None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(returned["loanId"], id);
    assert_eq!(returned["status"], "FINALIZADO");
    assert_eq!(returned["fine"], 0.0);
    assert_eq!(returned["message"], "Devolução realizada sem multa");
    assert_eq!(get(&pool, &book_uri).await.1["status"], "DISPONIVEL");

    let (status, body) = request(&pool, Method::POST, &return_uri, None).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["error"], "Empréstimo já devolvido");

    let (status, body) = request(&pool, Method::POST, "/api/loans/999999/return", None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"], "Empréstimo não encontrado");
}

#[sqlx::test(fixtures(path = "../seeds", scripts("books", "members", "loans")))]
async fn atrasados_e_multa(pool: PgPool) {
    let (status, overdue) = get(&pool, "/api/loans/overdue").await;
    assert_eq!(status, StatusCode::OK);
    let overdue = overdue.as_array().unwrap();
    assert_eq!(overdue.len(), 1);
    assert_eq!(overdue[0]["status"], "ATRASADO");

    // o seed vence esse empréstimo 5 dias atrás; a multa é de R$ 2,00 por dia
    let uri = format!("/api/loans/{}/return", id_of(&overdue[0]));
    let (status, returned) = request(&pool, Method::POST, &uri, None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(returned["fine"], 10.0);
    assert!(returned["message"].as_str().unwrap().contains("5 dia(s)"));

    assert_eq!(
        get(&pool, "/api/loans/overdue").await,
        (StatusCode::OK, json!([]))
    );
}
