ALTER TABLE blocks ADD CONSTRAINT unique_blocks_block_num UNIQUE (block_num);
ALTER TABLE blocks ADD CONSTRAINT unique_blocks_block_id UNIQUE (block_id);
ALTER TABLE transactions ADD CONSTRAINT unique_transactions_trx_id UNIQUE (trx_id);
ALTER TABLE notifies ADD CONSTRAINT unique_notifies_trx_id UNIQUE (trx_id);
