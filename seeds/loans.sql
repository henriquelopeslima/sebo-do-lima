-- Dados de exemplo para desenvolvimento; também usado como fixture nos testes.
-- Depende de books.sql e members.sql. Só insere com a tabela vazia, então pode ser executado mais de uma vez.
-- Datas relativas a hoje, para que o empréstimo atrasado continue atrasado em qualquer dia.
WITH seed (email, isbn, loan_date, due_date, return_date, status, fine) AS (VALUES
    -- em aberto, dentro do prazo
    ('joao@email.com',  '978-85-359-0278-5', CURRENT_DATE - 3,  CURRENT_DATE + 4,  NULL::date,        'ABERTO',     0::float8),
    -- em aberto e vencido: aparece como ATRASADO
    ('maria@email.com', '978-85-359-0279-2', CURRENT_DATE - 12, CURRENT_DATE - 5,  NULL,              'ABERTO',     0),
    -- devolvido em dia
    ('joao@email.com',  '978-85-359-0280-8', CURRENT_DATE - 30, CURRENT_DATE - 23, CURRENT_DATE - 25, 'FINALIZADO', 0),
    -- devolvido com 3 dias de atraso (multa de R$ 2,00 por dia)
    ('pedro@email.com', '978-85-359-0282-2', CURRENT_DATE - 40, CURRENT_DATE - 33, CURRENT_DATE - 30, 'FINALIZADO', 6)
),
inserted AS (
    INSERT INTO loans (book_id, member_id, loan_date, due_date, return_date, status, fine)
    SELECT b.id, m.id, s.loan_date, s.due_date, s.return_date, s.status, s.fine
    FROM seed s
    JOIN members m ON m.email = s.email
    JOIN books b ON b.id = (SELECT min(id) FROM books WHERE isbn = s.isbn AND status = 'DISPONIVEL')
    WHERE NOT EXISTS (SELECT 1 FROM loans)
    RETURNING book_id, return_date
)
-- livro com empréstimo em aberto fica EMPRESTADO, como a API faria
UPDATE books SET status = 'EMPRESTADO', updated_at = now()
WHERE id IN (SELECT book_id FROM inserted WHERE return_date IS NULL);
