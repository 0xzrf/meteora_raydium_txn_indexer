use crate::constants::RAYDIUM_CPMM_PUBKEY;
use crate::parser::{Actions, CopyReturnType, CopyTradeParseData, ToCopyTradeField, TokenTxnInfo};
pub struct RaydiumCpmmCopyTradingProgram {
    pub program_id: String,
    pub account_keys: Vec<String>,
    pub action: Option<Actions>,
    pub disc: [u8; 8],
    pub txn: String,
    pub token_transfers: Vec<TokenTxnInfo>,
}

impl RaydiumCpmmCopyTradingProgram {
    pub fn new(
        account_keys: Vec<String>,
        txn: String,
        ix_data: Vec<u8>,
        transfers: Vec<TokenTxnInfo>,
    ) -> Self {
        let (disc, _) = ix_data.split_at(8);
        let disc: [u8; 8] = disc.try_into().unwrap_or([0; 8]);

        RaydiumCpmmCopyTradingProgram {
            program_id: RAYDIUM_CPMM_PUBKEY.to_string(),
            account_keys,
            action: None,
            disc,
            txn,
            token_transfers: transfers,
        }
    }

    // Check to see if the instruciton initialized by the user is actually the one we want
    pub fn set_action_type(mut self) -> Self {
        match self.disc {
            Self::ADD_LIQUIDITY_IX => {
                self.action = Some(Actions::AddLiquidity);
            }

            Self::REMOVE_LIQUIDITY_IX => self.action = Some(Actions::RemoveLiquidity),
            _ => {}
        }

        self
    }

    pub fn get_parsed_data(self) -> Option<(CopyReturnType, String)> {
        if let Some(a) = &self.action {
            let owner = self.account_keys[0].clone();
            match a {
                Actions::AddLiquidity => {
                    println!("add liquidity triggered by user");
                    Some((self.add_liquidity(owner.clone()), owner))
                }
                Actions::RemoveLiquidity => {
                    println!("Remove liquidity initialized");

                    Some((self.remove_liquidity(owner.clone()), owner))
                    // None
                }
                _ => None, // This line will never reach since there's no close_position, create_position or claim_fee ix
            }
        } else {
            None
        }
    }
}

impl ToCopyTradeField for RaydiumCpmmCopyTradingProgram {
    const ADD_LIQUIDITY_IX: [u8; 8] = [242, 35, 198, 137, 82, 225, 242, 182];
    const REMOVE_LIQUIDITY_IX: [u8; 8] = [183, 18, 70, 156, 148, 109, 161, 34];
    const CLOSE_POSITION_IX: crate::parser::Disc = [0; 8];
    const CREATE_POSITION_IX: crate::parser::Disc = [0; 8];

    fn add_liquidity(self, owner: String) -> CopyReturnType {
        println!("add liquidity triggered by user");

        let keys = &self.account_keys;

        let mint_a = keys[10].clone();
        let mint_b = keys[11].clone();
        let pool_address = keys[2].clone();

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

        Ok(CopyTradeParseData {
            action: self.action.unwrap().as_str(),
            contract_address: self.program_id.to_string(),
            amount_a,
            owner,
            amount_b,
            pool_address,
            position_nft: None,
            token_a: mint_a,
            token_b: mint_b,
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

        let pool_address = keys[2].clone();
        let token_a = keys[10].clone();
        let token_b = keys[11].clone();

        let user_token_account_a = keys[4].clone();
        let user_token_account_b = keys[5].clone();

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

        Ok(CopyTradeParseData {
            pool_address,
            token_a,
            owner,
            token_b,
            contract_address: self.program_id,
            amount_a,
            amount_b,
            position_nft: None,
            action: self.action.unwrap().as_str(),
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

    fn close_position(self, _owner: String) -> CopyReturnType {
        // never gonna run
        todo!()
    }

    fn create_position(self, _owner: String) -> CopyReturnType {
        todo!()
    }
}
