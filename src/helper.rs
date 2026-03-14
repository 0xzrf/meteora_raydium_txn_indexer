use crate::constants::*;
use crate::db::DBOps;
use dotenv::dotenv;
use helius_laserstream::LaserstreamConfig;
use helius_laserstream::grpc::SubscribeRequest;
use helius_laserstream::grpc::SubscribeRequestFilterTransactions;
use std::collections::HashMap;
use std::env;
use tokio::sync::RwLockReadGuard;

pub fn get_ws_port() -> u64 {
    dotenv().ok();
    env::var("WS_PORT")
        .expect("Expected WS_PORT env variable to be set")
        .parse::<u64>()
        .expect("Expected WS_PORT to be of type u64")
}

pub fn is_devnet() -> bool {
    dotenv().ok();
    env::var("IS_DEVNET")
        .expect("Expected IS_DEVNET env variable to be set")
        .parse::<bool>()
        .expect("Expected IS_DEVNET env variable to be of type bool")
}

pub fn is_debugging() -> bool {
    dotenv().ok();
    env::var("DEBUGGING")
        .expect("Expected DEBUGGING env variable to be set")
        .parse::<bool>()
        .expect("Expected DEBUGGING env variable to be of type bool")
}

pub fn get_fee_receiver() -> String {
    dotenv().ok();
    env::var("FEE_RECEIVER").expect("Expected FEE_RECEIVER env variable to be set")
}

pub fn use_laser_stream_config() -> bool {
    dotenv().ok();
    env::var("USE_LASERSTREAM")
        .expect("Expected USE_LASERSTREAM env variable to be set")
        .parse::<bool>()
        .expect("Expected USE_LASERSTREAM env variable to be of type bool")
}

pub fn get_laserstream_subscription_config() -> LaserstreamConfig {
    let (api_key, url) = if use_laser_stream_config() {
        get_laser_stream_config()
    } else {
        get_alchemy_stream_config()
    };
    LaserstreamConfig::new(url, api_key)
        .with_replay(true)
        .with_max_reconnect_attempts(10000)
}

/// NOTE: raydium cpmm, clmm and amm program id have to be in the same ordering as shown, as it will break the code
pub fn get_integrated_protocols() -> Vec<String> {
    let fee_receiver = get_fee_receiver();
    if is_devnet() {
        vec![
            RAYDIUM_CPMM_DEVNET_PUBKEY.to_string(),
            RAYDIUM_CLMM_DEVNET_PUBKEY.to_string(),
            RAYDIUM_AMM_DEVNET_PUBKEY.to_string(),
            METEORA_DAMM_V2_PUBKEY.to_string(),
            METEORA_DLMM_PUBKEY.to_string(),
            fee_receiver,
        ]
    } else {
        vec![
            RAYDIUM_CPMM_PUBKEY.to_string(),
            RAYDIUM_CLMM_PUBKEY.to_string(),
            RAYDIUM_AMM_PUBKEY.to_string(),
            METEORA_DAMM_V2_PUBKEY.to_string(),
            METEORA_DLMM_PUBKEY.to_string(),
            fee_receiver,
        ]
    }
}

pub async fn get_users(db_ctx: &DBOps) -> (Vec<String>, Vec<String>) {
    let mut users = vec![];
    let mut copy_wallets = vec![];
    if is_devnet() {
        // hardcoded values in devnet env. for testing
        users = vec![
            // "zrfSbBaBYsgowcVeBfRQbg3yavUL4XnXfP2Qj8xxXne".to_string(),
            // "4DtJbj8Q1ShgTVFwhBSi5gjLDewV7VE5PCtJSzXc46p2".to_string(),
            "2vHLbFC39XWviJ1wLAs5ZY4yRSBGmyTkXRgVryRXr8S3".to_string(),
        ];
        copy_wallets = vec!["2vHLbFC39XWviJ1wLAs5ZY4yRSBGmyTkXRgVryRXr8S3".to_string()];
    } else {
        match db_ctx.get_users().await {
            Ok((user_vec, copy_wallet_vec)) => {
                tracing::info!(
                    user_count = user_vec.len(),
                    copy_wallet_count = copy_wallet_vec.len(),
                    "fetched users"
                );
                users = user_vec;
                copy_wallets = copy_wallet_vec;
            }
            Err(e) => {
                store_err(&format!("Error getting users from db: {e}"));
            }
        }
    }

    (users, copy_wallets)
}

pub fn get_laser_stream_config() -> (String, String) {
    dotenv().ok();
    if is_devnet() {
        (
            env::var("API_KEY").expect("API_KEY must be set in .env"),
            env::var("LASERSTREAM_DEVNET_URL").expect("LASERSTREAM_DEVNET_URL must be set in .env"),
        )
    } else {
        (
            env::var("API_KEY").expect("API_KEY must be set in .env"),
            env::var("LASERSTREAM_URL").expect("LASERSTREAM_URL must be set in .env"),
        )
    }
}

pub fn get_alchemy_stream_config() -> (String, String) {
    dotenv().ok();
    if is_devnet() {
        (
            env::var("ALCHEMY_API_KEY").expect("API_KEY must be set in .env"),
            env::var("ALCHEMY_DEVNET_URL").expect("LASERSTREAM_DEVNET_URL must be set in .env"),
        )
    } else {
        (
            env::var("ALCHEMY_API_KEY").expect("API_KEY must be set in .env"),
            env::var("ALCHEMY_MAINNET_URL").expect("LASERSTREAM_URL must be set in .env"),
        )
    }
}

pub fn get_dlmm_api_url() -> String {
    if is_devnet() {
        DLMM_DEVNET_API_URL.to_string()
    } else {
        DLMM_API_URL.to_string()
    }
}

pub fn get_damm_api_url() -> String {
    if is_devnet() {
        DAMM_DEVNET_API_URL.to_string()
    } else {
        DAMM_API_URL.to_string()
    }
}

pub fn get_raydium_amm_api() -> String {
    if is_devnet() {
        RAYDIUM_AMM_DEVNET_API_URL.to_string()
    } else {
        RAYDIUM_AMM_API_URL.to_string()
    }
}

use std::fs;

pub fn store_err(msg: &str) {
    tracing::error!(error = %msg, "critical error");
    let now = chrono::Local::now().to_string();

    let err_msg = format!("Error happened at: {now:#?}\n\t{msg}");

    if let Err(e) = fs::write("errors.txt", err_msg) {
        tracing::error!(error = %e, "unable to write to errors.txt");
    }
}

pub async fn new_sub_req_with_new_wallets<'info>(
    db_ctx: RwLockReadGuard<'info, DBOps>,
    wallets: &[String],
    add_users: bool,
) -> SubscribeRequest {
    let (mut users, mut copy_wallets) = get_users(&db_ctx).await;

    if add_users {
        users.extend_from_slice(wallets);
    } else {
        copy_wallets.extend_from_slice(wallets);
    }

    tracing::info!(
        user_count = users.len(),
        copy_wallet_count = copy_wallets.len(),
        "built new sub request"
    );

    get_sub_req(users, copy_wallets)
}

pub fn get_sub_req(users: Vec<String>, copy_wallets: Vec<String>) -> SubscribeRequest {
    let fee_receiver = get_fee_receiver();
    SubscribeRequest {
        transactions: HashMap::from([
            (
                USER_FILTER.to_string(),
                SubscribeRequestFilterTransactions {
                    account_required: users,
                    vote: Some(false),
                    failed: Some(false),
                    ..Default::default()
                },
            ),
            (
                COPY_TRADE_FILTER.to_string(),
                SubscribeRequestFilterTransactions {
                    account_required: copy_wallets,
                    vote: Some(false),
                    failed: Some(false),
                    ..Default::default()
                },
            ),
            (
                FEE_RECEIVER_FILTER.to_string(),
                SubscribeRequestFilterTransactions {
                    account_required: vec![fee_receiver],
                    vote: Some(false),
                    failed: Some(false),
                    ..Default::default()
                },
            ),
        ]),
        ..Default::default()
    }
}

pub fn log_txn_sig(sig: &str) {
    let url = if is_devnet() {
        format!("https://solscan.io/tx/{sig}?cluster=devnet")
    } else {
        format!("https://solscan.io/tx/{sig}")
    };
    tracing::debug!(txn_sig = %sig, url = %url, "solscan link");
}
