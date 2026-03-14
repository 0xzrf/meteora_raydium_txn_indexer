-- Add migration script here
ALTER TABLE user_actions
    ALTER COLUMN token_a_price TYPE DOUBLE PRECISION,
    ALTER COLUMN token_b_price TYPE DOUBLE PRECISION;