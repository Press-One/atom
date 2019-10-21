-- Your SQL goes here

CREATE TABLE notifies (
    data_id VARCHAR NOT NULL PRIMARY KEY,
    block_num bigint NOT NULL,
    trx_id VARCHAR NOT NULL,
    success Boolean NOT NULL default FALSE,
    retries int NOT NULL default 0,
    created_at timestamp NOT NULL default current_timestamp,
    updated_at timestamp
)
