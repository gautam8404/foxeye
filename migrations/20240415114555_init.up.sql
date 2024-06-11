-- Add up migration script here
CREATE EXTENSION IF NOT EXISTS vector;

CREATE TABLE IF NOT EXISTS crawler_queue (
    url_id BIGSERIAL PRIMARY KEY ,
    url TEXT NOT NULL UNIQUE ,
    depth INT NOT NULL , -- depth
    created_at TIMESTAMP DEFAULT now(),
    updated_at TIMESTAMP DEFAULT now()
);

CREATE TABLE IF NOT EXISTS document (
    doc_id BIGSERIAL PRIMARY KEY ,
    url TEXT NOT NULL,
    content TEXT,
    created_at TIMESTAMP DEFAULT now(),
    updated_at TIMESTAMP DEFAULT now()
);

CREATE TABLE IF NOT EXISTS chunk (
    chunk_id BIGSERIAL PRIMARY KEY ,
    doc_id BIGSERIAL,
    chunk_data TEXT,
    embedding vector(1024)
);

-- page links
-- doc id, urls one to many
-- [doc_id] primary key
-- get all documents
-- loop, urls
ALTER TABLE chunk ADD FOREIGN KEY (doc_id) REFERENCES document(doc_id);

CREATE INDEX IF NOT EXISTS idx_url ON crawler_queue (url);
