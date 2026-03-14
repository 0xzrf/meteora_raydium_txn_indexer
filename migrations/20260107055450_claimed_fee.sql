-- Add migration script here
ALTER TABLE activities
    ADD COLUMN claimed_fee DOUBLE PRECISION;