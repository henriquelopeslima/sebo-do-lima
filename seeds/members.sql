-- Dados de exemplo para desenvolvimento; também usado como fixture nos testes.
-- Só insere com a tabela vazia, então pode ser executado mais de uma vez.
INSERT INTO members (name, email, phone, status, loan_limit, registration_date)
SELECT * FROM (VALUES
    ('João Silva',      'joao@email.com',     '(11) 99999-9999', 'ATIVO',    3, DATE '2026-01-15'),
    ('Maria Oliveira',  'maria@email.com',    '(21) 98888-8888', 'ATIVO',    5, DATE '2026-03-02'),
    ('Pedro Santos',    'pedro@email.com',    NULL,              'ATIVO',    3, DATE '2026-06-20'),
    ('Ana Costa',       'ana@email.com',      '(31) 97777-7777', 'SUSPENSO', 3, DATE '2026-02-10'),
    ('Carlos Pereira',  'carlos@email.com',   NULL,              'INATIVO',  3, DATE '2025-09-05')
) AS seed (name, email, phone, status, loan_limit, registration_date)
WHERE NOT EXISTS (SELECT 1 FROM members);
