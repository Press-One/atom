-- Your SQL goes here
CREATE TABLE last_status(
    id SERIAL PRIMARY KEY,
    key VARCHAR NOT NULL,
    val bigint NOT NULL
);

DROP TABLE last_block_nums;
