use crate::parser::{
    Actions, ParseConfigs, ParseData, PoolReturnType, ToDbFields, token_transfer::TokenTxnInfo,
};
use crate::{
    constants::{METEORA_DAMM_V2_PUBKEY, NOT_IN_TX},
    is_debugging,
};
pub struct MeteoraDammV2Program {
    pub program_id: String,
    pub disc: [u8; 8],
    pub account_keys: Vec<String>,
    pub action: Option<Actions>,
    pub txn: String,
    pub token_transfers: Vec<TokenTxnInfo>,
}

impl MeteoraDammV2Program {
    const CLAIM_FEE_IX: [u8; 8] = [180, 38, 154, 17, 133, 33, 162, 211]; // claim_position_fee
    const OPEN_POSITION_IX: [u8; 8] = [48, 215, 197, 153, 96, 203, 180, 133]; // create_position
    const CLOSE_POSITION_IX: [u8; 8] = [123, 134, 81, 0, 49, 68, 98, 98]; // close_position
    const REMOVE_ALL_LIQUIDITY_IX: [u8; 8] = [10, 51, 61, 35, 112, 105, 24, 85]; // remove_all_liquidity

    pub fn new(configs: ParseConfigs) -> Self {
        let ParseConfigs {
            txn,
            ix_accounts,
            ix_data,
            token_transfers,
        } = configs;

        let (disc, _) = ix_data.split_at(8);

        let disc: [u8; 8] = disc.try_into().unwrap_or([0; 8]);

        MeteoraDammV2Program {
            program_id: METEORA_DAMM_V2_PUBKEY.to_string(),
            account_keys: ix_accounts,
            action: None,
            disc,
            txn,
            token_transfers,
        }
    }

    // Check to see if the instruciton initialized by the user is actually the one we want
    pub fn set_action_type(mut self) -> Self {
        if is_debugging() {
            Self::log_keys(&self.account_keys);
            Self::log_token_transfers(&self.token_transfers);
        }
        match self.disc {
            Self::ADD_LIQUIDITY_IX => self.action = Some(Actions::AddLiquidity),
            Self::CLAIM_FEE_IX => self.action = Some(Actions::ClaimFee),
            Self::CLOSE_POSITION_IX => self.action = Some(Actions::ClosePosition),
            Self::OPEN_POSITION_IX => self.action = Some(Actions::CreatePosition),
            Self::REMOVE_ALL_LIQUIDITY_IX | Self::REMOVE_LIQUIDITY_IX => {
                self.action = Some(Actions::RemoveLiquidity)
            }
            Self::CREATE_POOL_IX => self.action = Some(Actions::CreatePool),
            _ => {}
        }

        self
    }

    pub fn get_parsed_data(self) -> Option<(PoolReturnType, String)> {
        if let Some(a) = &self.action {
            match a {
                Actions::AddLiquidity => {
                    println!("add liquidity triggered by user");
                    let owner = self.account_keys[9].clone();
                    Some((self.add_liquidity(), owner))
                }
                Actions::ClaimFee => {
                    println!("Claim fee initialized");
                    let owner = self.account_keys[10].clone();
                    Some((self.claim_fee(), owner))
                }
                Actions::CreatePool => {
                    println!("create pool initialized");
                    let owner = self.account_keys[0].clone();
                    Some((self.create_pool(), owner))
                }
                Actions::RemoveLiquidity => {
                    let owner = self.account_keys[10].clone();
                    println!("Remove liquidity initialized");
                    Some((self.remove_liquidity(), owner))
                }
                Actions::CreatePosition => {
                    println!("Create position initialized");
                    let owner = self.account_keys[0].clone();
                    Some((self.create_position(), owner))
                }
                Actions::ClosePosition => {
                    println!("Close position initialized");
                    let owner = self.account_keys[6].clone();
                    Some((self.close_position(), owner))
                }
            }
        } else {
            println!("user initialized an ix");
            // self.log_msgs();
            None
        }
    }

    pub fn create_position(self) -> PoolReturnType {
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

        Ok(ParseData {
            pool_address,
            position_nft,
            token_a,
            token_a_price: None,
            token_b,
            token_b_price: None,
            action: self.action.unwrap().as_str(),
            contract_address: self.program_id,
            amount_a,
            amount_b,
            txn_sig: self.txn,
            decimal_a: None,
            decimal_b: None,
        })
    }

    pub fn claim_fee(self) -> PoolReturnType {
        let keys = &self.account_keys;

        if keys.len() < 11 {
            return Err("DAMM-V2 ERR: key length too short");
        }

        let pool_address = keys[1].clone();

        let token_a = keys[7].clone();
        let token_b = keys[8].clone();
        let position = Some(keys[2].clone());

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

        if amount_a == 0 || amount_b == 0 {
            println!("Couldn't capture amounts");
        }

        Ok(ParseData {
            pool_address,
            contract_address: self.program_id,
            amount_a,
            amount_b,
            position_nft: position,
            action: self.action.unwrap().as_str(),
            token_a,
            token_a_price: None,
            token_b,
            token_b_price: None,
            txn_sig: self.txn,
            decimal_a: None,
            decimal_b: None,
        })
    }

    pub fn close_position(self) -> PoolReturnType {
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

        Ok(ParseData {
            pool_address,
            contract_address: self.program_id,
            amount_a,
            amount_b,
            position_nft,
            action: self.action.unwrap().as_str(),
            token_a,
            token_a_price: None,
            token_b,
            token_b_price: None,
            txn_sig: self.txn,
            decimal_a: None,
            decimal_b: None,
        })
    }
}

impl ToDbFields for MeteoraDammV2Program {
    const CREATE_POOL_IX: [u8; 8] = [95, 180, 10, 172, 84, 174, 232, 40]; // initialize_pool
    const REMOVE_LIQUIDITY_IX: [u8; 8] = [80, 85, 209, 72, 24, 206, 177, 108]; // remove_liquidity
    const ADD_LIQUIDITY_IX: [u8; 8] = [181, 157, 89, 67, 143, 182, 52, 72]; // add_liquidity

    fn add_liquidity(self) -> PoolReturnType {
        let keys = &self.account_keys;
        if keys.len() < 8 {
            return Err("DAMM ERR: key length too short");
        }

        let token_a = keys[6].clone();
        let token_b = keys[7].clone();
        let pool_address = keys[0].clone();

        let position_nft = Some(keys[1].clone());

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

        if amount_a == 0 || amount_b == 0 {
            println!("Couldn't capture amounts");
        }

        Ok(ParseData {
            pool_address,
            contract_address: self.program_id,
            amount_a,
            amount_b,
            position_nft,
            action: self.action.unwrap().as_str(),
            token_a,
            token_a_price: None,
            token_b,
            token_b_price: None,
            txn_sig: self.txn,
            decimal_a: None,
            decimal_b: None,
        })
    }

    fn create_pool(self) -> PoolReturnType {
        let keys = &self.account_keys;
        if keys.len() < 10 {
            return Err("DAMM ERR: key length too short");
        }
        let pool_address = keys[6].clone();
        let token_a = keys[8].clone();
        let token_b = keys[9].clone();
        let position_nft = Some(keys[7].clone());

        let user_token_account_a = keys[12].clone();
        let user_token_account_b = keys[13].clone();

        let pool_token_account_a = keys[10].clone();
        let pool_token_account_b = keys[11].clone();

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

        Ok(ParseData {
            pool_address,
            contract_address: self.program_id,
            amount_a,
            amount_b,
            position_nft,
            action: self.action.unwrap().as_str(),
            token_a,
            token_a_price: None,
            token_b,
            token_b_price: None,
            txn_sig: self.txn,
            decimal_a: None,
            decimal_b: None,
        })
    }

    fn remove_liquidity(self) -> PoolReturnType {
        let keys = &self.account_keys;
        if keys.len() < 9 {
            return Err("DAMM ERR: key length too short");
        }
        let pool_address = keys[1].clone();
        let token_a = keys[7].clone();
        let token_b = keys[8].clone();

        let position_nft = Some(keys[2].clone());

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

        if amount_a == 0 || amount_b == 0 {
            println!("Couldn't capture amounts");
        }

        Ok(ParseData {
            pool_address,
            contract_address: self.program_id,
            amount_a,
            amount_b,
            position_nft,
            action: self.action.unwrap().as_str(),
            token_a,
            token_a_price: None,
            token_b,
            token_b_price: None,
            txn_sig: self.txn,
            decimal_a: None,
            decimal_b: None,
        })
    }
}
