-- Add migration script here
ALTER TABLE copy_trading
    ALTER COLUMN amount_a TYPE NUMERIC,
    ALTER COLUMN amount_b TYPE NUMERIC;