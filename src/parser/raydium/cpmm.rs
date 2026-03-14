use crate::{
    constants::{RAYDIUM_CPMM_DEVNET_PUBKEY, RAYDIUM_CPMM_PUBKEY},
    helper::{is_debugging, is_devnet},
    parser::{
        Actions, ParseConfigs, ParseData, PoolReturnType, ToDbFields, token_transfer::TokenTxnInfo,
    },
};
pub struct RaydiumCpmmProgram {
    pub program_id: String,
    pub account_keys: Vec<String>,
    pub action: Option<Actions>,
    pub disc: [u8; 8],
    pub txn: String,
    pub token_transfers: Vec<TokenTxnInfo>,
}

impl RaydiumCpmmProgram {
    pub fn new(configs: ParseConfigs) -> Self {
        let program_id = if is_devnet() {
            RAYDIUM_CPMM_DEVNET_PUBKEY.to_string()
        } else {
            RAYDIUM_CPMM_PUBKEY.to_string()
        };
        let ParseConfigs {
            txn,
            ix_accounts,
            ix_data,
            token_transfers,
        } = configs;

        let (disc, _) = ix_data.split_at(8);

        let disc: [u8; 8] = disc.try_into().unwrap_or([0; 8]);

        RaydiumCpmmProgram {
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
            Self::CREATE_POOL_IX => {
                self.action = Some(Actions::CreatePool);
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
                _ => None, // This line will never reach since there's no close_position, create_position or claim_fee ix
            }
        } else {
            None
        }
    }
}

impl ToDbFields for RaydiumCpmmProgram {
    const CREATE_POOL_IX: [u8; 8] = [175, 175, 109, 31, 13, 152, 155, 237]; // initialize
    const ADD_LIQUIDITY_IX: [u8; 8] = [242, 35, 198, 137, 82, 225, 242, 182]; // deposit
    const REMOVE_LIQUIDITY_IX: [u8; 8] = [183, 18, 70, 156, 148, 109, 161, 34]; // withdraw

    fn add_liquidity(self) -> PoolReturnType {
        let keys = &self.account_keys;

        if keys.len() < 12 {
            return Err("CPMM add liquidity has invalid account length");
        }

        let mint_a = keys[10].clone();
        let mint_b = keys[11].clone();
        let pool_address = keys[2].clone();
        let position_nft = Some(keys[3].clone());

        let user_token_account_a = keys[4].clone();
        let user_token_account_b = keys[5].clone();

        let pool_token_account_a = keys[6].clone();
        let pool_token_account_b = keys[7].clone();

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
            println!("Couldn't capture data");
        }

        Ok(ParseData {
            action: self.action.unwrap().as_str(),
            contract_address: self.program_id.to_string(),
            amount_a,
            amount_b,
            pool_address,
            position_nft,
            token_a: mint_a,
            token_b: mint_b,
            token_a_price: None,
            token_b_price: None,
            txn_sig: self.txn,
            decimal_a: None,
            decimal_b: None,
        })
    }

    fn create_pool(self) -> PoolReturnType {
        let keys = &self.account_keys;
        if keys.len() < 6 {
            return Err("CPMM add liquidity has invalid account length");
        }

        let token_a = keys[4].clone();
        let token_b = keys[5].clone();
        let pool_address = keys[3].clone();

        let user_token_account_a = keys[7].clone();
        let user_token_account_b = keys[8].clone();

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
            action: self.action.unwrap().as_str(),
            contract_address: self.program_id,
            amount_a,
            amount_b,
            token_a,
            token_b,
            token_a_price: None,
            token_b_price: None,
            position_nft: None,
            pool_address,
            txn_sig: self.txn,
            decimal_a: None,
            decimal_b: None,
        })
    }

    fn remove_liquidity(self) -> PoolReturnType {
        let keys = &self.account_keys;

        if keys.len() < 14 {
            return Err("CPMM remove liquidity has invalid account length");
        }

        let pool_address = keys[2].clone();
        let token_a = keys[10].clone();
        let token_b = keys[11].clone();

        let user_token_account_a = keys[4].clone();
        let user_token_account_b = keys[5].clone();

        let pool_token_account_a = keys[6].clone();
        let pool_token_account_b = keys[7].clone();

        let position_nft = Some(keys[3].clone());

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
            pool_address,
            token_a,
            token_b,
            contract_address: self.program_id,
            amount_a,
            amount_b,
            position_nft,
            token_a_price: None,
            token_b_price: None,
            action: self.action.unwrap().as_str(),
            txn_sig: self.txn,
            decimal_a: None,
            decimal_b: None,
        })
    }
}
