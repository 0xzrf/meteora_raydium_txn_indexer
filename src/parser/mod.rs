pub mod prelude;
pub use prelude::*;
use serde::Serialize;
pub mod api_parsers;
pub mod meteora;
pub mod raydium;
pub mod token_transfer;
use crate::parser::token_transfer::TokenTxnInfo;
#[derive(Debug, Clone)]
pub struct ParseData {
    pub contract_address: String,
    pub amount_b: u64, // these amounts will have the context based on the action
    pub amount_a: u64,
    pub position_nft: Option<String>, // Pubkey base58 string
    pub token_a: String,
    pub token_b: String,
    pub pool_address: String,
    pub token_a_price: Option<f64>, // still in development
    pub token_b_price: Option<f64>,
    pub action: String,
    pub txn_sig: String,
    pub decimal_a: Option<u8>,
    pub decimal_b: Option<u8>,
}

impl PartialEq<ParseData> for ParseData {
    fn eq(&self, other: &ParseData) -> bool {
        self.txn_sig.eq(&other.txn_sig)
            && self.action.eq(&other.action)
            && self.contract_address.eq(&other.contract_address)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct CopyTradeParseData {
    pub contract_address: String,
    pub amount_a: u64,
    pub amount_b: u64,
    pub position_nft: Option<String>,
    pub token_a: String,
    pub token_b: String,
    pub pool_address: String,
    pub txn_sig: String,
    pub max_bin_id: i32,
    pub min_bin_id: i32,
    pub strategy: Option<u8>,
    pub action: String,
    pub owner: String,
    pub token_a_price: Option<f64>,
    pub token_b_price: Option<f64>,
    pub decimal_a: Option<u8>,
    pub decimal_b: Option<u8>,
}

impl PartialEq<CopyTradeParseData> for CopyTradeParseData {
    fn eq(&self, other: &CopyTradeParseData) -> bool {
        self.txn_sig.eq(&other.txn_sig) && self.action.eq(&other.action)
    }
}

#[derive(Debug)]
pub struct ParseConfigs {
    pub txn: String,
    pub ix_accounts: Vec<String>,
    pub ix_data: Vec<u8>,
    pub token_transfers: Vec<TokenTxnInfo>,
}

pub struct ParseConfigsCopyTrading {
    pub txn: String,
    pub ix_accounts: Vec<String>,
    pub token_transfers: Vec<TokenTxnInfo>,
    pub ix_data: Vec<u8>,
}

#[derive(Debug, Clone)]
pub enum Actions {
    CreatePool,
    AddLiquidity,
    RemoveLiquidity,
    ClaimFee,
    ClosePosition,
    CreatePosition,
}

impl Actions {
    pub fn as_str(&self) -> String {
        match self {
            Actions::AddLiquidity => String::from("add_liquidity"),
            Actions::ClaimFee => String::from("claim_fee"),
            Actions::ClosePosition => String::from("close_position"),
            Actions::CreatePool => String::from("create_pool"),
            Actions::CreatePosition => String::from("create_position"),
            Actions::RemoveLiquidity => String::from("remove_liquidity"),
        }
    }
}

type Disc = [u8; 8];

pub trait ToCopyTradeField {
    const ADD_LIQUIDITY_IX: Disc;
    const REMOVE_LIQUIDITY_IX: Disc;
    const CREATE_POSITION_IX: Disc;
    const CLOSE_POSITION_IX: Disc;

    fn add_liquidity(self, owner: String) -> CopyReturnType;
    fn remove_liquidity(self, owner: String) -> CopyReturnType;
    fn create_position(self, owner: String) -> CopyReturnType;
    fn close_position(self, owner: String) -> CopyReturnType;

    fn get_token_transfer_for_user(
        from: &str,
        to: &str,
        token_transfers: &Vec<TokenTxnInfo>,
    ) -> u64 {
        let mut amount = 0;
        for transfer in token_transfers {
            if transfer.from.eq(from) && transfer.to.eq(to) {
                amount = transfer.amount
            }
        }
        amount
    }

    fn log_keys(keys: &[String]) {
        for (i, key) in &mut keys.iter().enumerate() {
            println!("{i} - {key}");
        }
    }

    fn log_token_transfers(token_transfers: &Vec<TokenTxnInfo>) {
        for transfer in token_transfers {
            println!("{transfer:#?}");
        }
    }
}

type PoolReturnType = Result<ParseData, &'static str>;
type CopyReturnType = Result<CopyTradeParseData, &'static str>;

pub trait ToDbFields {
    const CREATE_POOL_IX: Disc;
    const ADD_LIQUIDITY_IX: Disc;
    const REMOVE_LIQUIDITY_IX: Disc;

    fn create_pool(self) -> PoolReturnType;
    fn add_liquidity(self) -> PoolReturnType;
    fn remove_liquidity(self) -> PoolReturnType;

    fn log_keys(keys: &[String]) {
        for (i, key) in &mut keys.iter().enumerate() {
            println!("{i} - {key}");
        }
    }

    fn log_token_transfers(token_transfers: &Vec<TokenTxnInfo>) {
        for transfer in token_transfers {
            println!("{transfer:#?}");
        }
    }

    fn get_token_transfer_for_user(
        from: &str,
        to: &str,
        token_transfers: &Vec<TokenTxnInfo>,
    ) -> u64 {
        let mut amount = 0;
        for transfer in token_transfers {
            if transfer.from.eq(from) && transfer.to.eq(to) {
                amount = transfer.amount
            }
        }
        amount
    }
}

pub struct DbTaskStruct {
    parsed_data: Option<PoolReturnType>,
    copy_trade: Option<CopyReturnType>,
    is_user: bool,
    signer: String,
}
