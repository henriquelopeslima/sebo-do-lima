use sqlx::PgPool;

use super::{
    dto::{LoanCreate, LoanReturn},
    model::{Loan, fine_for},
    repository,
};
use crate::{books, error::AppError, members};

fn invalid(message: &str) -> AppError {
    AppError::Validation(message.to_owned())
}

/// Cria o empréstimo e marca o livro como emprestado, na mesma transação.
pub async fn create(pool: &PgPool, input: &LoanCreate) -> Result<Loan, AppError> {
    let days = input.days_to_return()?;
    let mut tx = pool.begin().await?;

    // Os bloqueios (membro, depois livro) serializam empréstimos concorrentes do mesmo membro
    // ou do mesmo livro. Sair com erro descarta a transação e desfaz tudo.
    let member = members::repository::find_for_update(&mut tx, input.member_id)
        .await?
        .ok_or_else(|| invalid("Membro não encontrado"))?;
    let open_loans = repository::count_open_by_member(&mut tx, member.id).await?;
    member.ensure_can_borrow(open_loans)?;

    let book = books::repository::find_for_update(&mut tx, input.book_id)
        .await?
        .ok_or_else(|| invalid("Livro não encontrado"))?;
    book.ensure_available()?;

    let loan = repository::insert(&mut tx, book.id, member.id, days).await?;
    books::repository::mark_lent(&mut tx, book.id).await?;

    tx.commit().await?;
    Ok(loan)
}

/// Finaliza o empréstimo, calcula a multa e libera o livro, na mesma transação.
pub async fn return_loan(pool: &PgPool, id: i64) -> Result<LoanReturn, AppError> {
    let mut tx = pool.begin().await?;

    // o bloqueio faz a segunda de duas devoluções simultâneas enxergar a primeira já concluída
    let loan = repository::find_for_update(&mut tx, id)
        .await?
        .ok_or(AppError::NotFound("Empréstimo não encontrado"))?;
    if loan.return_date.is_some() {
        return Err(AppError::Conflict("Empréstimo já devolvido"));
    }

    let today = repository::today(&mut tx).await?;
    let (days_late, fine) = fine_for(loan.due_date, today);
    repository::finish(&mut tx, id, today, fine).await?;
    books::repository::release(&mut tx, loan.book_id).await?;

    tx.commit().await?;
    Ok(LoanReturn::new(id, today, fine, days_late))
}

pub async fn overdue(pool: &PgPool) -> Result<Vec<Loan>, AppError> {
    Ok(repository::list_overdue(pool).await?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        books::model::BookStatus,
        loans::model::{FINE_PER_DAY, LoanStatus},
    };

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
    async fn seed_de_emprestimos_e_valido(pool: PgPool) -> Result<(), AppError> {
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
        assert!(repository::list_by_member(&pool, joao).await?.is_empty());

        create(&pool, &loan(joao, available_books(&pool).await?[0])).await?;

        let history = repository::list_by_member(&pool, joao).await?;
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].status, LoanStatus::Aberto);
        assert_eq!(history[0].return_date, None);
        assert!(repository::list_by_member(&pool, maria).await?.is_empty());
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
        // Review Focus 3: a recusa não pode deixar rastro
        assert_eq!(book_status(&pool, books[3]).await?, BookStatus::Disponivel);
        let loans_of_book: i64 =
            sqlx::query_scalar("SELECT count(*) FROM loans WHERE book_id = $1")
                .bind(books[3])
                .fetch_one(&pool)
                .await?;
        assert_eq!(loans_of_book, 0);
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

    // Review Focus 1: o bloqueio do livro precisa valer para toda a transação do caso de uso
    #[sqlx::test(fixtures(path = "../../seeds", scripts("books", "members")))]
    async fn emprestimos_simultaneos_do_mesmo_livro(pool: PgPool) -> Result<(), AppError> {
        let joao = member_id(&pool, "joao@email.com").await?;
        let maria = member_id(&pool, "maria@email.com").await?;
        let book = available_books(&pool).await?[0];
        let (first, second) = (loan(joao, book), loan(maria, book));
        let (a, b) = tokio::join!(create(&pool, &first), create(&pool, &second));
        assert_eq!(u8::from(a.is_ok()) + u8::from(b.is_ok()), 1);
        Ok(())
    }

    // Review Focus 2
    #[sqlx::test(fixtures(path = "../../seeds", scripts("books", "members")))]
    async fn devolucoes_simultaneas(pool: PgPool) -> Result<(), AppError> {
        let joao = member_id(&pool, "joao@email.com").await?;
        let created = create(&pool, &loan(joao, available_books(&pool).await?[0])).await?;
        let (a, b) = tokio::join!(
            return_loan(&pool, created.id),
            return_loan(&pool, created.id)
        );
        assert_eq!(u8::from(a.is_ok()) + u8::from(b.is_ok()), 1);
        assert!(
            [a, b]
                .into_iter()
                .any(|result| matches!(result, Err(AppError::Conflict(_))))
        );
        Ok(())
    }

    // Review Focus 4
    #[sqlx::test(fixtures(path = "../../seeds", scripts("books", "members")))]
    async fn devolucao_nao_ressuscita_livro_perdido(pool: PgPool) -> Result<(), AppError> {
        let joao = member_id(&pool, "joao@email.com").await?;
        let book = available_books(&pool).await?[0];
        let created = create(&pool, &loan(joao, book)).await?;
        sqlx::query("UPDATE books SET status = 'PERDIDO' WHERE id = $1")
            .bind(book)
            .execute(&pool)
            .await?;
        return_loan(&pool, created.id).await?;
        assert_eq!(book_status(&pool, book).await?, BookStatus::Perdido);
        Ok(())
    }
}
