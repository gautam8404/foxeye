-- Add up migration script here
ALTER TABLE chunk DROP CONSTRAINT chunk_doc_id_fkey;
ALTER TABLE chunk ADD CONSTRAINT chunk_doc_id_fkey FOREIGN KEY (doc_id) REFERENCES document(doc_id) ON DELETE CASCADE;