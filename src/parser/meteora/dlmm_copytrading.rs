use crate::constants::{METEORA_DLMM_PUBKEY, NOT_IN_TX};
use crate::parser::{
    Actions, CopyReturnType, CopyTradeParseData, ToCopyTradeField, token_transfer::TokenTxnInfo,
};
use bytemuck::{Pod, Zeroable};

pub struct MeteoraDlmmCopyTradeProgram {
    pub program_id: String,
    pub disc: [u8; 8],
    pub ix_data: Vec<u8>,
    pub account_keys: Vec<String>,
    pub action: Option<Actions>,
    pub txn: String,
    pub token_transfers: Vec<TokenTxnInfo>,
}

#[repr(C, packed)]
#[derive(Pod, Zeroable, Clone, Copy, Debug)]
pub struct AddLiqduidityParams {
    pub amount_x: u64,
    pub amount_y: u64,
    pub active_id: i32,
    pub max_active_bin_slippage: i32,
    pub strategy_parameters: StrategyParameters,
}

#[repr(C, packed)]
#[derive(Copy, Clone, Debug, Zeroable, Pod)]
struct RemoveLiquidityHeader {
    from_bin_id: i32,   // 4
    to_bin_id: i32,     // 4
    bps_to_remove: u16, // 2
    slices_len: u32,    // 4
} // total: 14 bytes (packed -> no padding)

#[derive(Debug)]
pub struct RemoveLiquidityParsed {
    pub from_bin_id: i32,
    pub to_bin_id: i32,
    pub bps_to_remove: u16,
    pub slice_lengths: Vec<u16>,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Zeroable, Pod)]
pub struct RebalanceHeader {
    pub active_id: i32,               // 4
    pub max_active_bin_slippage: u16, // 2
    pub should_claim_fee: u8,         // 1 (was a bool in JSON; on-chain it's a byte)
    pub should_claim_reward: u8,      // 1
    pub min_withdraw_x_amount: u64,   // 8
    pub max_deposit_x_amount: u64,    // 8
    pub min_withdraw_y_amount: u64,   // 8
    pub max_deposit_y_amount: u64,    // 8
    pub shrink_mode: u8,              // 1
    pub padding: [u8; 31],            // 31
    pub removes_len: u32,             // 4  (vec length for removes)
    pub adds_len: u32,                // 4  (vec length for adds)
}

#[repr(C, packed)]
#[derive(Copy, Clone, Debug, Zeroable, Pod)]
pub struct AddLiquidityParamsPod {
    pub min_delta_id: i32,        // 4
    pub max_delta_id: i32,        // 4
    pub x0: u64,                  // 8
    pub y0: u64,                  // 8
    pub delta_x: u64,             // 8
    pub delta_y: u64,             // 8
    pub bit_flag: u8,             // 1
    pub favor_x_in_active_id: u8, // 1 (was bool in JSON; on-chain it's a byte)
    pub padding: [u8; 16],        // 16
}

#[repr(C, packed)]
#[derive(Pod, Zeroable, Clone, Copy, Debug)]
pub struct StrategyParameters {
    pub min_bin_id: i32,
    pub max_bin_id: i32,
    pub strategy_type: u8,
    pub parameters: [u8; 64],
}

#[derive(Debug)]
pub enum Strats {
    Spot = 1,
    Curve = 2,
    BidAsk = 3,
}

impl From<Strats> for u8 {
    fn from(value: Strats) -> Self {
        match value {
            Strats::BidAsk => 3,
            Strats::Curve => 2,
            Strats::Spot => 1,
        }
    }
}

impl MeteoraDlmmCopyTradeProgram {
    pub const REBALANCE_LIQUIDITY: [u8; 8] = [92, 4, 176, 193, 119, 185, 83, 9];
    const CLAIM_FEE_IX: [u8; 8] = [112, 191, 101, 171, 28, 144, 127, 187]; // claim_fee2
    pub fn new(
        ix_data: Vec<u8>,
        account_keys: Vec<String>,
        txn: String,
        token_transfers: Vec<TokenTxnInfo>,
    ) -> Self {
        let (disc, ix_args) = ix_data.split_at(8);

        let disc: [u8; 8] = disc.try_into().unwrap_or([0; 8]);

        let ix_args = ix_args.to_vec();

        MeteoraDlmmCopyTradeProgram {
            program_id: METEORA_DLMM_PUBKEY.to_string(),
            disc,
            ix_data: ix_args,
            account_keys,
            action: None,
            txn,
            token_transfers,
        }
    }

    pub fn set_action_type(mut self) -> Self {
        match self.disc {
            Self::ADD_LIQUIDITY_IX | Self::REBALANCE_LIQUIDITY => {
                self.action = Some(Actions::AddLiquidity);
            }
            Self::REMOVE_LIQUIDITY_IX => self.action = Some(Actions::RemoveLiquidity),
            Self::CREATE_POSITION_IX => self.action = Some(Actions::CreatePosition),
            Self::CLOSE_POSITION_IX => self.action = Some(Actions::ClosePosition),
            Self::CLAIM_FEE_IX => self.action = Some(Actions::ClaimFee),
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

                    if self.disc.eq(&Self::ADD_LIQUIDITY_IX) {
                        Some((self.add_liquidity(owner.clone()), owner))
                    } else {
                        Some((self.rebalance_liquidity(owner.clone()), owner))
                    }
                }
                Actions::ClaimFee => {
                    println!("Claim fee initialized");
                    let owner = self.account_keys[2].clone();
                    Some((self.claim_fee(owner.clone()), owner))
                }
                Actions::RemoveLiquidity => {
                    println!("Remove liquidity initialized");
                    let owner = self.account_keys[9].clone();
                    Some((self.remove_liquidity(owner.clone()), owner))
                }
                Actions::CreatePosition => {
                    let owner = self.account_keys[3].clone();
                    println!("create position initialized");
                    Some((self.create_position(owner.clone()), owner))
                }
                Actions::ClosePosition => {
                    println!("close position initialized");
                    let owner = self.account_keys[1].clone();
                    Some((self.close_position(owner.clone()), owner))
                }
                _ => None,
            }
        } else {
            println!("user initialized an ix");
            None
        }
    }

    pub fn parse_rebalance_ix(bytes: Vec<u8>) -> Result<Strats, &'static str> {
        // 2) remove Anchor discriminator (first 8 bytes)

        let data = &bytes;

        // 3) parse fixed-size header using bytemuck
        if data.len() < size_of::<RebalanceHeader>() {
            return Err("data shorter than header");
        }
        let header: &RebalanceHeader =
            bytemuck::try_from_bytes(&data[..size_of::<RebalanceHeader>()]).unwrap(); // hndle

        // 4) parse `adds` entries (manual because it's a vec)
        let mut offset = size_of::<RebalanceHeader>();
        let mut adds = Vec::new();
        for i in 0..header.adds_len {
            let start = offset;
            let end = start + size_of::<AddLiquidityParamsPod>();
            if data.len() < end {
                return Err("unexpected EOF while reading add");
            }
            let pod = bytemuck::try_from_bytes::<AddLiquidityParamsPod>(&data[start..end]).unwrap(); // TODO handle this
            adds.push(*pod);
            offset = end;
        }

        if adds.is_empty() {
            return Err("adds value empty");
        }

        let adds_val = adds[0];

        let y0 = adds_val.y0;
        let x0 = adds_val.x0;
        let delta_x = adds_val.delta_x;
        let delta_y = adds_val.delta_y;

        println!("y0: {y0}, x0: {x0}, delta_x: {delta_x}, delta_y: {delta_y}");

        if delta_x == 0 && delta_y == 0 {
            Ok(Strats::Spot) // spot
        } else if x0 == 0 || y0 == 0 {
            Ok(Strats::BidAsk) // curve
        } else {
            Ok(Strats::Curve)
        }
    }

    pub fn claim_fee(self, owner: String) -> CopyReturnType {
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

        Ok(CopyTradeParseData {
            contract_address: self.program_id,
            amount_a,
            amount_b,
            position_nft,
            token_a,
            token_b,
            pool_address,
            txn_sig: self.txn,
            max_bin_id: 0,
            owner,
            min_bin_id: 0,
            strategy: None,
            action: self.action.unwrap().as_str(),
            decimal_a: None,
            decimal_b: None,
            token_a_price: None,
            token_b_price: None,
        })
    }
    pub fn rebalance_liquidity(self, owner: String) -> CopyReturnType {
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

        let strat = match Self::parse_rebalance_ix(self.ix_data) {
            Ok(val) => Some(val.into()),
            Err(e) => {
                println!("Error occured; {e}");
                None
            }
        };

        if amount_a == 0 || amount_b == 0 {
            println!("Couldn't get the data");
        }

        Ok(CopyTradeParseData {
            contract_address: self.program_id,
            amount_a,
            amount_b,
            position_nft,
            token_a,
            token_b,
            pool_address,
            txn_sig: self.txn,
            max_bin_id: 0,
            owner,
            min_bin_id: 0,
            strategy: strat,
            action: self.action.unwrap().as_str(),
            decimal_a: None,
            decimal_b: None,
            token_a_price: None,
            token_b_price: None,
        })
    }

    pub fn to_add_liquidity_params(&self) -> Result<AddLiqduidityParams, &'static str> {
        let data_len: usize = core::mem::size_of::<AddLiqduidityParams>();

        let (ix, _) = self.ix_data.split_at(data_len);

        let data = bytemuck::try_from_bytes::<AddLiqduidityParams>(ix);

        match data {
            Ok(arg) => Ok(*arg),
            Err(_) => Err("Couldn't convert the ix to add liquidity params"),
        }
    }

    pub fn parse_remove_liquidity(data: &[u8]) -> Result<RemoveLiquidityParsed, &'static str> {
        const HEADER_SIZE: usize = std::mem::size_of::<RemoveLiquidityHeader>(); // 14

        if data.len() < HEADER_SIZE {
            return Err("buffer too small for header");
        }

        // Zero-copy cast the first 14 bytes into the header
        let header: &RemoveLiquidityHeader =
            bytemuck::try_from_bytes(&data[..HEADER_SIZE]).map_err(|_| "bytemuck error")?;

        // Convert numeric fields from little-endian to native
        let from_bin_id = i32::from_le(header.from_bin_id);
        let to_bin_id = i32::from_le(header.to_bin_id);
        let bps_to_remove = u16::from_le(header.bps_to_remove);
        let slices_len = u32::from_le(header.slices_len) as usize;

        // compute required length for slice entries (each is u16)
        let needed = HEADER_SIZE
            .checked_add(slices_len.checked_mul(2).ok_or("slice count overflow")?)
            .ok_or("size overflow")?;

        if data.len() < needed {
            return Err("buffer too small");
        }

        // Read slice lengths (each u16 LE)
        let mut slice_lengths = Vec::with_capacity(slices_len);
        let mut off = HEADER_SIZE;
        for _ in 0..slices_len {
            let bytes: [u8; 2] = data[off..off + 2].try_into().unwrap();
            slice_lengths.push(u16::from_le_bytes(bytes));
            off += 2;
        }

        Ok(RemoveLiquidityParsed {
            from_bin_id,
            to_bin_id,
            bps_to_remove,
            slice_lengths,
        })
    }
}

impl ToCopyTradeField for MeteoraDlmmCopyTradeProgram {
    const REMOVE_LIQUIDITY_IX: [u8; 8] = [204, 2, 195, 145, 53, 145, 145, 205]; // remove_liquidity_by_range
    const ADD_LIQUIDITY_IX: [u8; 8] = [3, 221, 149, 218, 111, 141, 118, 213]; // AddLiquidityByStrategy2
    const CLOSE_POSITION_IX: [u8; 8] = [174, 90, 35, 115, 186, 40, 147, 226]; // close_position2
    const CREATE_POSITION_IX: [u8; 8] = [219, 192, 234, 71, 190, 191, 102, 80]; // initialize_position2

    fn add_liquidity(self, owner: String) -> CopyReturnType {
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

        let params = self.to_add_liquidity_params()?; // This function only hits when addLiquidity is called

        let min_bin_id = params.strategy_parameters.min_bin_id;
        let max_bin_id = params.strategy_parameters.max_bin_id;
        let strategy = params.strategy_parameters.strategy_type - 6;

        Ok(CopyTradeParseData {
            contract_address: self.program_id,
            amount_a,
            amount_b,
            position_nft,
            token_a,
            token_b,
            pool_address,
            txn_sig: self.txn,
            max_bin_id,
            owner,
            min_bin_id,
            strategy: Some(strategy),
            action: self.action.unwrap().as_str(),
            decimal_a: None,
            decimal_b: None,
            token_a_price: None,
            token_b_price: None,
        })
    }

    fn remove_liquidity(self, owner: String) -> CopyReturnType {
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
            &pool_token_account_a,
            &user_token_account_a,
            &self.token_transfers,
        );
        let amount_b: u64 = Self::get_token_transfer_for_user(
            &pool_token_account_b,
            &user_token_account_b,
            &self.token_transfers,
        );
        let RemoveLiquidityParsed {
            from_bin_id,
            to_bin_id,
            bps_to_remove: _,
            slice_lengths: _,
        } = Self::parse_remove_liquidity(&self.ix_data)?;

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
            max_bin_id: to_bin_id,
            min_bin_id: from_bin_id,
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

        let token_a = NOT_IN_TX.to_string();
        let token_b = NOT_IN_TX.to_string();
        let pool_address = NOT_IN_TX.to_string();

        let position_nft = Some(keys[0].clone());
        let amount_a: u64 = 0;
        let amount_b: u64 = 0;

        Ok(CopyTradeParseData {
            contract_address: self.program_id,
            amount_a,
            amount_b,
            position_nft,
            token_a,
            token_b,
            pool_address,
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

    fn create_position(self, owner: String) -> CopyReturnType {
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
pub mod test_data_deserialization {
    use super::*;

    #[test]
    pub fn test_get_deserialized_data_dlmm_add_liquidity() {
        let data_len: usize = core::mem::size_of::<AddLiqduidityParams>();

        let hex_str = "03dd95da6f8d76d540420f0000000000881d050000000000ddfdffff01000000cffdffff13feffff06000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000200000000000100";

        let vec = (0..hex_str.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex_str[i..i + 2], 16).unwrap())
            .collect::<Vec<u8>>();

        let split_point = 8 + data_len;

        let (data, _) = vec.split_at(split_point);

        let (_, ix_data) = data.split_at(8);

        let data = bytemuck::try_from_bytes::<AddLiqduidityParams>(ix_data);

        match data {
            Ok(val) => {
                let amount_x = val.amount_x;
                let amount_y = val.amount_y;

                let min_bin_id = val.strategy_parameters.min_bin_id;
                let max_bin_id = val.strategy_parameters.max_bin_id;
                let strat = val
                    .strategy_parameters
                    .strategy_type
                    .checked_sub(6)
                    .expect("Strat underflow occured");

                assert!(amount_x.eq(&1000000));
                assert!(amount_y.eq(&335240));
                assert!(max_bin_id.eq(&-493));
                assert!(min_bin_id.eq(&-561));
                assert!(strat.eq(&0));
            }
            Err(_) => {
                println!("Couuldn't deserialize");
            }
        }
    }

    #[test]
    pub fn test_get_deserialized_data_remove_liquidity() {
        let hex_str = "cc02c391359191cdececffff0bedffff10270200000000000100";

        let vec = (0..hex_str.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex_str[i..i + 2], 16).unwrap())
            .collect::<Vec<u8>>();

        let split_point = 8;

        let (_, data) = vec.split_at(split_point);

        let parsed_remove_data = MeteoraDlmmCopyTradeProgram::parse_remove_liquidity(data);

        match parsed_remove_data {
            Ok(val) => println!("{val:#?}"),
            Err(reason) => println!("{reason}"),
        }
    }

    #[test]
    pub fn test_rebalance_liquidity_deserialization_works() {
        let hex_str = "5c04b0c177b95309bbebffff2600000000000000000000000ada6d00000000000000000000000000d4100f000000000000000000000000000000000000000000000000000000000000000000000000000000000001000000dfffffff2300000088cd00000000000003d9000000000000df0500000000000061060000000000000c00000000000000000000000000000000000200000000000100";
        let vec = (0..hex_str.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex_str[i..i + 2], 16).unwrap())
            .collect::<Vec<u8>>();

        let (_, ix_data) = vec.split_at(8);

        match MeteoraDlmmCopyTradeProgram::parse_rebalance_ix(ix_data.to_vec()) {
            Ok(val) => {
                println!("val: {:#?}", val)
            }
            Err(e) => println!("{e}"),
        }
    }
}
