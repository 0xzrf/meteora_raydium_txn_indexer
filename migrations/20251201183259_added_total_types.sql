-- Add migration script here
ALTER TABLE activities
    ALTER COLUMN total_deposit TYPE DOUBLE PRECISION,
    ALTER COLUMN total_withdraw TYPE DOUBLE PRECISION;