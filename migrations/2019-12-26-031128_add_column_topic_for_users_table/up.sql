ALTER TABLE users ADD COLUMN topic VARCHAR NOT NULL default '';
CREATE INDEX idx_users_topic ON users(topic);
