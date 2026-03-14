use bs58::decode;
use helius_laserstream::solana::storage::confirmed_block::{InnerInstruction, InnerInstructions};
use llp_indexer::parser::token_transfer::{TokenTxnInfo, get_token_transfers_for_ix_index};
use solana_client::rpc_client::RpcClient;
use solana_client::rpc_config::RpcTransactionConfig;
use solana_client::rpc_response::{OptionSerializer, transaction::Signature};
use solana_transaction_status::{
    EncodedConfirmedTransactionWithStatusMeta, EncodedTransaction, UiCompiledInstruction,
    UiInnerInstructions, UiInstruction, UiMessage,
};
use std::str::FromStr;

fn retrun_accounts_and_ix(
    txn: &EncodedConfirmedTransactionWithStatusMeta,
) -> Option<(Vec<String>, Vec<UiCompiledInstruction>)> {
    match &txn.transaction.transaction {
        EncodedTransaction::Json(ui_tx) => match &ui_tx.message {
            UiMessage::Raw(raw_msg) => {
                Some((raw_msg.account_keys.clone(), raw_msg.instructions.clone()))
            }
            _ => None,
        },
        _ => None,
    }
}

#[derive(Debug)]
pub struct ParseConfigs {
    pub txn: String,
    pub ix_accounts: Vec<String>,
    pub program_id: String,
    pub token_transfers: Vec<TokenTxnInfo>,
    pub ix_data: Vec<u8>,
}

fn to_inner_ix(inner: &Vec<UiInstruction>) -> Vec<InnerInstruction> {
    let mut inner_ix: Vec<InnerInstruction> = vec![];

    for ix in inner {
        match ix {
            UiInstruction::Compiled(compiled) => {
                let data = InnerInstruction {
                    accounts: compiled.accounts.clone(),
                    program_id_index: compiled.program_id_index as u32,
                    data: decode(compiled.data.clone()).into_vec().unwrap(),
                    stack_height: compiled.stack_height,
                };

                inner_ix.push(data);
            }
            UiInstruction::Parsed(_) => println!("Parsed inner ix not supported yet"),
        }
    }

    inner_ix
}

fn to_inner_ixs(inner: &Vec<UiInnerInstructions>) -> Vec<InnerInstructions> {
    let mut inner_ixs: Vec<InnerInstructions> = vec![];

    for ixs in inner {
        let inner_ix = to_inner_ix(&ixs.instructions);

        inner_ixs.push(InnerInstructions {
            index: ixs.index as u32,
            instructions: inner_ix,
        });
    }

    inner_ixs
}

pub fn fetch_txn(sig: &str, url: &str, integrated_protocols: &[String]) -> Vec<ParseConfigs> {
    let client = RpcClient::new(url);
    let sign = Signature::from_str(sig).unwrap();

    let results = client.get_transaction_with_config(
        &sign,
        RpcTransactionConfig {
            max_supported_transaction_version: Some(0),
            ..Default::default()
        },
    );

    let mut parsed_txns: Vec<ParseConfigs> = vec![];

    match results {
        Ok(txn) => {
            let (mut account_keys, ixs) = retrun_accounts_and_ix(&txn).unwrap();
            let meta = txn.transaction.meta.unwrap();

            if let OptionSerializer::Some(data) = &meta.loaded_addresses {
                for key in data.writable.iter().chain(data.readonly.iter()) {
                    if !account_keys.contains(key) {
                        account_keys.push(key.clone());
                    }
                }
            }

            for (index, ix) in ixs.iter().enumerate() {
                let program_id = &account_keys[ix.program_id_index as usize];

                if integrated_protocols.contains(program_id) {
                    let ix_accounts: Vec<String> = ix
                        .accounts
                        .iter()
                        .map(|idx| account_keys[*idx as usize].clone())
                        .collect();

                    let inner_ixs = to_inner_ixs(&meta.inner_instructions.clone().unwrap());

                    let token_transfer_for_ix =
                        get_token_transfers_for_ix_index(&inner_ixs, &account_keys, index as u32);

                    let sign = String::from("some signature"); // TODO: Get the actual signature

                    let ix_data = decode(ix.data.clone()).into_vec().unwrap();

                    let configs = ParseConfigs {
                        txn: sign,
                        ix_accounts,
                        program_id: program_id.clone(),
                        token_transfers: token_transfer_for_ix,
                        ix_data: ix_data.to_vec(),
                    };

                    parsed_txns.push(configs);
                }
            }
        }
        Err(e) => {
            println!("Couldn't get the txn::: {e}")
        }
    }

    parsed_txns
}
