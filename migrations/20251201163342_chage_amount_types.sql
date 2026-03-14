-- Add migration script here
ALTER TABLE user_actions
    ALTER COLUMN amount_a TYPE NUMERIC,
    ALTER COLUMN amount_b TYPE NUMERIC;