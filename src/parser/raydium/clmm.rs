use crate::{
    constants::{NOT_IN_TX, RAYDIUM_CLMM_DEVNET_PUBKEY, RAYDIUM_CLMM_PUBKEY},
    helper::{is_debugging, is_devnet},
    parser::{
        Actions, ParseConfigs, ParseData, PoolReturnType, ToDbFields, token_transfer::TokenTxnInfo,
    },
};
pub struct RaydiumClmmProgram {
    pub program_id: String,
    pub account_keys: Vec<String>,
    pub action: Option<Actions>,
    pub txn: String,
    pub disc: [u8; 8],
    pub token_transfers: Vec<TokenTxnInfo>,
}

impl RaydiumClmmProgram {
    const OPEN_POSITION_IX: [u8; 8] = [77, 184, 74, 214, 112, 86, 241, 199]; // open_position_v2
    const OPEN_POSITION_NFT_2022_IX: [u8; 8] = [77, 255, 174, 82, 125, 29, 201, 46]; // open_position_with_token22_nft
    const CLOSE_POSITION_IX: [u8; 8] = [123, 134, 81, 0, 49, 68, 98, 98]; // close_position

    pub fn new(configs: ParseConfigs) -> Self {
        let program_id = if is_devnet() {
            RAYDIUM_CLMM_DEVNET_PUBKEY.to_string()
        } else {
            RAYDIUM_CLMM_PUBKEY.to_string()
        };
        let ParseConfigs {
            txn,
            ix_accounts,
            ix_data,
            token_transfers,
        } = configs;

        let (disc, _) = ix_data.split_at(8);

        let disc: [u8; 8] = disc.try_into().unwrap_or([0; 8]);

        RaydiumClmmProgram {
            program_id,
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
            Self::ADD_LIQUIDITY_IX => {
                self.action = Some(Actions::AddLiquidity);
            }
            Self::CLOSE_POSITION_IX => self.action = Some(Actions::ClosePosition),
            Self::CREATE_POOL_IX => self.action = Some(Actions::CreatePool),
            Self::OPEN_POSITION_IX | Self::OPEN_POSITION_NFT_2022_IX => {
                self.action = Some(Actions::CreatePosition)
            }
            Self::REMOVE_LIQUIDITY_IX => self.action = Some(Actions::RemoveLiquidity),
            _ => {}
        }

        self
    }

    pub fn get_parsed_data(self) -> Option<(PoolReturnType, String)> {
        if let Some(a) = &self.action {
            let owner = self.account_keys[0].clone();
            match a {
                Actions::AddLiquidity => {
                    println!("add liquidity triggered by user");

                    Some((self.add_liquidity(), owner))
                }
                Actions::CreatePool => {
                    println!("create pool initialized");

                    Some((self.create_pool(), owner))
                }
                Actions::RemoveLiquidity => {
                    println!("Remove liquidity initialized");

                    Some((self.remove_liquidity(), owner))
                }
                Actions::CreatePosition => {
                    if self.disc.eq(&Self::OPEN_POSITION_IX) {
                        println!("open_position_v2 initialized");
                        Some((self.create_position_v2(), owner))
                    } else {
                        println!("create_position_nft_2022 initialized");
                        Some((self.create_position_nft_token2022(), owner))
                    }
                }
                Actions::ClosePosition => {
                    println!("Close position initialized");
                    Some((self.close_position(), owner))
                }
                Actions::ClaimFee => None,
            }
        } else {
            println!("user initialized an ix");
            None
        }
    }

    pub fn create_position_v2(self) -> PoolReturnType {
        let keys = &self.account_keys;

        if keys.len() < 22 {
            return Err("CLMM ERR: key length too short");
        }

        let pool_address = keys[5].clone();
        let token_a = keys[20].clone();
        let token_b = keys[21].clone();
        let position_nft = Some(keys[9].clone());

        let user_token_account_a = keys[10].clone();
        let user_token_account_b = keys[11].clone();

        let pool_token_account_a = keys[12].clone();
        let pool_token_account_b = keys[13].clone();

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
            action: self.action.unwrap().as_str(), // It's fine since we only call this when the action is set
            contract_address: self.program_id.to_string(),
            amount_a,
            amount_b,
            pool_address,
            position_nft,
            token_a,
            token_a_price: None,
            token_b,
            token_b_price: None,
            txn_sig: self.txn,
            decimal_a: None,
            decimal_b: None,
        })
    }

    pub fn create_position_nft_token2022(self) -> PoolReturnType {
        let keys = &self.account_keys;

        if keys.len() < 20 {
            return Err("CLMM ERR: key length too short");
        }

        let pool_address = keys[4].clone();
        let token_a = keys[18].clone();
        let token_b = keys[19].clone();
        let position_nft = Some(keys[8].clone());

        let user_token_account_a = keys[9].clone();
        let user_token_account_b = keys[10].clone();

        let pool_token_account_a = keys[11].clone();
        let pool_token_account_b = keys[12].clone();

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
            action: self.action.unwrap().as_str(), // It's fine since we only call this when the action is set
            contract_address: self.program_id.to_string(),
            amount_a,
            amount_b,
            pool_address,
            position_nft,
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
            return Err("CLMM ERR: key length too short");
        }

        let pool_address = NOT_IN_TX.to_string();
        let token_a = NOT_IN_TX.to_string();
        let token_b = NOT_IN_TX.to_string();

        let position_nft = Some(keys[3].clone());

        Ok(ParseData {
            action: self.action.unwrap().as_str(), // It's fine since we only call this when the action is set
            contract_address: self.program_id.to_string(),
            amount_a: 0, // Since we're closing the position
            amount_b: 0,
            pool_address,
            position_nft,
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

impl ToDbFields for RaydiumClmmProgram {
    const CREATE_POOL_IX: [u8; 8] = [233, 146, 209, 142, 207, 104, 64, 188]; //create_pool
    const REMOVE_LIQUIDITY_IX: [u8; 8] = [58, 127, 188, 62, 79, 82, 196, 96]; // decrease_liquidity_v2
    const ADD_LIQUIDITY_IX: [u8; 8] = [133, 29, 89, 223, 69, 238, 176, 10]; // increase_liquidity_v2

    fn add_liquidity(self) -> PoolReturnType {
        let keys = &self.account_keys;

        if keys.len() < 15 {
            return Err("CLMM ERR: key length too short");
        }

        let token_a = keys[13].clone();
        let token_b = keys[14].clone();
        let pool_address = keys[2].clone();
        let position_nft = Some(keys[4].clone());

        let user_token_account_a = keys[7].clone();
        let user_token_account_b = keys[8].clone();

        let pool_token_account_a = keys[9].clone();
        let pool_token_account_b = keys[10].clone();

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
            action: self.action.unwrap().as_str(), // It's fine since we only call this when the action is set
            contract_address: self.program_id.to_string(),
            amount_a,
            amount_b,
            pool_address,
            position_nft,
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
        if keys.len() < 5 {
            return Err("CLMM ERR: key length too short");
        }

        let token_a = keys[3].clone();
        let token_b = keys[4].clone();
        let pool_address = keys[2].clone();

        let amount_a: u64 = 0;
        let amount_b: u64 = 0;

        Ok(ParseData {
            action: self.action.unwrap().as_str(), // It's fine since we only call this when the action is set
            contract_address: self.program_id.to_string(),
            amount_a,
            amount_b,
            pool_address,
            position_nft: None,
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

        if keys.len() < 16 {
            return Err("CLMM ERR: key length too short");
        }

        let pool_address = keys[3].clone();
        let token_a = keys[14].clone();
        let token_b = keys[15].clone();
        let position_nft = Some(keys[2].clone());

        let user_token_account_a = keys[9].clone();
        let user_token_account_b = keys[10].clone();

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

        Ok(ParseData {
            action: self.action.unwrap().as_str(), // It's fine since we only call this when the action is set
            contract_address: self.program_id.to_string(),
            amount_a,
            amount_b,
            pool_address,
            position_nft,
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
