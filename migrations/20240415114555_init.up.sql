-- Add up migration script here
CREATE EXTENSION IF NOT EXISTS vector;

CREATE TABLE IF NOT EXISTS crawler_queue (
    url_id BIGSERIAL PRIMARY KEY ,
    url TEXT NOT NULL UNIQUE ,
    host TEXT NOT NULL ,
    depth INT NOT NULL , -- depth
    created_at TIMESTAMP DEFAULT now(),
    updated_at TIMESTAMP DEFAULT now()
);

CREATE TABLE IF NOT EXISTS document (
    doc_id VARCHAR(128) PRIMARY KEY ,
    url TEXT NOT NULL UNIQUE ,
    content TEXT,
    created_at TIMESTAMP DEFAULT now(),
    updated_at TIMESTAMP DEFAULT now()
);

CREATE TABLE IF NOT EXISTS chunk (
    chunk_id VARCHAR(128) PRIMARY KEY ,
    doc_id VARCHAR(128),
    chunk_start BIGINT,
    chunk_end BIGINT,
    embedding vector(1024),
    created_at TIMESTAMP DEFAULT now(),
    updated_at TIMESTAMP DEFAULT now()
);

-- page links
-- doc id, urls one to many
-- [doc_id] primary key
-- get all documents
-- loop, urls
ALTER TABLE chunk ADD FOREIGN KEY (doc_id) REFERENCES document(doc_id);

CREATE INDEX IF NOT EXISTS idx_url ON crawler_queue (url);
