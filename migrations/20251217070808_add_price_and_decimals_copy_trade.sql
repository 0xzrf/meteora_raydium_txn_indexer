-- Add migration script here
ALTER TABLE copy_trading
    ADD COLUMN decimal_a SMALLINT,
    ADD COLUMN decimal_b SMALLINT,
    ADD COLUMN token_a_price DOUBLE PRECISION,
    ADD COLUMN token_b_price DOUBLE PRECISION;
    
