-- Criada aqui porque GET /members/{id}/loans lê o histórico; as rotas de /loans vêm depois.
CREATE TABLE loans (
    id          BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    book_id     BIGINT NOT NULL REFERENCES books (id),
    member_id   BIGINT NOT NULL REFERENCES members (id),
    loan_date   DATE NOT NULL DEFAULT CURRENT_DATE,
    due_date    DATE NOT NULL,
    return_date DATE,
    status      TEXT NOT NULL DEFAULT 'ABERTO'
                CHECK (status IN ('ABERTO', 'FINALIZADO', 'ATRASADO')),
    fine        DOUBLE PRECISION NOT NULL DEFAULT 0
);

CREATE INDEX loans_member_id_idx ON loans (member_id);
