// METEORA PUBKEYS
pub const METEORA_DAMM_V2_PUBKEY: &str = "cpamdpZCGKUy5JxQXB4dcpGPiikHawvSWAd6mEn1sGG";
pub const METEORA_DLMM_PUBKEY: &str = "LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo";

// RAYDIUM PUBKEYS
pub const RAYDIUM_CPMM_PUBKEY: &str = "CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C";
pub const RAYDIUM_CLMM_PUBKEY: &str = "CAMMCzo5YL8w4VFF8KVHrK22GGUsp5VTaW7grrKgrWqK";
pub const RAYDIUM_CPMM_DEVNET_PUBKEY: &str = "CPMDWBwJDtYax9qW7AyRuVC19Cc4L4Vcy4n2BHAbHkCW";
pub const RAYDIUM_CLMM_DEVNET_PUBKEY: &str = "DRaycpLY18LhpbydsBWbVJtxpNv9oXPgjRSfpF2bWpYb";
pub const RAYDIUM_AMM_DEVNET_PUBKEY: &str = "DRaya7Kj3aMWQSy19kSjvmuwq9docCHofyP9kanQGaav";
pub const RAYDIUM_AMM_PUBKEY: &str = "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8";

// GENERAL PUBKEYS
pub const TOKEN_PROGRAM: &str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";
pub const TOKEN_2022_PROGRAM: &str = "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb";
pub const SYSTEM_PROGRAM_ADDR: &str = "11111111111111111111111111111111";

// API CONFIG PUBKEYS
pub const DAMM_DEVNET_API_URL: &str = "https://dammv2-api.devnet.meteora.ag";
pub const DLMM_DEVNET_API_URL: &str = "https://devnet-dlmm-api.meteora.ag";
pub const DAMM_API_URL: &str = "https://dammv2-api.meteora.ag";
pub const DLMM_API_URL: &str = "https://dlmm-api.meteora.ag";
pub const RAYDIUM_AMM_API_URL: &str = "https://api-v3.raydium.io";
pub const RAYDIUM_AMM_DEVNET_API_URL: &str = "https://api-v3-devnet.raydium.io";

// A CONSTANT TO AVOID MISMATCH
pub const NOT_IN_TX: &str = "NOT_IN_IX";
pub const CACHE_LIMIT: usize = 5;
pub const USER_FILTER: &str = "user_filter";
pub const COPY_TRADE_FILTER: &str = "copy_trade_filter";
pub const FEE_RECEIVER_FILTER: &str = "fee_receiver_filter";

pub fn to_program_type(contract_id: &str) -> Option<&str> {
    match contract_id {
        id if id.eq(RAYDIUM_CLMM_PUBKEY) => Some("clmm"),
        id if id.eq(RAYDIUM_CPMM_PUBKEY) => Some("cpmm"),
        id if id.eq(METEORA_DAMM_V2_PUBKEY) => Some("damm"),
        id if id.eq(METEORA_DLMM_PUBKEY) => Some("dlmm"),
        _ => None,
    }
}
