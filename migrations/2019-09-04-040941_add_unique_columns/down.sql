ALTER TABLE blocks DROP CONSTRAINT unique_blocks_block_num;
ALTER TABLE blocks DROP CONSTRAINT unique_blocks_block_id;
ALTER TABLE transactions DROP CONSTRAINT unique_transactions_trx_id;
ALTER TABLE notifies DROP CONSTRAINT unique_notifies_trx_id;
