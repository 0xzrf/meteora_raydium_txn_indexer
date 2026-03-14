use crate::constants::{METEORA_DAMM_V2_PUBKEY, NOT_IN_TX};
use crate::parser::{
    Actions, CopyReturnType, CopyTradeParseData, ToCopyTradeField, token_transfer::TokenTxnInfo,
};
pub struct MeteoraDammV2CopyTradingProgram {
    pub program_id: String,
    pub disc: [u8; 8],
    pub account_keys: Vec<String>,
    pub action: Option<Actions>,
    pub txn: String,
    pub token_transfers: Vec<TokenTxnInfo>,
}

impl MeteoraDammV2CopyTradingProgram {
    const REMOVE_ALL_LIQUIDITY_IX: [u8; 8] = [10, 51, 61, 35, 112, 105, 24, 85];

    pub fn new(
        account_keys: Vec<String>,
        ix_data: Vec<u8>,
        txn: String,
        token_transfers: Vec<TokenTxnInfo>,
    ) -> Self {
        let (disc, _) = ix_data.split_at(8);

        let disc: [u8; 8] = disc.try_into().unwrap_or([0; 8]);

        MeteoraDammV2CopyTradingProgram {
            program_id: METEORA_DAMM_V2_PUBKEY.to_string(),
            account_keys,
            action: None,
            txn,
            disc,
            token_transfers,
        }
    }

    // Check to see if the instruciton initialized by the user is actually the one we want
    pub fn set_action_type(mut self) -> Self {
        match self.disc {
            Self::ADD_LIQUIDITY_IX => self.action = Some(Actions::AddLiquidity),
            Self::REMOVE_ALL_LIQUIDITY_IX | Self::REMOVE_LIQUIDITY_IX => {
                self.action = Some(Actions::RemoveLiquidity)
            }
            Self::CREATE_POSITION_IX => self.action = Some(Actions::CreatePosition),
            Self::CLOSE_POSITION_IX => self.action = Some(Actions::ClosePosition),
            _ => {}
        }

        self
    }

    pub fn get_parsed_data(self) -> Option<(CopyReturnType, String)> {
        if let Some(a) = &self.action {
            match a {
                Actions::AddLiquidity => {
                    println!("add liquidity triggered by user");
                    let owner = self.account_keys[9].clone();
                    Some((self.add_liquidity(owner.clone()), owner))
                }
                Actions::RemoveLiquidity => {
                    let owner = self.account_keys[10].clone();
                    println!("Remove liquidity initialized");
                    Some((self.remove_liquidity(owner.clone()), owner))
                }
                Actions::ClosePosition => {
                    println!("close position initialized");
                    let owner = self.account_keys[10].clone();
                    Some((self.close_position(owner.clone()), owner))
                }
                Actions::CreatePosition => {
                    println!("create position initialized");
                    let owner = self.account_keys[10].clone(); // TODO: change index
                    Some((self.create_position(owner.clone()), owner))
                }
                _ => None,
            }
        } else {
            println!("user initialized an ix");
            // self.log_msgs();
            None
        }
    }

    pub fn log_keys(&self) {
        for (i, key) in &mut self.account_keys.iter().enumerate() {
            println!("{i} {key}");
        }
    }
}

impl ToCopyTradeField for MeteoraDammV2CopyTradingProgram {
    const REMOVE_LIQUIDITY_IX: [u8; 8] = [80, 85, 209, 72, 24, 206, 177, 108];
    const ADD_LIQUIDITY_IX: [u8; 8] = [181, 157, 89, 67, 143, 182, 52, 72];
    const CREATE_POSITION_IX: [u8; 8] = [48, 215, 197, 153, 96, 203, 180, 133]; // create_position
    const CLOSE_POSITION_IX: [u8; 8] = [123, 134, 81, 0, 49, 68, 98, 98]; // close_position

    fn add_liquidity(self, owner: String) -> CopyReturnType {
        let keys = &self.account_keys;

        let token_a = keys[7].clone();
        let pool_address = keys[0].clone();
        let token_b = keys[6].clone();

        let user_token_account_a = keys[2].clone();
        let user_token_account_b = keys[3].clone();

        let pool_token_account_a = keys[4].clone();
        let pool_token_account_b = keys[5].clone();

        let amount_a: u64 = Self::get_token_transfer_for_user(
            &user_token_account_a,
            &pool_token_account_a,
            &self.token_transfers,
        );
        let amount_b: u64 = Self::get_token_transfer_for_user(
            &user_token_account_b,
            &pool_token_account_b,
            &self.token_transfers,
        );

        Ok(CopyTradeParseData {
            pool_address,
            contract_address: self.program_id,
            amount_a,
            amount_b,
            position_nft: None,
            action: self.action.unwrap().as_str(),
            owner,
            token_a,
            token_b,
            txn_sig: self.txn,
            max_bin_id: 0,
            min_bin_id: 0,
            strategy: None,
            decimal_a: None,
            decimal_b: None,
            token_a_price: None,
            token_b_price: None,
        })
    }

    fn remove_liquidity(self, owner: String) -> CopyReturnType {
        let keys = &self.account_keys;

        let pool_address = keys[1].clone();
        let token_a = keys[8].clone();
        let token_b = keys[7].clone();

        let user_token_account_a = keys[3].clone();
        let user_token_account_b = keys[4].clone();

        let pool_token_account_a = keys[5].clone();
        let pool_token_account_b = keys[6].clone();

        let amount_a: u64 = Self::get_token_transfer_for_user(
            &pool_token_account_a,
            &user_token_account_a,
            &self.token_transfers,
        );
        let amount_b: u64 = Self::get_token_transfer_for_user(
            &pool_token_account_b,
            &user_token_account_b,
            &self.token_transfers,
        );

        Ok(CopyTradeParseData {
            pool_address,
            contract_address: self.program_id,
            amount_a,
            amount_b,
            position_nft: None,
            token_a,
            token_b,
            txn_sig: self.txn,
            max_bin_id: 0,
            min_bin_id: 0,
            owner,
            strategy: None,
            action: self.action.unwrap().as_str(),
            decimal_a: None,
            decimal_b: None,
            token_a_price: None,
            token_b_price: None,
        })
    }

    fn close_position(self, owner: String) -> CopyReturnType {
        let keys = &self.account_keys;

        if keys.len() < 4 {
            return Err("DAMM ERR: key length too short");
        }

        let position_nft = Some(keys[3].clone());
        let pool_address = keys[2].clone();

        let token_a = NOT_IN_TX.to_string();
        let token_b = NOT_IN_TX.to_string();

        let amount_a: u64 = 0;
        let amount_b: u64 = 0;

        Ok(CopyTradeParseData {
            pool_address,
            contract_address: self.program_id,
            amount_a,
            amount_b,
            position_nft,
            token_a,
            token_b,
            txn_sig: self.txn,
            max_bin_id: 0,
            min_bin_id: 0,
            strategy: None,
            action: self.action.unwrap().as_str(),
            owner,
            decimal_a: None,
            decimal_b: None,
            token_a_price: None,
            token_b_price: None,
        })
    }
    fn create_position(self, owner: String) -> CopyReturnType {
        let keys = &self.account_keys;

        if keys.len() < 5 {
            return Err("DAMM ERR: key length too short");
        }

        let position_nft = Some(keys[4].clone());
        let pool_address = keys[3].clone();

        let token_a = NOT_IN_TX.to_string();
        let token_b = NOT_IN_TX.to_string();

        let amount_a = 0; // Since the user is creating position
        let amount_b = 0;

        Ok(CopyTradeParseData {
            pool_address,
            contract_address: self.program_id,
            amount_a,
            amount_b,
            position_nft,
            token_a,
            token_b,
            txn_sig: self.txn,
            owner,
            max_bin_id: 0,
            min_bin_id: 0,
            strategy: None,
            action: self.action.unwrap().as_str(),
            decimal_a: None,
            decimal_b: None,
            token_a_price: None,
            token_b_price: None,
        })
    }
}
// 5151
