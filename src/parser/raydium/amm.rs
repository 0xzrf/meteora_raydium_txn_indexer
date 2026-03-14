use crate::{
    constants::{NOT_IN_TX, RAYDIUM_AMM_DEVNET_PUBKEY, RAYDIUM_AMM_PUBKEY},
    helper::{is_debugging, is_devnet},
    parser::{Actions, ParseConfigs, ParseData, PoolReturnType, token_transfer::TokenTxnInfo},
};
pub struct RaydiumAmmProgram {
    pub program_id: String,
    pub account_keys: Vec<String>,
    pub disc: u8,
    pub action: Option<Actions>,
    pub txn: String,
    pub token_transfers: Vec<TokenTxnInfo>,
}

impl RaydiumAmmProgram {
    pub const ADD_LIQUIDITY_IX: u8 = 3;
    pub const REMOVE_LIQUIDITY_IX: u8 = 4;

    pub fn new(configs: ParseConfigs) -> Self {
        let program_id = if is_devnet() {
            RAYDIUM_AMM_DEVNET_PUBKEY.to_string()
        } else {
            RAYDIUM_AMM_PUBKEY.to_string()
        };
        let ParseConfigs {
            txn,
            ix_accounts,
            ix_data,
            token_transfers,
        } = configs;

        let mut disc = 0;

        if let Some((discriminator, _)) = ix_data.split_first() {
            disc = *discriminator;
        }

        RaydiumAmmProgram {
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
            Self::REMOVE_LIQUIDITY_IX => self.action = Some(Actions::RemoveLiquidity),
            _ => {}
        }

        self
    }

    pub fn get_parsed_data(self) -> Option<(PoolReturnType, String)> {
        if let Some(a) = &self.action {
            match a {
                Actions::AddLiquidity => {
                    let owner = self.account_keys[12].clone();
                    println!("add liquidity triggered by user");
                    Some((self.add_liquidity(), owner))
                }
                Actions::RemoveLiquidity => {
                    println!("Remove liquidity initialized");
                    let owner = self.account_keys[18].clone();
                    Some((self.remove_liquidity(), owner))
                }
                _ => None,
            }
        } else {
            None
        }
    }

    pub fn add_liquidity(self) -> PoolReturnType {
        let keys = &self.account_keys;

        if keys.len() < 14 {
            return Err("CPMM add liquidity has invalid account length");
        }

        let mint_a = NOT_IN_TX.to_string();
        let mint_b = NOT_IN_TX.to_string();
        let pool_address = keys[1].clone();

        let position_nft = Some(keys[11].clone());

        let user_token_account_a = keys[9].clone();
        let user_token_account_b = keys[10].clone();

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

    pub fn remove_liquidity(self) -> PoolReturnType {
        let keys = &self.account_keys;

        if keys.len() < 22 {
            return Err("CPMM add liquidity has invalid account length");
        }

        let mint_a = NOT_IN_TX.to_string();
        let mint_b = NOT_IN_TX.to_string();
        let pool_address = keys[1].clone();

        let user_token_account_a = keys[16].clone();
        let user_token_account_b = keys[17].clone();

        let position_nft = Some(keys[13].clone());

        let pool_token_account_a = keys[6].clone();
        let pool_token_account_b = keys[7].clone();

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
