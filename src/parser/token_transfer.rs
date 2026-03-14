use crate::constants::{TOKEN_2022_PROGRAM, TOKEN_PROGRAM};
use helius_laserstream::solana::storage::confirmed_block::InnerInstructions;
use spl_token::instruction::TokenInstruction;

#[derive(Debug)]
pub struct TokenTxnInfo {
    pub from: String,
    pub to: String,
    pub amount: u64,
}

impl TokenTxnInfo {
    pub fn new(from: String, to: String, amount: u64) -> Self {
        TokenTxnInfo { from, to, amount }
    }
}

pub fn get_token_transfers_for_ix_index(
    inner_ix: &Vec<InnerInstructions>,
    account_keys: &[String],
    index: u32,
) -> Vec<TokenTxnInfo> {
    let mut token_transfers: Vec<TokenTxnInfo> = vec![];
    for inner in inner_ix {
        if inner.index != index {
            continue;
        }

        for ix in &inner.instructions {
            let program_id = &account_keys[ix.program_id_index as usize];

            if program_id != TOKEN_PROGRAM && program_id != TOKEN_2022_PROGRAM {
                continue;
            }

            // let (disc, _) = ix.data.split_first().unwrap(); // if it's a token program account, it'll always have at least 1 ix data

            // // checks if the ix was a token transfer
            // if disc.ne(&12) {
            //     continue;
            // }

            match TokenInstruction::unpack(&ix.data) {
                Ok(token_ix) => {
                    // resolve accounts → pubkeys
                    let accounts: Vec<String> = ix
                        .accounts
                        .iter()
                        .map(|idx| account_keys[*idx as usize].clone())
                        .collect();

                    match token_ix {
                        TokenInstruction::Transfer { amount } => {
                            // SPL Token Transfer layout:
                            // 0 = source
                            // 1 = destination
                            // 2 = authority
                            let from = accounts[0].clone();
                            let to = accounts[1].clone();

                            let token_tx_info = TokenTxnInfo::new(from, to, amount);

                            token_transfers.push(token_tx_info);
                        }

                        TokenInstruction::TransferChecked { amount, .. } => {
                            // TransferChecked layout:
                            // 0 = source
                            // 1 = mint
                            // 2 = destination
                            // 3 = authority
                            let from = accounts[0].clone();
                            let to = accounts[2].clone();

                            let token_tx_info = TokenTxnInfo::new(from, to, amount);

                            token_transfers.push(token_tx_info);
                        }

                        _ => {
                            // ignore approves, burns, mints, etc
                        }
                    }
                }
                Err(err) => {
                    println!("Failed to unpack token instruction: {err}");
                }
            }
        }
    }
    token_transfers
}
