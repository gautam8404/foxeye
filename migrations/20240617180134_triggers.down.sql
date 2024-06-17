-- Add down migration script here

-- Drop index
DROP INDEX IF EXISTS idx_doc_id;

-- Drop triggers
DROP TRIGGER IF EXISTS set_timestamp_crawler_queue ON crawler_queue;
DROP TRIGGER IF EXISTS set_timestamp_document ON document;
DROP TRIGGER IF EXISTS set_timestamp_chunk ON chunk;

-- Drop trigger function
DROP FUNCTION IF EXISTS update_updated_at_column;

