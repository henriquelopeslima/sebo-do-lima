use chrono::NaiveDate;
use sqlx::{PgConnection, PgPool};

use super::model::Loan;

/// Data de hoje segundo o banco: a mesma referência usada pela view `loans_with_status`.
pub async fn today(conn: &mut PgConnection) -> sqlx::Result<NaiveDate> {
    sqlx::query_scalar("SELECT CURRENT_DATE")
        .fetch_one(conn)
        .await
}

pub async fn count_open_by_member(conn: &mut PgConnection, member_id: i64) -> sqlx::Result<i64> {
    sqlx::query_scalar("SELECT count(*) FROM loans WHERE member_id = $1 AND return_date IS NULL")
        .bind(member_id)
        .fetch_one(conn)
        .await
}

pub async fn insert(
    conn: &mut PgConnection,
    book_id: i64,
    member_id: i64,
    days: i32,
) -> sqlx::Result<Loan> {
    sqlx::query_as(
        "INSERT INTO loans (book_id, member_id, due_date)
         VALUES ($1, $2, CURRENT_DATE + $3)
         RETURNING *",
    )
    .bind(book_id)
    .bind(member_id)
    .bind(days)
    .fetch_one(conn)
    .await
}

/// Bloqueia o empréstimo até o fim da transação.
pub async fn find_for_update(conn: &mut PgConnection, id: i64) -> sqlx::Result<Option<Loan>> {
    sqlx::query_as("SELECT * FROM loans WHERE id = $1 FOR UPDATE")
        .bind(id)
        .fetch_optional(conn)
        .await
}

pub async fn finish(
    conn: &mut PgConnection,
    id: i64,
    return_date: NaiveDate,
    fine: f64,
) -> sqlx::Result<()> {
    sqlx::query(
        "UPDATE loans SET status = 'FINALIZADO', return_date = $2, fine = $3 WHERE id = $1",
    )
    .bind(id)
    .bind(return_date)
    .bind(fine)
    .execute(conn)
    .await?;
    Ok(())
}

/// Histórico de empréstimos do membro, do mais recente para o mais antigo.
pub async fn list_by_member(pool: &PgPool, member_id: i64) -> sqlx::Result<Vec<Loan>> {
    sqlx::query_as(
        "SELECT * FROM loans_with_status WHERE member_id = $1 ORDER BY loan_date DESC, id DESC",
    )
    .bind(member_id)
    .fetch_all(pool)
    .await
}

pub async fn list_overdue(pool: &PgPool) -> sqlx::Result<Vec<Loan>> {
    sqlx::query_as(
        "SELECT * FROM loans_with_status WHERE status = 'ATRASADO' ORDER BY due_date, id",
    )
    .fetch_all(pool)
    .await
}
