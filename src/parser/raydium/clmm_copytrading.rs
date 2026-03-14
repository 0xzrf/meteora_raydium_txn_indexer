use crate::constants::NOT_IN_TX;
use crate::parser::{
    Actions, CopyReturnType, CopyTradeParseData, ToCopyTradeField, token_transfer::TokenTxnInfo,
};
use crate::{constants::RAYDIUM_CLMM_PUBKEY, helper::is_debugging};

pub struct RaydiumClmmCopyTradingProgram {
    pub program_id: String,
    pub account_keys: Vec<String>,
    pub ix_data: Vec<u8>,
    pub action: Option<Actions>,
    pub txn: String,
    pub disc: [u8; 8],
    pub token_transfers: Vec<TokenTxnInfo>,
}

#[derive(Debug)]
struct AddLiquidityArgs {
    tick_lower_index: i32,
    tick_upper_index: i32,
    _tick_array_lower_start_index: i32,
    _tick_array_upper_start_index: i32,
    _liquidity: u128,
    _amount_0_max: u64,
    _amount_1_max: u64,
    _with_metadata: bool,
    _base_flag: Option<bool>,
}

impl RaydiumClmmCopyTradingProgram {
    const OPEN_POSITION_NFT_2022_IX: [u8; 8] = [77, 255, 174, 82, 125, 29, 201, 46]; // open_position_with_token22_nft
    pub fn new(
        ix_data: Vec<u8>,
        account_keys: Vec<String>,
        txn: String,
        token_transfers: Vec<TokenTxnInfo>,
    ) -> Self {
        let (disc, ix_args) = ix_data.split_at(8);

        let disc: [u8; 8] = disc.try_into().unwrap_or([0; 8]);

        let ix_args = ix_args.to_vec();

        RaydiumClmmCopyTradingProgram {
            program_id: RAYDIUM_CLMM_PUBKEY.to_string(),
            account_keys,
            action: None,
            txn,
            ix_data: ix_args,
            disc,
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
            Self::CREATE_POSITION_IX | Self::OPEN_POSITION_NFT_2022_IX => {
                self.action = Some(Actions::CreatePosition)
            }
            Self::CLOSE_POSITION_IX => self.action = Some(Actions::ClosePosition),
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
                }
                Actions::ClosePosition => {
                    println!("close position initialized");

                    Some((self.close_position(owner.clone()), owner))
                }
                Actions::CreatePosition => {
                    if self.disc.eq(&Self::CREATE_POSITION_IX) {
                        println!("create position initialized");
                        Some((self.create_position(owner.clone()), owner))
                    } else {
                        println!("create position nft token_2022 initialized");
                        Some((self.create_position_nft_token2022(owner.clone()), owner))
                    }
                }
                _ => None,
            }
        } else {
            println!("user initialized an ix");
            None
        }
    }

    pub fn create_position_nft_token2022(self, owner: String) -> CopyReturnType {
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

        let AddLiquidityArgs {
            tick_lower_index,
            tick_upper_index,
            _tick_array_lower_start_index: _,
            _tick_array_upper_start_index: _,
            _liquidity: _,
            _amount_0_max: _,
            _amount_1_max: _,
            _with_metadata: _,
            _base_flag: _,
        } = Self::parse_add_liquidity(&self.ix_data)?;

        Ok(CopyTradeParseData {
            contract_address: self.program_id,
            owner,
            amount_a,
            amount_b,
            position_nft,
            token_a,
            token_b,
            pool_address,
            txn_sig: self.txn,
            max_bin_id: tick_upper_index,
            min_bin_id: tick_lower_index,
            action: self.action.unwrap().as_str(),
            strategy: None,
            decimal_a: None,
            decimal_b: None,
            token_a_price: None,
            token_b_price: None,
        })
    }

    fn parse_add_liquidity(data: &[u8]) -> Result<AddLiquidityArgs, &'static str> {
        // sizes: 4*4 + 16 + 8 + 8 + 1 + 1..2 = 50..51 bytes minimum
        if data.len() < 4 * 4 + 16 + 8 + 8 + 1 + 1 {
            return Err("input too short");
        }

        let mut off = 0;
        let read_i32 = |d: &[u8], o: &mut usize| {
            let s: [u8; 4] = d[*o..*o + 4].try_into().unwrap();
            *o += 4;
            i32::from_le_bytes(s)
        };
        let read_u64 = |d: &[u8], o: &mut usize| {
            let s: [u8; 8] = d[*o..*o + 8].try_into().unwrap();
            *o += 8;
            u64::from_le_bytes(s)
        };
        let read_u128 = |d: &[u8], o: &mut usize| {
            let s: [u8; 16] = d[*o..*o + 16].try_into().unwrap();
            *o += 16;
            u128::from_le_bytes(s)
        };

        let tick_lower_index = read_i32(data, &mut off);
        let tick_upper_index = read_i32(data, &mut off);
        let tick_array_lower_start_index = read_i32(data, &mut off);
        let tick_array_upper_start_index = read_i32(data, &mut off);
        let liquidity = read_u128(data, &mut off);
        let amount_0_max = read_u64(data, &mut off);
        let amount_1_max = read_u64(data, &mut off);

        let with_metadata = {
            if off >= data.len() {
                return Err("missing with_metadata");
            }
            let v = data[off] != 0;
            off += 1;
            v
        };

        // Option<bool>: tag byte then optional bool byte
        if off >= data.len() {
            return Err("missing option tag for base_flag");
        }
        let base_flag = match data[off] {
            0 => {
                off += 1;
                None
            }
            1 => {
                off += 1;
                if off >= data.len() {
                    return Err("missing bool for Option<bool>");
                }
                let v = data[off] != 0;
                off += 1;
                Some(v)
            }
            _ => return Err("invalid option tag"),
        };

        Ok(AddLiquidityArgs {
            tick_lower_index,
            tick_upper_index,
            _tick_array_lower_start_index: tick_array_lower_start_index,
            _tick_array_upper_start_index: tick_array_upper_start_index,
            _liquidity: liquidity,
            _amount_0_max: amount_0_max,
            _amount_1_max: amount_1_max,
            _with_metadata: with_metadata,
            _base_flag: base_flag,
        })
    }
}

impl ToCopyTradeField for RaydiumClmmCopyTradingProgram {
    const REMOVE_LIQUIDITY_IX: [u8; 8] = [58, 127, 188, 62, 79, 82, 196, 96]; // decrease_liquidity_v2
    const ADD_LIQUIDITY_IX: [u8; 8] = [133, 29, 89, 223, 69, 238, 176, 10]; // increase_liquidity_v2
    const CREATE_POSITION_IX: [u8; 8] = [77, 184, 74, 214, 112, 86, 241, 199]; // open_position_v2
    const CLOSE_POSITION_IX: [u8; 8] = [123, 134, 81, 0, 49, 68, 98, 98]; // close_position

    fn add_liquidity(self, owner: String) -> CopyReturnType {
        let keys = &self.account_keys;

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

        Ok(CopyTradeParseData {
            contract_address: self.program_id,
            owner,
            amount_a,
            amount_b,
            position_nft,
            token_a,
            token_b,
            pool_address,
            txn_sig: self.txn,
            max_bin_id: 0,
            min_bin_id: 0,
            action: self.action.unwrap().as_str(),
            strategy: None,
            decimal_a: None,
            decimal_b: None,
            token_a_price: None,
            token_b_price: None,
        })
    }

    fn remove_liquidity(self, owner: String) -> CopyReturnType {
        let keys = &self.account_keys;

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

        if amount_a == 0 || amount_b == 0 {
            println!("Couldn't capture amounts");
        }

        Ok(CopyTradeParseData {
            contract_address: self.program_id,
            amount_a,
            amount_b,
            owner,
            position_nft,
            token_a,
            token_b,
            pool_address,
            txn_sig: self.txn,
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

    fn close_position(self, owner: String) -> CopyReturnType {
        let keys = &self.account_keys;

        if keys.len() < 4 {
            return Err("CLMM ERR: key length too short");
        }

        let pool_address = "NOT_IN_IX".to_string();
        let token_a = NOT_IN_TX.to_string();
        let token_b = NOT_IN_TX.to_string();

        let position_nft = Some(keys[3].clone());

        let amount_a = 0;
        let amount_b = 0;

        Ok(CopyTradeParseData {
            contract_address: self.program_id,
            amount_a,
            owner,
            amount_b,
            position_nft,
            token_a,
            token_b,
            pool_address,
            txn_sig: self.txn,
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
    fn create_position(self, owner: String) -> CopyReturnType {
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

        let ix_parsed = Self::parse_add_liquidity(&self.ix_data)?;

        Ok(CopyTradeParseData {
            contract_address: self.program_id,
            amount_a,
            owner,
            amount_b,
            position_nft,
            token_a,
            token_b,
            pool_address,
            txn_sig: self.txn,
            max_bin_id: ix_parsed.tick_upper_index,
            min_bin_id: ix_parsed.tick_lower_index,
            strategy: None,
            action: self.action.unwrap().as_str(),
            decimal_a: None,
            decimal_b: None,
            token_a_price: None,
            token_b_price: None,
        })
    }
}

#[cfg(test)]
pub mod test_clmm_copy_trade {
    use crate::parser::raydium::RaydiumClmmCopyTradingProgram;

    #[test]
    pub fn get_add_liquidity_parsed_data_clmm() {
        // {
        //     "tick_lower_index": {
        //       "type": "i32",
        //       "data": "-19871"
        //     },
        //     "tick_upper_index": {
        //       "type": "i32",
        //       "data": "-19304"
        //     },
        //     "amount_0_max": {
        //       "type": "u64",
        //       "data": "6088503"
        //     },
        //     "amount_1_max": {
        //       "type": "u64",
        //       "data": "1000000"
        //   }
        let hex_str = "4dffae527d1dc92e61b2ffff98b4ffff30b2ffff88b4ffff0000000000000000000000000000000037e75c000000000040420f0000000000010100";

        let vec = (0..hex_str.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex_str[i..i + 2], 16).unwrap())
            .collect::<Vec<u8>>();

        let split_point = 8;

        let (_, data) = vec.split_at(split_point);

        match RaydiumClmmCopyTradingProgram::parse_add_liquidity(data) {
            Ok(val) => {
                assert!(val._amount_0_max.eq(&6088503));
                assert!(val._amount_1_max.eq(&1000000));
                assert!(val.tick_lower_index.eq(&-19871));
                assert!(val.tick_upper_index.eq(&-19304));
            }
            Err(_) => panic!(),
        }
    }
}
