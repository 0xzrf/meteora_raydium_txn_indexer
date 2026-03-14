-- Add migration script here
DROP TABLE IF EXISTS refferals;
DROP TABLE IF EXISTS withdrawals;

-- Add migration script here
CREATE TABLE fee_history (
    id BIGSERIAL PRIMARY KEY,

    from_pubkey TEXT NOT NULL,
    to_pubkey TEXT NOT NULL,
    amount NUMERIC NOT NULL,
    txn_sig TEXT NOT NULL,
    sol_price DOUBLE PRECISION NOT NULL,

    created_at TIMESTAMPTZ DEFAULT NOW()
);