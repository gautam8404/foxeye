-- Add up migration script here
CREATE INDEX IF NOT EXISTS idx_host ON crawler_queue(host);
CREATE INDEX IF NOT EXISTS idx_doc_url ON document(url);
