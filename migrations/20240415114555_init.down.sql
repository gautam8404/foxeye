-- Add down migration script here
DROP INDEX IF EXISTS idx_url;
DROP TABLE IF EXISTS crawler_queue;
DROP TABLE IF EXISTS chunk;
DROP TABLE IF EXISTS document;
DROP EXTENSION vector;
