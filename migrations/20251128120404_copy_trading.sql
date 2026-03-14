-- Add migration script here
CREATE TABLE copy_trading (
    id BIGSERIAL PRIMARY KEY,

    user_pubkey TEXT NOT NULL,

    contract_address TEXT NOT NULL,
    amount_b BIGINT NOT NULL,
    amount_a BIGINT NOT NULL,

    position_nft TEXT,                -- nullable
    token_a TEXT NOT NULL,
    token_b TEXT NOT NULL,

    pool_address TEXT NOT NULL,

    action TEXT NOT NULL,
    txn_sig TEXT NOT NULL,

    min_bin_id BIGINT NOT NULL,
    max_bin_id BIGINT NOT NULL,
    strategy SMALLINT,

    created_at TIMESTAMPTZ DEFAULT NOW()
);