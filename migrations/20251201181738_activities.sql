CREATE TABLE activities (
    id BIGSERIAL PRIMARY KEY,

    user_pubkey TEXT NOT NULL,

    pool_type TEXT NOT NULL,
    total_deposit BIGINT NOT NULL,
    total_withdraw BIGINT NOT NULL,

    position_nft TEXT NOT NULL,
    token_a TEXT NOT NULL,
    token_b TEXT NOT NULL,

    pool_address TEXT NOT NULL,

    action TEXT NOT NULL,
    txn_sig TEXT NOT NULL,

    close_time TIMESTAMPTZ DEFAULT NOW()
);