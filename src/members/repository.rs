use sqlx::PgPool;

use super::dto::{DEFAULT_LOAN_LIMIT, MemberCreate};
use crate::{loans::model::Loan, members::model::Member};

pub async fn list(pool: &PgPool) -> sqlx::Result<Vec<Member>> {
    sqlx::query_as("SELECT * FROM members ORDER BY id")
        .fetch_all(pool)
        .await
}

pub async fn find(pool: &PgPool, id: i64) -> sqlx::Result<Option<Member>> {
    sqlx::query_as("SELECT * FROM members WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
}

/// Falha com violação de unicidade se o e-mail já está cadastrado.
pub async fn create(pool: &PgPool, input: &MemberCreate) -> sqlx::Result<Member> {
    sqlx::query_as(
        "INSERT INTO members (name, email, phone, loan_limit)
         VALUES ($1, $2, $3, $4)
         RETURNING *",
    )
    .bind(&input.name)
    .bind(&input.email)
    .bind(&input.phone)
    .bind(input.loan_limit.unwrap_or(DEFAULT_LOAN_LIMIT))
    .fetch_one(pool)
    .await
}

/// Histórico de empréstimos do membro, do mais recente para o mais antigo.
pub async fn loans(pool: &PgPool, member_id: i64) -> sqlx::Result<Vec<Loan>> {
    sqlx::query_as(
        "SELECT * FROM loans_with_status WHERE member_id = $1 ORDER BY loan_date DESC, id DESC",
    )
    .bind(member_id)
    .fetch_all(pool)
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{loans::model::LoanStatus, members::model::MemberStatus};

    fn new_member(email: &str) -> MemberCreate {
        MemberCreate {
            name: "João Silva".into(),
            email: email.into(),
            phone: None,
            loan_limit: None,
        }
    }

    #[sqlx::test]
    async fn cadastro_e_consulta(pool: PgPool) -> sqlx::Result<()> {
        let member = create(&pool, &new_member("joao@email.com")).await?;
        assert_eq!(member.status, MemberStatus::Ativo);
        assert_eq!(member.loan_limit, DEFAULT_LOAN_LIMIT);

        assert_eq!(
            find(&pool, member.id).await?.unwrap().email,
            "joao@email.com"
        );
        assert!(find(&pool, member.id + 1).await?.is_none());
        assert_eq!(list(&pool).await?.len(), 1);
        Ok(())
    }

    // garante que o seed continua válido quando o schema mudar
    #[sqlx::test(fixtures(path = "../../seeds", scripts("members")))]
    async fn seed_de_membros_e_valido(pool: PgPool) -> sqlx::Result<()> {
        assert_eq!(list(&pool).await?.len(), 5);
        Ok(())
    }

    #[sqlx::test]
    async fn email_duplicado_ignora_maiusculas(pool: PgPool) -> sqlx::Result<()> {
        create(&pool, &new_member("joao@email.com")).await?;
        let error = create(&pool, &new_member("JOAO@email.com"))
            .await
            .unwrap_err();
        assert!(error.as_database_error().unwrap().is_unique_violation());
        Ok(())
    }

    #[sqlx::test(fixtures(path = "../../seeds", scripts("books")))]
    async fn historico_de_emprestimos(pool: PgPool) -> sqlx::Result<()> {
        let member = create(&pool, &new_member("joao@email.com")).await?;
        let other = create(&pool, &new_member("maria@email.com")).await?;
        assert!(loans(&pool, member.id).await?.is_empty());

        sqlx::query(
            "INSERT INTO loans (book_id, member_id, loan_date, due_date)
             SELECT id, $1, CURRENT_DATE, CURRENT_DATE + 7 FROM books ORDER BY id LIMIT 1",
        )
        .bind(member.id)
        .execute(&pool)
        .await?;

        let history = loans(&pool, member.id).await?;
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].status, LoanStatus::Aberto);
        assert_eq!(history[0].return_date, None);
        assert!(loans(&pool, other.id).await?.is_empty());
        Ok(())
    }
}
