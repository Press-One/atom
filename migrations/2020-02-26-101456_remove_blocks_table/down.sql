CREATE TABLE blocks (
    id SERIAL PRIMARY KEY,
    block_id VARCHAR NOT NULL UNIQUE,
    block_num bigint NOT NULL UNIQUE,
    block_type VARCHAR NOT NULL,
    block_timestamp VARCHAR NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone
);
ALTER TABLE transactions ADD CONSTRAINT transactions_block_num_fkey FOREIGN KEY (block_num) REFERENCES blocks(block_num);
