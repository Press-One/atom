-- Your SQL goes here
CREATE TABLE users (
  user_address CHAR(40) PRIMARY KEY,
  status CHAR(10) NOT NULL,
  tx_id CHAR(64) NOT NULL,
  updated_at timestamp NOT NULL
)
