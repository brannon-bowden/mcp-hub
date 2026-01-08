-- Migration 003: Add content hash for integrity verification
-- Adds a content_hash column to custom_registries for verifying cached data integrity

ALTER TABLE custom_registries ADD COLUMN content_hash TEXT;
