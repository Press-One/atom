-- This file should undo anything in `up.sql`
ALTER TABLE contents ALTER COLUMN created_at DROP NOT NULL;
