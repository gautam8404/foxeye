-- Add up migration script here

-- create index on doc_id
CREATE INDEX IF NOT EXISTS idx_doc_id ON chunk (doc_id);

-- function to update updated_at automatically
CREATE OR REPLACE FUNCTION update_updated_at_column()
    RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = now();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER set_timestamp_crawler_queue
    BEFORE UPDATE ON crawler_queue
    FOR EACH ROW
EXECUTE PROCEDURE update_updated_at_column();

CREATE TRIGGER set_timestamp_document
    BEFORE UPDATE ON document
    FOR EACH ROW
EXECUTE PROCEDURE update_updated_at_column();

CREATE TRIGGER set_timestamp_chunk
    BEFORE UPDATE ON chunk
    FOR EACH ROW
EXECUTE PROCEDURE update_updated_at_column();


