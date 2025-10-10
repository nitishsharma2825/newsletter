-- Add migration script here
-- we wrap the whole migration in a transaction
BEGIN;
    -- Backfill status for historical entries
    UPDATE subscriptions
    SET status = 'confirmed'
    WHERE status IS NULL;
    -- Make status mandatory
    ALTER TABLE subscriptions ALTER COLUMN status SET NOT NULL;
COMMIT;