CREATE TABLE members (
    id                BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    name              TEXT NOT NULL,
    email             TEXT NOT NULL,
    phone             TEXT,
    status            TEXT NOT NULL DEFAULT 'ATIVO'
                      CHECK (status IN ('ATIVO', 'SUSPENSO', 'INATIVO')),
    loan_limit        INTEGER NOT NULL CHECK (loan_limit >= 0),
    registration_date DATE NOT NULL DEFAULT CURRENT_DATE
);

-- um cadastro por e-mail, sem diferenciar maiúsculas
CREATE UNIQUE INDEX members_email_key ON members (lower(email));
