-- Add up migration script here
CREATE TABLE IF NOT EXISTS urls (
    url_id BIGSERIAL PRIMARY KEY,
    url TEXT NOT NULL,
    source_doc BIGSERIAL,
    created_at TIMESTAMP DEFAULT now(),
    updated_at TIMESTAMP DEFAULT now()
);

CREATE TABLE IF NOT EXISTS crawler_queue (
    url_id BIGSERIAL PRIMARY KEY ,
    url TEXT NOT NULL,
    depth INT, -- depth
    source_doc BIGSERIAL,
    created_at TIMESTAMP DEFAULT now(),
    updated_at TIMESTAMP DEFAULT now()
);

CREATE TABLE IF NOT EXISTS document (
    doc_id BIGSERIAL PRIMARY KEY ,
    url TEXT NOT NULL ,
    content TEXT,
    created_at TIMESTAMP DEFAULT now(),
    updated_at TIMESTAMP DEFAULT now()
);

-- page links
-- doc id, urls one to many
-- [doc_id] primary key
-- get all documents
-- loop, urls

CREATE TABLE IF NOT EXISTS terms (
    term_id BIGSERIAL PRIMARY KEY,
    term TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS inverted_index (
    term_id BIGSERIAL,
    doc_id BIGSERIAL,
    term_frequency INT,
    position INT, -- Optional for phrase search
    PRIMARY KEY (term_id, doc_id)
);

ALTER TABLE inverted_index ADD FOREIGN KEY (term_id) REFERENCES terms(term_id);
ALTER TABLE inverted_index ADD FOREIGN KEY (doc_id) REFERENCES document(doc_id);
ALTER TABLE urls ADD FOREIGN KEY (source_doc) REFERENCES document(doc_id);
ALTER TABLE crawler_queue ADD FOREIGN KEY (source_doc) REFERENCES document(doc_id);

CREATE INDEX IF NOT EXISTS idx_url ON urls (url);
CREATE INDEX IF NOT EXISTS term_idx ON terms (term);
