-- Add migration script here
ALTER TABLE user_actions
    ADD COLUMN decimal_a SMALLINT,
    ADD COLUMN decimal_b SMALLINT;
