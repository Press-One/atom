-- CREATE TYPE status AS ENUM ('SUBMITTED', 'CONFIRMED');
CREATE TABLE transactions (
    id SERIAL PRIMARY KEY,
    block_num bigint REFERENCES blocks(block_num) NOT NULL,
    data_type VARCHAR NOT NULL,
    data VARCHAR NOT NULL,
    status VARCHAR NOT NULL DEFAULT 'SUBMITTED',
    created_at timestamp NOT NULL default current_timestamp,
    updated_at timestamp
)
