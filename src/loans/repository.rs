use chrono::NaiveDate;
use sqlx::PgPool;

use super::dto::{FINE_PER_DAY, LoanCreate, LoanReturn};
use crate::{
    books::model::BookStatus, error::AppError, loans::model::Loan, members::model::MemberStatus,
};

fn invalid(message: &str) -> AppError {
    AppError::Validation(message.to_owned())
}

/// Cria o empréstimo e marca o livro como emprestado, na mesma transação.
pub async fn create(pool: &PgPool, input: &LoanCreate) -> Result<Loan, AppError> {
    let days = input.days_to_return()?;
    let mut tx = pool.begin().await?;

    // FOR UPDATE serializa empréstimos concorrentes do mesmo membro (limite) e do mesmo livro
    let member: Option<(MemberStatus, i32)> =
        sqlx::query_as("SELECT status, loan_limit FROM members WHERE id = $1 FOR UPDATE")
            .bind(input.member_id)
            .fetch_optional(&mut *tx)
            .await?;
    let Some((member_status, loan_limit)) = member else {
        return Err(invalid("Membro não encontrado"));
    };
    if member_status != MemberStatus::Ativo {
        return Err(invalid("Membro não está ativo"));
    }
    let open_loans: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM loans WHERE member_id = $1 AND return_date IS NULL",
    )
    .bind(input.member_id)
    .fetch_one(&mut *tx)
    .await?;
    if open_loans >= i64::from(loan_limit) {
        return Err(invalid("Membro atingiu o limite de empréstimos"));
    }

    let book_status: Option<BookStatus> =
        sqlx::query_scalar("SELECT status FROM books WHERE id = $1 FOR UPDATE")
            .bind(input.book_id)
            .fetch_optional(&mut *tx)
            .await?;
    match book_status {
        None => return Err(invalid("Livro não encontrado")),
        Some(BookStatus::Disponivel) => {}
        Some(_) => return Err(invalid("Livro indisponível")),
    }

    let loan = sqlx::query_as(
        "INSERT INTO loans (book_id, member_id, due_date)
         VALUES ($1, $2, CURRENT_DATE + $3)
         RETURNING *",
    )
    .bind(input.book_id)
    .bind(input.member_id)
    .bind(days)
    .fetch_one(&mut *tx)
    .await?;
    sqlx::query("UPDATE books SET status = 'EMPRESTADO', updated_at = now() WHERE id = $1")
        .bind(input.book_id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;
    Ok(loan)
}

/// Finaliza o empréstimo, calcula a multa e libera o livro, na mesma transação.
pub async fn return_loan(pool: &PgPool, id: i64) -> Result<LoanReturn, AppError> {
    let mut tx = pool.begin().await?;

    // `return_date IS NULL` no WHERE torna a devolução atômica: só uma requisição concorrente finaliza
    let returned: Option<(i64, NaiveDate, f64, i32)> = sqlx::query_as(
        "UPDATE loans SET
             status      = 'FINALIZADO',
             return_date = CURRENT_DATE,
             fine        = GREATEST(CURRENT_DATE - due_date, 0) * $2
         WHERE id = $1 AND return_date IS NULL
         RETURNING book_id, return_date, fine, GREATEST(return_date - due_date, 0)",
    )
    .bind(id)
    .bind(FINE_PER_DAY)
    .fetch_optional(&mut *tx)
    .await?;
    let Some((book_id, return_date, fine, days_late)) = returned else {
        let exists: bool = sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM loans WHERE id = $1)")
            .bind(id)
            .fetch_one(&mut *tx)
            .await?;
        return Err(if exists {
            AppError::Conflict("Empréstimo já devolvido")
        } else {
            AppError::NotFound("Empréstimo não encontrado")
        });
    };

    // só libera se ainda consta como emprestado: não desfaz um PERDIDO/MANUTENCAO marcado nesse meio-tempo
    sqlx::query(
        "UPDATE books SET status = 'DISPONIVEL', updated_at = now()
         WHERE id = $1 AND status = 'EMPRESTADO'",
    )
    .bind(book_id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(LoanReturn::new(id, return_date, fine, days_late))
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

pub async fn overdue(pool: &PgPool) -> sqlx::Result<Vec<Loan>> {
    sqlx::query_as(
        "SELECT * FROM loans_with_status WHERE status = 'ATRASADO' ORDER BY due_date, id",
    )
    .fetch_all(pool)
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::loans::model::LoanStatus;

    async fn member_id(pool: &PgPool, email: &str) -> sqlx::Result<i64> {
        sqlx::query_scalar("SELECT id FROM members WHERE email = $1")
            .bind(email)
            .fetch_one(pool)
            .await
    }

    async fn available_books(pool: &PgPool) -> sqlx::Result<Vec<i64>> {
        sqlx::query_scalar("SELECT id FROM books WHERE status = 'DISPONIVEL' ORDER BY id")
            .fetch_all(pool)
            .await
    }

    async fn book_status(pool: &PgPool, id: i64) -> sqlx::Result<BookStatus> {
        sqlx::query_scalar("SELECT status FROM books WHERE id = $1")
            .bind(id)
            .fetch_one(pool)
            .await
    }

    fn loan(member_id: i64, book_id: i64) -> LoanCreate {
        LoanCreate {
            member_id,
            book_id,
            days_to_return: None,
        }
    }

    // garante que o seed continua válido quando o schema mudar
    #[sqlx::test(fixtures(path = "../../seeds", scripts("books", "members", "loans")))]
    async fn seed_de_emprestimos_e_valido(pool: PgPool) -> sqlx::Result<()> {
        let (loans, lent_books): (i64, i64) = sqlx::query_as(
            "SELECT (SELECT count(*) FROM loans),
                    (SELECT count(*) FROM books WHERE status = 'EMPRESTADO')",
        )
        .fetch_one(&pool)
        .await?;
        assert_eq!((loans, lent_books), (4, 2));
        assert_eq!(overdue(&pool).await?.len(), 1);
        Ok(())
    }

    #[sqlx::test(fixtures(path = "../../seeds", scripts("books", "members")))]
    async fn emprestimo_e_devolucao_em_dia(pool: PgPool) -> Result<(), AppError> {
        let joao = member_id(&pool, "joao@email.com").await?;
        let maria = member_id(&pool, "maria@email.com").await?;
        let book = available_books(&pool).await?[0];

        let created = create(&pool, &loan(joao, book)).await?;
        assert_eq!(created.status, LoanStatus::Aberto);
        assert_eq!((created.due_date - created.loan_date).num_days(), 7);
        assert_eq!(book_status(&pool, book).await?, BookStatus::Emprestado);
        assert!(matches!(
            create(&pool, &loan(maria, book)).await,
            Err(AppError::Validation(_))
        ));

        let returned = return_loan(&pool, created.id).await?;
        assert_eq!(returned.fine, 0.0);
        assert_eq!(returned.return_date, created.loan_date);
        assert_eq!(book_status(&pool, book).await?, BookStatus::Disponivel);

        assert!(matches!(
            return_loan(&pool, created.id).await,
            Err(AppError::Conflict(_))
        ));
        assert!(matches!(
            return_loan(&pool, created.id + 1).await,
            Err(AppError::NotFound(_))
        ));
        Ok(())
    }

    #[sqlx::test(fixtures(path = "../../seeds", scripts("books", "members")))]
    async fn historico_de_emprestimos(pool: PgPool) -> Result<(), AppError> {
        let joao = member_id(&pool, "joao@email.com").await?;
        let maria = member_id(&pool, "maria@email.com").await?;
        assert!(list_by_member(&pool, joao).await?.is_empty());

        create(&pool, &loan(joao, available_books(&pool).await?[0])).await?;

        let history = list_by_member(&pool, joao).await?;
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].status, LoanStatus::Aberto);
        assert_eq!(history[0].return_date, None);
        assert!(list_by_member(&pool, maria).await?.is_empty());
        Ok(())
    }

    #[sqlx::test(fixtures(path = "../../seeds", scripts("books", "members")))]
    async fn regras_do_membro(pool: PgPool) -> Result<(), AppError> {
        let books = available_books(&pool).await?;
        let rejected = async |member: i64, book: i64| {
            matches!(
                create(&pool, &loan(member, book)).await,
                Err(AppError::Validation(_))
            )
        };

        let suspended = member_id(&pool, "ana@email.com").await?;
        assert!(rejected(suspended, books[0]).await);
        assert!(rejected(i64::MAX, books[0]).await);

        // João tem limite de 3 empréstimos em aberto
        let joao = member_id(&pool, "joao@email.com").await?;
        for &book in &books[..3] {
            create(&pool, &loan(joao, book)).await?;
        }
        assert!(rejected(joao, books[3]).await);
        // a recusa não pode deixar o livro marcado como emprestado
        assert_eq!(book_status(&pool, books[3]).await?, BookStatus::Disponivel);
        Ok(())
    }

    #[sqlx::test(fixtures(path = "../../seeds", scripts("books", "members")))]
    async fn atraso_gera_multa(pool: PgPool) -> Result<(), AppError> {
        let joao = member_id(&pool, "joao@email.com").await?;
        let books = available_books(&pool).await?;
        let late = create(&pool, &loan(joao, books[0])).await?;
        create(&pool, &loan(joao, books[1])).await?;
        assert!(overdue(&pool).await?.is_empty());

        sqlx::query("UPDATE loans SET due_date = CURRENT_DATE - 3 WHERE id = $1")
            .bind(late.id)
            .execute(&pool)
            .await?;
        let overdue_loans = overdue(&pool).await?;
        assert_eq!(overdue_loans.len(), 1);
        assert_eq!(overdue_loans[0].id, late.id);
        assert_eq!(overdue_loans[0].status, LoanStatus::Atrasado);

        let returned = return_loan(&pool, late.id).await?;
        assert_eq!(returned.fine, 3.0 * FINE_PER_DAY);
        assert!(returned.message.contains("3 dia(s)"));
        assert!(overdue(&pool).await?.is_empty());
        Ok(())
    }
}
