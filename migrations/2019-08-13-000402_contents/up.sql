-- Your SQL goes here
CREATE TABLE contents(
  file_hash CHAR(64) NOT NULL PRIMARY KEY,
  url VARCHAR(255) NOT NULL,
  content TEXT NOT NULL,
  created_at timestamp DEFAULT now()
)
