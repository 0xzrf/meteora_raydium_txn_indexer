CREATE TABLE users (
    user_pubkey TEXT PRIMARY KEY,  -- unique

    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE copy_trade_users (
    user_pubkey TEXT PRIMARY KEY,  -- unique

    created_at TIMESTAMPTZ DEFAULT NOW()
);
