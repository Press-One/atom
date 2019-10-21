CREATE TABLE last_block_nums (
    id SERIAL PRIMARY KEY,
    block_num bigint NOT NULL UNIQUE
)
