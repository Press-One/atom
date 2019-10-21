-- Your SQL goes here
ALTER TABLE transactions ADD COLUMN trx_id VARCHAR NOT NULL default '';
ALTER TABLE transactions ADD COLUMN signature VARCHAR NOT NULL default '';
ALTER TABLE transactions ADD COLUMN hash VARCHAR NOT NULL default '';
ALTER TABLE transactions ADD COLUMN user_address VARCHAR NOT NULL default '';
