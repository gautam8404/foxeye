-- Add down migration script here
DROP INDEX IF EXISTS term_idx;
DROP INDEX IF EXISTS idx_url;
DROP TABLE IF EXISTS inverted_index;
DROP TABLE IF EXISTS terms;
DROP TABLE IF EXISTS urls;
DROP TABLE IF EXISTS crawler_queue;
DROP TABLE IF EXISTS chunk;
DROP TABLE IF EXISTS document;
DROP EXTENSION vector;
