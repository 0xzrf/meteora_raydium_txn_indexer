-- Add migration script here
CREATE TABLE user_actions (
    id BIGSERIAL PRIMARY KEY,

    user_pubkey TEXT NOT NULL,

    contract_address TEXT NOT NULL,
    amount_b BIGINT NOT NULL,
    amount_a BIGINT NOT NULL,

    position_nft TEXT,                -- nullable
    token_a TEXT NOT NULL,
    token_b TEXT NOT NULL,

    pool_address TEXT NOT NULL,

    token_a_price BIGINT,             -- nullable
    token_b_price BIGINT,             -- nullable

    action TEXT NOT NULL,
    txn_sig TEXT NOT NULL,

    created_at TIMESTAMPTZ DEFAULT NOW()
);