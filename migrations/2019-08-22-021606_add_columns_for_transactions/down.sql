-- This file should undo anything in `up.sql`
ALTER TABLE transactions DROP COLUMN trx_id;
ALTER TABLE transactions DROP COLUMN signature;
ALTER TABLE transactions DROP COLUMN hash;
ALTER TABLE transactions DROP COLUMN user_address;
