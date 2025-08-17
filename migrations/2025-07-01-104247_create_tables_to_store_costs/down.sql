-- This file should undo anything in `up.sql`
DROP TABLE payments_quotas;
DROP TABLE payments;
DROP TABLE quotas;

ALTER TABLE costs DROP COLUMN group_id;
ALTER TABLE costs DROP COLUMN user_id;
ALTER TABLE costs DROP COLUMN amount;