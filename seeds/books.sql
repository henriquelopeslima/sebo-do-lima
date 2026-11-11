-- Dados de exemplo para desenvolvimento; também usado como fixture nos testes.
-- Só insere com a tabela vazia, então pode ser executado mais de uma vez.
-- Nenhum livro EMPRESTADO: esse status só faz sentido com um empréstimo registrado.
INSERT INTO books (title, isbn, author, category, status, publish_year)
SELECT * FROM (VALUES
    ('Dom Casmurro',                     '978-85-359-0277-8', 'Machado de Assis',   'Ficção',    'DISPONIVEL', 1899),
    ('Dom Casmurro',                     '978-85-359-0277-8', 'Machado de Assis',   'Ficção',    'MANUTENCAO', 1899),
    ('Memórias Póstumas de Brás Cubas',  '978-85-359-0278-5', 'Machado de Assis',   'Ficção',    'DISPONIVEL', 1881),
    ('Grande Sertão: Veredas',           '978-85-359-0279-2', 'Guimarães Rosa',     'Ficção',    'DISPONIVEL', 1956),
    ('A Hora da Estrela',                '978-85-359-0280-8', 'Clarice Lispector',  'Ficção',    'DISPONIVEL', 1977),
    ('Vidas Secas',                      '978-85-359-0281-5', 'Graciliano Ramos',   'Ficção',    'PERDIDO',    1938),
    ('Raízes do Brasil',                 '978-85-359-0282-2', 'Sérgio Buarque de Holanda', 'História', 'DISPONIVEL', 1936),
    ('Casa-Grande & Senzala',            '978-85-359-0283-9', 'Gilberto Freyre',    'História',  'DISPONIVEL', 1933),
    ('Alguma Poesia',                    '978-85-359-0284-6', 'Carlos Drummond de Andrade', 'Poesia', 'DISPONIVEL', 1930),
    ('The Rust Programming Language',    '978-85-359-0285-3', 'Steve Klabnik e Carol Nichols', 'Técnico', 'DISPONIVEL', NULL)
) AS seed (title, isbn, author, category, status, publish_year)
WHERE NOT EXISTS (SELECT 1 FROM books);
