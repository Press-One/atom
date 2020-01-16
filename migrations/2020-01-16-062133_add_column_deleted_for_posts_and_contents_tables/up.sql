ALTER TABLE posts RENAME update_by_tx_id TO updated_tx_id;

ALTER TABLE contents ADD COLUMN updated_at timestamp NOT NULL DEFAULT now();
ALTER TABLE contents ADD COLUMN deleted BOOLEAN NOT NULL DEFAULT 'f';

ALTER TABLE posts ADD COLUMN deleted BOOLEAN NOT NULL DEFAULT 'f';

CREATE INDEX idx_posts_publish_tx_id ON posts(publish_tx_id);
CREATE INDEX idx_contents_deleted ON contents(deleted);
CREATE INDEX idx_posts_deleted ON posts(deleted);
