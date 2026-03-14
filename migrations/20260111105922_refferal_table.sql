-- Add migration script here
CREATE TABLE refferals (
    id BIGSERIAL PRIMARY KEY,

    from_pubkey TEXT NOT NULL,
    to_pubkey TEXT NOT NULL,
    amount BIGINT NOT NULL,
    txn_sig TEXT NOT NULL,
    sol_price DOUBLE PRECISION NOT NULL,

    created_at TIMESTAMPTZ DEFAULT NOW()
);