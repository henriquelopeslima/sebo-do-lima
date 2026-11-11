CREATE TABLE books (
    id           BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    title        TEXT NOT NULL,
    -- sem UNIQUE: cada linha é um exemplar, e exemplares do mesmo título repetem o ISBN
    isbn         TEXT NOT NULL,
    author       TEXT NOT NULL,
    category     TEXT NOT NULL,
    status       TEXT NOT NULL DEFAULT 'DISPONIVEL'
                 CHECK (status IN ('DISPONIVEL', 'EMPRESTADO', 'MANUTENCAO', 'PERDIDO')),
    publish_year INTEGER,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);
