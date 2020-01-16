ALTER TABLE posts RENAME updated_tx_id TO update_by_tx_id;

ALTER TABLE contents DROP COLUMN IF EXISTS updated_at;
ALTER TABLE contents DROP COLUMN IF EXISTS deleted;

ALTER TABLE posts DROP COLUMN IF EXISTS deleted;

DROP INDEX IF EXISTS idx_posts_publish_tx_id;
DROP INDEX IF EXISTS idx_contents_deleted;
DROP INDEX IF EXISTS idx_posts_deleted;
