-- Your SQL goes here
CREATE TABLE posts (
  id SERIAL PRIMARY KEY,
  publish_tx_id CHAR(64) NOT NULL unique,
  user_address CHAR(40) NOT NULL,
  file_hash CHAR(64) NOT NULL,
  topic CHAR(40) NOT NULL,
  url VARCHAR(255) NOT NULL,
  update_by_tx_id CHAR(64) NOT NULL DEFAULT '',
  updated_at timestamp NOT NULL
)
