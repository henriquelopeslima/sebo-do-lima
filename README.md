# Sebo do Lima

API REST de gestão de biblioteca/sebo: catálogo de livros, cadastro de membros e empréstimos com cálculo de multa por atraso.

## Stack

| Ferramenta | Uso |
|---|---|
| Rust (edição 2024) | Linguagem; `unsafe` proibido, Clippy `pedantic` |
| [Axum](https://github.com/tokio-rs/axum) + Tokio | Servidor HTTP assíncrono |
| SQLx + PostgreSQL 18 | Acesso a dados, consultas verificadas e migrations (rodam na subida da API) |
| Docker Compose | Ambiente de desenvolvimento (API, banco, seed, Swagger UI) |
| GitHub Actions | `lint` (rustfmt + clippy) e `coverage` (cargo-llvm-cov, mínimo de 95% de linhas) |

## Executando

```sh
# opcional: cp .env.example .env (porta, credenciais do banco, UID/GID)
docker compose up                 # API em http://127.0.0.1:3000 (cria as tabelas)
docker compose run --rm seed      # dados de exemplo (com a API já no ar)
docker compose up docs            # Swagger UI em http://127.0.0.1:8080
```

Sem Docker: defina `DATABASE_URL` (obrigatória) e `PORT` (opcional, padrão `3000`), como no `.env.example`, e rode `cargo run`.

Testes (precisam de um Postgres acessível via `DATABASE_URL`, pois usam `#[sqlx::test]`):

```sh
cargo test
cargo llvm-cov --fail-under-lines 95
```

## Negócio

**Livros** — cada linha é um exemplar (o mesmo ISBN pode se repetir). Status: `DISPONIVEL`, `EMPRESTADO`, `MANUTENCAO`, `PERDIDO`.

**Membros** — e-mail único (sem diferenciar maiúsculas). Status: `ATIVO`, `SUSPENSO`, `INATIVO`. Cada membro tem um `loan_limit` de empréstimos simultâneos.

**Empréstimos**
- Só membros `ATIVO` que não atingiram o limite podem emprestar, e só livros `DISPONIVEL`.
- Prazo padrão de 7 dias (`daysToReturn`, de 1 a 365).
- Criar e devolver rodam em transação com bloqueio de linhas, evitando empréstimos ou devoluções duplicadas em concorrência. O livro vira `EMPRESTADO` ao emprestar e volta a `DISPONIVEL` ao devolver.
- Multa de R$ 2,00 por dia de atraso, calculada na devolução.
- `ATRASADO` não é gravado: é derivado de `due_date` na view `loans_with_status`, sem depender de job.

## Endpoints

| Rota | Descrição |
|---|---|
| `GET /hello` | Verificação simples |
| `GET, POST /books` | Lista (com filtros) e cria livros |
| `GET, PUT, DELETE /books/{id}` | Consulta, atualiza e remove um livro |
| `GET, POST /members` | Lista e cadastra membros |
| `GET /members/{id}` | Consulta um membro |
| `GET /members/{id}/loans` | Histórico de empréstimos do membro |
| `POST /loans` | Realiza um empréstimo |
| `POST /loans/{id}/return` | Devolve e calcula a multa |
| `GET /loans/overdue` | Empréstimos em atraso |

Contrato completo (OpenAPI): [`docs/contract-api.yml`](docs/contract-api.yml).

## Observabilidade

- **Logs:** JSON em stdout (`tracing`), uma linha por requisição e uma por erro de banco. Nível via `RUST_LOG` (padrão `info`).
- **Métricas:** `GET /metrics` (Prometheus) com `http_requests_total` e `http_request_duration_seconds`, rotuladas por método, rota e status.
- **SLO:** disponibilidade de 99,5% em 30 dias, em [`docs/slo.md`](docs/slo.md).

## Estrutura

```
src/
  books/ members/ loans/   # cada módulo: handlers → service → repository, mais model e dto
  config.rs                # variáveis de ambiente
  error.rs                 # AppError → resposta HTTP
  validation.rs            # validação de campos de texto
migrations/                # schema SQL
seeds/                     # dados de exemplo
tests/api.rs               # testes de integração da API
```
