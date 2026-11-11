-- ATRASADO não é gravado: é derivado da data, para não depender de um job que atualize a tabela.
-- Toda leitura de empréstimos deve passar por esta view.
CREATE VIEW loans_with_status AS
SELECT id, book_id, member_id, loan_date, due_date, return_date,
       CASE WHEN return_date IS NULL AND due_date < CURRENT_DATE THEN 'ATRASADO' ELSE status END AS status,
       fine
FROM loans;
