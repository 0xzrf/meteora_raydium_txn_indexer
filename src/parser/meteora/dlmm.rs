use crate::constants::NOT_IN_TX;
use crate::parser::{
    Actions, ParseConfigs, ParseData, PoolReturnType, ToDbFields, token_transfer::TokenTxnInfo,
};
use crate::{constants::METEORA_DLMM_PUBKEY, helper::is_debugging};
pub struct MeteoraDlmmProgram {
    pub program_id: String,
    pub disc: [u8; 8],
    pub account_keys: Vec<String>,
    pub action: Option<Actions>,
    pub txn: String,
    pub token_transfers: Vec<TokenTxnInfo>,
}

impl MeteoraDlmmProgram {
    const OPEN_POSITION_IX: [u8; 8] = [143, 19, 242, 145, 213, 15, 104, 115]; // initialize_position2
    const CLAIM_FEE_IX: [u8; 8] = [112, 191, 101, 171, 28, 144, 127, 187]; // claim_fee2
    const CLOSE_POSITION_IX: [u8; 8] = [174, 90, 35, 115, 186, 40, 147, 226]; // close_position2
    const CLOSE_POSITION_IF_EMPTY: [u8; 8] = [59, 124, 212, 118, 91, 152, 110, 157]; // close_position_if_empty
    pub const REBALANCE_LIQUIDITY: [u8; 8] = [92, 4, 176, 193, 119, 185, 83, 9]; // rebalance_liquidity

    pub fn new(configs: ParseConfigs) -> Self {
        let ParseConfigs {
            txn,
            ix_accounts,
            ix_data,
            token_transfers,
        } = configs;

        let (disc, _) = ix_data.split_at(8);

        let disc: [u8; 8] = disc.try_into().unwrap_or([0; 8]);

        MeteoraDlmmProgram {
            program_id: METEORA_DLMM_PUBKEY.to_string(),
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
            Self::ADD_LIQUIDITY_IX | Self::REBALANCE_LIQUIDITY => {
                self.action = Some(Actions::AddLiquidity);
            }
            Self::REMOVE_LIQUIDITY_IX => {
                self.action = Some(Actions::RemoveLiquidity);
            }
            Self::CREATE_POOL_IX => {
                self.action = Some(Actions::CreatePool);
            }
            Self::OPEN_POSITION_IX => {
                self.action = Some(Actions::CreatePosition);
            }
            Self::CLOSE_POSITION_IX | Self::CLOSE_POSITION_IF_EMPTY => {
                self.action = Some(Actions::ClosePosition);
            }
            Self::CLAIM_FEE_IX => {
                self.action = Some(Actions::ClaimFee);
            }
            _ => {}
        }

        self
    }

    pub fn get_parsed_data(self) -> Option<(PoolReturnType, String)> {
        if let Some(a) = &self.action {
            match a {
                Actions::AddLiquidity => {
                    let owner = self.account_keys[9].clone();
                    println!("add liquidity triggered by user");

                    if self.disc.eq(&Self::ADD_LIQUIDITY_IX) {
                        Some((self.add_liquidity(), owner))
                    } else {
                        Some((self.rebalance_liquidity(), owner))
                    }
                }
                Actions::RemoveLiquidity => {
                    println!("Remove liquidity initialized");
                    let owner = self.account_keys[9].clone();

                    Some((self.remove_liquidity(), owner))
                }
                Actions::CreatePool => {
                    println!("create pool initialized");
                    let owner = self.account_keys[8].clone();

                    Some((self.create_pool(), owner))
                }
                Actions::ClaimFee => {
                    println!("Claim fee initialized");
                    let owner = self.account_keys[2].clone();
                    Some((self.claim_fee(), owner))
                }
                Actions::CreatePosition => {
                    println!("Create position initialized");
                    let owner = self.account_keys[3].clone();

                    Some((self.create_position(), owner))
                }
                Actions::ClosePosition => {
                    println!("Close position initialized");
                    let owner = self.account_keys[1].clone();

                    Some((self.close_position(), owner))
                }
            }
        } else {
            println!("user initialized an ix");
            None
        }
    }

    pub fn rebalance_liquidity(self) -> PoolReturnType {
        let keys = &self.account_keys;

        let token_a = keys[7].clone();
        let token_b = keys[8].clone();
        let pool_address = keys[1].clone();

        let position_nft = Some(keys[0].clone());

        let user_token_account_a = keys[3].clone();
        let user_token_account_b = keys[4].clone();

        let pool_token_account_a = keys[5].clone();
        let pool_token_account_b = keys[6].clone();

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
            println!("Couldn't get the data");
        }

        Ok(ParseData {
            contract_address: self.program_id,
            amount_a,
            amount_b,
            position_nft,
            token_a,
            token_b,
            pool_address,
            txn_sig: self.txn,
            action: self.action.unwrap().as_str(),
            decimal_a: None,
            decimal_b: None,
            token_a_price: None,
            token_b_price: None,
        })
    }

    pub fn claim_fee(self) -> PoolReturnType {
        let keys = &self.account_keys;
        if keys.len() < 9 {
            return Err("DLMM ERR: key length too short");
        }
        let token_a = keys[7].clone();
        let token_b = keys[8].clone();
        let pool_address = keys[0].clone();

        let position_nft = Some(keys[1].clone());
        let user_token_account_a = keys[5].clone();
        let user_token_account_b = keys[6].clone();

        let pool_token_account_a = keys[3].clone();
        let pool_token_account_b = keys[4].clone();

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
            println!("Couldn't get the data");
        }

        Ok(ParseData {
            amount_a,
            amount_b,
            action: self.action.unwrap().as_str(),
            contract_address: self.program_id,
            position_nft,
            pool_address,
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

        let token_a = NOT_IN_TX.to_string();
        let token_b = NOT_IN_TX.to_string();
        let pool_address = NOT_IN_TX.to_string();

        let position_nft = Some(keys[0].clone());
        let amount_a: u64 = 0;
        let amount_b: u64 = 0;

        Ok(ParseData {
            amount_a,
            amount_b,
            action: self.action.unwrap().as_str(),
            contract_address: self.program_id,
            position_nft,
            pool_address,
            token_a,
            token_a_price: None,
            token_b,
            token_b_price: None,
            txn_sig: self.txn,
            decimal_a: None,
            decimal_b: None,
        })
    }

    pub fn create_position(self) -> PoolReturnType {
        let keys = &self.account_keys;
        if keys.len() < 3 {
            return Err("DLMM ERR: key length too short");
        }

        let token_a = NOT_IN_TX.to_string();
        let token_b = NOT_IN_TX.to_string();
        let pool_address = keys[2].clone();

        let position_nft = Some(keys[1].clone());
        let amount_a: u64 = 0;
        let amount_b: u64 = 0;

        Ok(ParseData {
            amount_a,
            amount_b,
            action: self.action.unwrap().as_str(),
            contract_address: self.program_id,
            position_nft,
            pool_address,
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

impl ToDbFields for MeteoraDlmmProgram {
    const CREATE_POOL_IX: [u8; 8] = [243, 73, 129, 126, 51, 19, 241, 107]; // initialize_customizable_permissionless_lb_pair2
    const REMOVE_LIQUIDITY_IX: [u8; 8] = [204, 2, 195, 145, 53, 145, 145, 205]; // remove_liquidity_by_range2
    const ADD_LIQUIDITY_IX: [u8; 8] = [3, 221, 149, 218, 111, 141, 118, 213]; // add_liquidity_by_strategy2

    fn add_liquidity(self) -> PoolReturnType {
        let keys = &self.account_keys;

        if keys.len() < 9 {
            return Err("DLMM ERR: key length too short");
        }

        let token_a = keys[7].clone();
        let token_b = keys[8].clone();
        let pool_address = keys[1].clone();

        let user_token_account_a = keys[3].clone();
        let user_token_account_b = keys[4].clone();

        let pool_token_account_a = keys[5].clone();
        let pool_token_account_b = keys[6].clone();

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

        let position_nft = Some(keys[0].clone());
        if amount_a == 0 || amount_b == 0 {
            println!("Couldn't get the data");
        }

        Ok(ParseData {
            amount_a,
            amount_b,
            action: self.action.unwrap().as_str(),
            contract_address: self.program_id,
            position_nft,
            pool_address,
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

        if keys.len() < 13 {
            return Err("DLMM ERR: key length too short");
        }

        let token_a = keys[2].clone();
        let token_b = keys[3].clone();
        let pool_address = keys[0].clone();

        let amount_a: u64 = 0;
        let amount_b: u64 = 0;

        Ok(ParseData {
            amount_a,
            amount_b,
            action: self.action.unwrap().as_str(),
            contract_address: self.program_id,
            position_nft: None,
            pool_address,
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
            return Err("DLMM ERR: key length too short");
        }

        let token_a = keys[7].clone();
        let token_b = keys[8].clone();
        let pool_address = keys[1].clone();

        let position_nft = Some(keys[0].clone());

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
            println!("Couldn't get the data");
        }

        Ok(ParseData {
            amount_a,
            amount_b,
            action: self.action.unwrap().as_str(),
            contract_address: self.program_id,
            position_nft,
            pool_address,
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
