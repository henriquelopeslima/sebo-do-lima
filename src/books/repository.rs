use sqlx::PgPool;

use super::dto::{BookCreate, BookFilter, BookUpdate};
use crate::books::model::Book;

pub async fn list(pool: &PgPool, filter: &BookFilter) -> sqlx::Result<Vec<Book>> {
    // strpos em vez de ILIKE: '%' e '_' digitados na busca não viram curingas
    sqlx::query_as(
        "SELECT * FROM books
         WHERE ($1::text IS NULL OR status = $1)
           AND ($2::text IS NULL OR category = $2)
           AND ($3::text IS NULL
                OR strpos(lower(title), lower($3)) > 0
                OR strpos(lower(author), lower($3)) > 0)
         ORDER BY id",
    )
    .bind(filter.status)
    .bind(&filter.category)
    .bind(&filter.search)
    .fetch_all(pool)
    .await
}

pub async fn find(pool: &PgPool, id: i64) -> sqlx::Result<Option<Book>> {
    sqlx::query_as("SELECT * FROM books WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub async fn create(pool: &PgPool, input: &BookCreate) -> sqlx::Result<Book> {
    sqlx::query_as(
        "INSERT INTO books (title, isbn, author, category, publish_year)
         VALUES ($1, $2, $3, $4, $5)
         RETURNING *",
    )
    .bind(&input.title)
    .bind(&input.isbn)
    .bind(&input.author)
    .bind(&input.category)
    .bind(input.publish_year)
    .fetch_one(pool)
    .await
}

/// Atualiza só os campos enviados; `None` se o livro não existe.
pub async fn update(pool: &PgPool, id: i64, input: &BookUpdate) -> sqlx::Result<Option<Book>> {
    // shortcut: COALESCE não permite limpar publish_year para NULL; distinguir "ausente" de "null" se isso for necessário
    sqlx::query_as(
        "UPDATE books SET
             title        = COALESCE($2, title),
             isbn         = COALESCE($3, isbn),
             author       = COALESCE($4, author),
             category     = COALESCE($5, category),
             status       = COALESCE($6, status),
             publish_year = COALESCE($7, publish_year),
             updated_at   = now()
         WHERE id = $1
         RETURNING *",
    )
    .bind(id)
    .bind(&input.title)
    .bind(&input.isbn)
    .bind(&input.author)
    .bind(&input.category)
    .bind(input.status)
    .bind(input.publish_year)
    .fetch_optional(pool)
    .await
}

/// `false` se o livro não existe.
pub async fn delete(pool: &PgPool, id: i64) -> sqlx::Result<bool> {
    let result = sqlx::query("DELETE FROM books WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::books::model::BookStatus;

    fn new_book(title: &str, author: &str, category: &str) -> BookCreate {
        BookCreate {
            title: title.into(),
            isbn: "978-85-359-0277-8".into(),
            author: author.into(),
            category: category.into(),
            publish_year: Some(1899),
        }
    }

    #[sqlx::test]
    async fn ciclo_de_vida_do_livro(pool: PgPool) -> sqlx::Result<()> {
        let book = create(
            &pool,
            &new_book("Dom Casmurro", "Machado de Assis", "Ficção"),
        )
        .await?;
        assert_eq!(book.status, BookStatus::Disponivel);
        assert_eq!(find(&pool, book.id).await?.unwrap().title, "Dom Casmurro");

        let changes = BookUpdate {
            status: Some(BookStatus::Manutencao),
            ..Default::default()
        };
        let updated = update(&pool, book.id, &changes).await?.unwrap();
        assert_eq!(updated.status, BookStatus::Manutencao);
        assert_eq!(updated.title, "Dom Casmurro");
        assert_eq!(updated.publish_year, Some(1899));

        assert!(delete(&pool, book.id).await?);
        assert!(!delete(&pool, book.id).await?);
        assert!(find(&pool, book.id).await?.is_none());
        assert!(update(&pool, book.id, &changes).await?.is_none());
        Ok(())
    }

    // garante que o seed continua válido quando o schema mudar
    #[sqlx::test(fixtures(path = "../../seeds", scripts("books")))]
    async fn seed_de_livros_e_valido(pool: PgPool) -> sqlx::Result<()> {
        assert_eq!(list(&pool, &BookFilter::default()).await?.len(), 10);
        Ok(())
    }

    #[sqlx::test]
    async fn filtros_da_listagem(pool: PgPool) -> sqlx::Result<()> {
        let casmurro = create(
            &pool,
            &new_book("Dom Casmurro", "Machado de Assis", "Ficção"),
        )
        .await?;
        create(&pool, &new_book("100% Rust", "Ferris", "Técnico")).await?;
        let lent = BookUpdate {
            status: Some(BookStatus::Emprestado),
            ..Default::default()
        };
        update(&pool, casmurro.id, &lent).await?;

        let titles = async |filter: BookFilter| -> sqlx::Result<Vec<String>> {
            let books = list(&pool, &filter).await?;
            Ok(books.into_iter().map(|book| book.title).collect())
        };
        let search = |term: &str| BookFilter {
            search: Some(term.into()),
            ..Default::default()
        };

        assert_eq!(titles(BookFilter::default()).await?.len(), 2);
        assert_eq!(titles(search("machado")).await?, ["Dom Casmurro"]);
        assert_eq!(titles(search("CASMURRO")).await?, ["Dom Casmurro"]);
        assert_eq!(titles(search("%")).await?, ["100% Rust"]);
        assert_eq!(
            titles(BookFilter {
                status: Some(BookStatus::Emprestado),
                ..Default::default()
            })
            .await?,
            ["Dom Casmurro"]
        );
        assert_eq!(
            titles(BookFilter {
                category: Some("Técnico".into()),
                ..Default::default()
            })
            .await?,
            ["100% Rust"]
        );
        Ok(())
    }
}
