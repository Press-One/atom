-- CREATE TYPE block_status AS ENUM ('SUBMITTED', 'CONFIRMED');

CREATE TABLE blocks (
    id SERIAL PRIMARY KEY,
    block_id VARCHAR NOT NULL UNIQUE,
    block_num bigint NOT NULL UNIQUE,
    block_type VARCHAR NOT NULL,
    block_timestamp VARCHAR NOT NULL,
    created_at timestamp NOT NULL default current_timestamp,
    updated_at timestamp
)
