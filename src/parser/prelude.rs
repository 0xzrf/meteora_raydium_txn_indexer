use super::{
    ParseData, api_parsers::*, meteora::*, raydium::*,
    token_transfer::get_token_transfers_for_ix_index,
};
use crate::{
    api_routes::{
        add_copytrade_wallet::add_copytrade_wallet_logic, add_user_wallet::add_wallet_logic,
    },
    constants::*,
    db::DBOps,
    helper::{
        get_fee_receiver, get_integrated_protocols, get_ws_port, log_txn_sig,
        new_sub_req_with_new_wallets, store_err,
    },
    parser::{
        CopyReturnType, CopyTradeParseData, DbTaskStruct, ParseConfigs, ParseConfigsCopyTrading,
        PoolReturnType, meteora::dlmm_copytrading::MeteoraDlmmCopyTradeProgram,
    },
    price_feed::get_price_for_token,
    wide_event::{
        Outcome, emit_copy_trade_processed, emit_db_operation, emit_fee_receiver_processed,
        emit_subscription_event, emit_txn_processed,
    },
};
use axum::{Extension, Router, routing::post};
use futures_util::{SinkExt, Stream, StreamExt};
use helius_laserstream::{
    LaserstreamError, StreamHandle,
    grpc::{SubscribeRequest, SubscribeUpdate, subscribe_update::UpdateOneof},
};
use std::{pin::Pin, sync::Arc};
use tokio::net::TcpListener;
use tokio::sync::{RwLock, broadcast, mpsc::unbounded_channel};
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::Message;

pub struct Parser;

pub type StreamType = Pin<Box<dyn Stream<Item = Result<SubscribeUpdate, LaserstreamError>> + Send>>;

impl Parser {
    pub async fn handle_stream(
        &self,
        stream: impl Stream<Item = Result<SubscribeUpdate, LaserstreamError>>,
        handler: StreamHandle,
        db_ops: DBOps,
    ) -> Result<(), LaserstreamError> {
        let integrated_protocols = get_integrated_protocols();
        let handler_clone = handler.clone();
        futures_util::pin_mut!(stream);

        let db_arc = Arc::new(RwLock::new(db_ops));

        let db_user_clone = Arc::clone(&db_arc);
        let db_referral_clone = Arc::clone(&db_arc);
        let db_parse_clone = Arc::clone(&db_arc);

        let fee_receiver = get_fee_receiver();

        let (wallet_sender, mut wallet_receiver) = unbounded_channel::<(Vec<String>, u8)>();
        let (copy_trade_sender, _) = broadcast::channel::<CopyTradeParseData>(5); // accept upto 5 broadcasts

        let copy_trade_indexer_clone = copy_trade_sender.clone();

        let (parse_sender, mut parse_receiver) = unbounded_channel::<DbTaskStruct>();
        let (sub_req_sender, mut sub_req_recvr) = unbounded_channel::<SubscribeRequest>();

        let mut cached_parsed_data: Vec<ParseData> = vec![];
        let mut cached_parsed_data_copy: Vec<CopyTradeParseData> = vec![];
        let mut cached_fee_receiver_txns: Vec<String> = vec![];

        tokio::spawn(async move {
            while let Some(sub_req) = sub_req_recvr.recv().await {
                println!("new sub req: {sub_req:#?}");
                match handler_clone.write(sub_req.clone()).await {
                    Ok(_) => {
                        emit_subscription_event("subscription_write", Outcome::Success, None, None);
                    }
                    Err(e) => {
                        emit_subscription_event(
                            "subscription_write",
                            Outcome::Error,
                            None,
                            Some(&format!("{e:#?}")),
                        );
                    }
                };

                // let configs = get_laserstream_subscription_config();

                // let (new_stream, new_handler) = subscribe(configs, sub_req);
            }
        });

        // Manipulate the users or copy wallets
        tokio::spawn(async move {
            tokio::spawn(async move {
                let router = Router::new()
                    .route("/add_wallet", post(add_wallet_logic))
                    .route("/add_copytrade_wallet", post(add_copytrade_wallet_logic))
                    .layer(Extension(wallet_sender));

                let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap(); // .unwrap to not allow the indexer to run without tcp server listening
                let addr = listener.local_addr().unwrap();
                tracing::info!(binding = %addr, service = "add_wallet_api", "HTTP server listening");
                axum::serve(listener, router).await.unwrap(); // unwrap and show the reason if axum is not able to start listening on post 3000
            });

            tokio::spawn(async move {
                while let Some((wallets, wallet_type)) = wallet_receiver.recv().await {
                    let new_sub_req: SubscribeRequest =
                        new_sub_req_with_new_wallets(db_user_clone.read().await, &wallets, false)
                            .await;
                    if wallet_type == 0 {
                        tracing::info!(
                            wallets_count = wallets.len(),
                            wallet_type = "user",
                            "got user wallets"
                        );
                        for wallet in &wallets {
                            if let Err(msg) =
                                db_user_clone.read().await.push_user_pubkey(wallet).await
                            {
                                emit_subscription_event(
                                    "push_user_pubkey",
                                    Outcome::Error,
                                    None,
                                    Some(msg),
                                );
                                continue;
                            }
                        }
                    } else {
                        tracing::info!(
                            wallets_count = wallets.len(),
                            wallet_type = "copy_trade",
                            "got copy_trade wallets"
                        );
                        for wallet in &wallets {
                            if let Err(msg) = db_user_clone
                                .read()
                                .await
                                .push_copytrade_pubkey(wallet)
                                .await
                            {
                                emit_subscription_event(
                                    "push_copytrade_pubkey",
                                    Outcome::Error,
                                    None,
                                    Some(msg),
                                );
                                continue;
                            }
                        }
                    }

                    match sub_req_sender.send(new_sub_req) {
                        Ok(_) => {
                            emit_subscription_event(
                                "send_subscribe_request",
                                Outcome::Success,
                                Some(wallets.len() as u32),
                                None,
                            );
                        }
                        Err(e) => {
                            emit_subscription_event(
                                "send_subscribe_request",
                                Outcome::Error,
                                Some(wallets.len() as u32),
                                Some(&format!("{e:#?}")),
                            );
                        }
                    }
                }
            });
        });

        // Task to handle parsed data and push it to db (one wide event per txn)
        tokio::spawn(async move {
            while let Some(DbTaskStruct {
                parsed_data,
                copy_trade,
                is_user,
                signer,
            }) = parse_receiver.recv().await
            {
                if is_user {
                    if let Ok(parsed_data) = parsed_data.clone().unwrap() {
                        let parsed_data = Self::get_token_info(parsed_data).await;

                        tracing::debug!(
                            txn_sig = %parsed_data.txn_sig,
                            signer = %signer,
                            action = %parsed_data.action,
                            contract_address = %parsed_data.contract_address,
                            "parsed user data"
                        );

                        let position_nft = parsed_data.position_nft.clone();

                        if parsed_data.action.eq("close_position")
                            && let Some(position) = position_nft
                        {
                            match db_parse_clone
                                .read()
                                .await
                                .get_position_activity_row(&position, parsed_data.txn_sig.clone())
                                .await
                            {
                                Ok(data) => match db_parse_clone
                                    .read()
                                    .await
                                    .push_db_activities(&data)
                                    .await
                                {
                                    Ok(_) => {
                                        emit_db_operation(
                                            "push_db_activities",
                                            Outcome::Success,
                                            Some(&parsed_data.txn_sig),
                                            Some(&signer),
                                            None,
                                        );
                                    }
                                    Err(e) => {
                                        emit_db_operation(
                                            "push_db_activities",
                                            Outcome::Error,
                                            Some(&parsed_data.txn_sig),
                                            Some(&signer),
                                            Some(&e.to_string()),
                                        );
                                    }
                                },
                                Err(msg) => {
                                    emit_db_operation(
                                        "get_position_activity_row",
                                        Outcome::Error,
                                        Some(&parsed_data.txn_sig),
                                        Some(&signer),
                                        Some(msg),
                                    );
                                    store_err(msg);
                                }
                            }
                        } else if parsed_data.action.eq("close_position")
                            && parsed_data.position_nft.is_none()
                        {
                            tracing::warn!(
                                txn_sig = %parsed_data.txn_sig,
                                signer = %signer,
                                "close_position without position_nft"
                            );
                        }

                        match db_parse_clone
                            .read()
                            .await
                            .push_db(&parsed_data, &signer)
                            .await
                        {
                            Ok(_) => {
                                emit_txn_processed(
                                    Outcome::Success,
                                    &parsed_data.txn_sig,
                                    &signer,
                                    "user",
                                    &parsed_data.action,
                                    &parsed_data.contract_address,
                                    Some(&parsed_data.pool_address),
                                    None,
                                    None,
                                );
                            }
                            Err(err) => {
                                emit_txn_processed(
                                    Outcome::Error,
                                    &parsed_data.txn_sig,
                                    &signer,
                                    "user",
                                    &parsed_data.action,
                                    &parsed_data.contract_address,
                                    Some(&parsed_data.pool_address),
                                    None,
                                    Some(&err.to_string()),
                                );
                                store_err(&err.to_string());
                            }
                        }
                    } else if let Err(msg) = parsed_data.unwrap() {
                        emit_db_operation(
                            "parse_user",
                            Outcome::Error,
                            None,
                            Some(&signer),
                            Some(msg),
                        );
                        store_err(msg);
                    }
                } else if let Ok(data) = copy_trade.clone().unwrap() {
                    tracing::debug!(
                        txn_sig = %data.txn_sig,
                        signer = %signer,
                        action = %data.action,
                        "copy trade data"
                    );

                    match db_parse_clone
                        .read()
                        .await
                        .push_db_copy(&data, &signer)
                        .await
                    {
                        Ok(_) => {
                            emit_copy_trade_processed(
                                Outcome::Success,
                                &data.txn_sig,
                                &signer,
                                &data.action,
                                &data.contract_address,
                                &data.pool_address,
                                None,
                                None,
                            );
                        }
                        Err(err) => {
                            emit_copy_trade_processed(
                                Outcome::Error,
                                &data.txn_sig,
                                &signer,
                                &data.action,
                                &data.contract_address,
                                &data.pool_address,
                                None,
                                Some(&err.to_string()),
                            );
                        }
                    }
                } else if let Err(msg) = copy_trade.unwrap() {
                    emit_db_operation(
                        "parse_copy_trade",
                        Outcome::Error,
                        None,
                        Some(&signer),
                        Some(msg),
                    );
                    store_err(msg);
                }
            }
        });

        // websocket service to send copytrade info to subscribed channels
        tokio::spawn(async move {
            let ws_port = get_ws_port();
            let ws_addr = format!("0.0.0.0:{ws_port}");
            let listener = TcpListener::bind(&ws_addr).await.unwrap();
            tracing::info!(binding = %ws_addr, service = "copytrade_ws", "WebSocket server listening");

            while let Ok((stream, _)) = listener.accept().await {
                let tx = copy_trade_sender.clone();

                tokio::spawn(async move {
                    let ws = accept_async(stream).await.unwrap();

                    let (mut sink, mut stream) = ws.split();
                    let mut rx = tx.subscribe();

                    loop {
                        tokio::select! {
                            // publish → websocket
                            msg = rx.recv() => {
                                match msg {
                                    Ok(data) => {
                                        if let Ok(json) = serde_json::to_string(&data) && sink.send(Message::Text(json)).await.is_err() {
                                                break;
                                        }
                                    }
                                    Err(_) => break,
                                }
                            }

                            frame = stream.next() => {
                                match frame {
                                    Some(Ok(Message::Ping(p))) => { let _ = sink.send(Message::Pong(p)).await; }
                                    Some(Ok(Message::Close(_))) | None => break,
                                    _ => {} // ignore everything else
                                }
                            }
                        }
                    }
                });
            }
        });

        // Handle stream data from laserstream
        while let Some(result) = stream.next().await {
            match result {
                Ok(update) => {
                    if let Some(UpdateOneof::Transaction(tx_update)) = update.update_oneof
                        && let Some(tx_info) = tx_update.transaction
                    {
                        let sig = bs58::encode(&tx_info.signature).into_string();
                        tracing::debug!(txn_sig = %sig, filters = ?update.filters, "transaction update");
                        log_txn_sig(&sig);

                        if let Some(tx) = tx_info.transaction
                            && let Some(message) = tx.message
                        {
                            println!("filters: {:#?}", &update.filters);
                            for filter in &update.filters {
                                match filter {
                                    user_filter
                                        if user_filter.eq(USER_FILTER)
                                            | user_filter.eq(COPY_TRADE_FILTER) =>
                                    {
                                        if let Some(meta) = &tx_info.meta {
                                            let mut account_keys: Vec<String> = message
                                                .account_keys
                                                .iter()
                                                .map(|k| bs58::encode(k).into_string())
                                                .collect();

                                            if message.versioned
                                                && (!meta.loaded_readonly_addresses.is_empty()
                                                    || !meta.loaded_writable_addresses.is_empty())
                                            {
                                                tracing::debug!(txn_sig = %sig, "versioned program, adding loaded addresses");
                                                for k in meta.loaded_writable_addresses.iter() {
                                                    account_keys
                                                        .push(bs58::encode(k).into_string());
                                                }
                                                for k in meta.loaded_readonly_addresses.iter() {
                                                    account_keys
                                                        .push(bs58::encode(k).into_string());
                                                }
                                            }
                                            for (index, ix) in
                                                message.instructions.iter().enumerate()
                                            {
                                                let program_id =
                                                    &account_keys[ix.program_id_index as usize];
                                                let account_len = account_keys.len();
                                                if integrated_protocols.contains(program_id) {
                                                    let ix_accounts: Vec<String> = ix
                                                    .accounts
                                                    .iter()
                                                    .map(|idx| {
                                                        if *idx as usize >= account_len {
                                                            store_err(&format!("index out of bounds for signature: {sig} and index {idx}"));
                                                            "OUT_OF_BOUND".to_string() 
                                                        } else {
                                                            account_keys[*idx as usize].clone()
                                                        }
                                                    })
                                                    .collect();

                                                    let token_transfer_for_ix =
                                                        get_token_transfers_for_ix_index(
                                                            &meta.inner_instructions,
                                                            &account_keys,
                                                            index as u32,
                                                        );

                                                    if filter.eq(USER_FILTER) {
                                                        let parse_config = ParseConfigs {
                                                            ix_data: ix.data.clone(),
                                                            txn: sig.clone(),
                                                            ix_accounts,
                                                            token_transfers: token_transfer_for_ix,
                                                        };

                                                        let data = Self::get_parsed_data(
                                                            program_id,
                                                            parse_config,
                                                            &integrated_protocols,
                                                        );
                                                        match data {
                                                            Some((parsed_data, owner)) => {
                                                                if let Err(e) = parsed_data {
                                                                    store_err(e);
                                                                    break;
                                                                }

                                                                // THERE IS AN ISSUE THAT TRANSACTIONS ARE BEING SENT MULTIPLE TIMES,
                                                                // TO TACKLE THIS, WE'RE CACHING THE TXNS AND SEEING IF IT ALREADY EXISTS
                                                                let check_parsed_data =
                                                                    parsed_data.clone().unwrap();
                                                                if cached_parsed_data
                                                                    .contains(&check_parsed_data)
                                                                {
                                                                    emit_txn_processed(
                                                                        Outcome::SkippedDuplicate,
                                                                        &sig,
                                                                        &owner,
                                                                        "user",
                                                                        &check_parsed_data.action,
                                                                        &check_parsed_data
                                                                            .contract_address,
                                                                        Some(
                                                                            &check_parsed_data
                                                                                .pool_address,
                                                                        ),
                                                                        None,
                                                                        None,
                                                                    );
                                                                    break; // exit loop and do not send
                                                                }

                                                                if cached_parsed_data.len()
                                                                    == CACHE_LIMIT
                                                                {
                                                                    cached_parsed_data.remove(0);
                                                                }
                                                                cached_parsed_data.push(
                                                                    check_parsed_data.clone(),
                                                                );

                                                                let data = DbTaskStruct {
                                                                    copy_trade: None,
                                                                    parsed_data: Some(
                                                                        parsed_data.clone(),
                                                                    ),
                                                                    is_user: true,
                                                                    signer: owner.clone(),
                                                                };

                                                                match parse_sender.send(data) {
                                                                    Ok(_) => {
                                                                        tracing::debug!(txn_sig = %sig, signer = %owner, "sent to db task");
                                                                    }
                                                                    Err(e) => {
                                                                        emit_txn_processed(
                                                                            Outcome::Error,
                                                                            &sig,
                                                                            &owner,
                                                                            "user",
                                                                            &check_parsed_data
                                                                                .action,
                                                                            &check_parsed_data
                                                                                .contract_address,
                                                                            Some(
                                                                                &check_parsed_data
                                                                                    .pool_address,
                                                                            ),
                                                                            None,
                                                                            Some(&format!(
                                                                                "send failed: {e:#?}"
                                                                            )),
                                                                        );
                                                                    }
                                                                }
                                                            }
                                                            None => {
                                                                tracing::debug!(txn_sig = %sig, "user tx not parsed (unknown ix)");
                                                            }
                                                        }
                                                    } else {
                                                        let parse_config =
                                                            ParseConfigsCopyTrading {
                                                                txn: sig.clone(),
                                                                ix_accounts,
                                                                token_transfers:
                                                                    token_transfer_for_ix,
                                                                ix_data: ix.data.clone(),
                                                            };
                                                        let data = Self::get_copy_parsed_data(
                                                            program_id,
                                                            parse_config,
                                                            &integrated_protocols,
                                                        );
                                                        match data {
                                                            Some((copy_trade, owner)) => {
                                                                if let Err(e) = copy_trade {
                                                                    store_err(e);
                                                                    break;
                                                                }

                                                                // THERE IS AN ISSUE THAT TRANSACTIONS ARE BEING SENT MULTIPLE TIMES,
                                                                // TO TACKLE THIS, WE'RE CACHING THE TXNS AND SEEING IF IT ALREADY EXISTS
                                                                let check_parsed_data =
                                                                    copy_trade.clone().unwrap();
                                                                if cached_parsed_data_copy
                                                                    .contains(&check_parsed_data)
                                                                {
                                                                    emit_copy_trade_processed(
                                                                        Outcome::SkippedDuplicate,
                                                                        &sig,
                                                                        &owner,
                                                                        &check_parsed_data.action,
                                                                        &check_parsed_data
                                                                            .contract_address,
                                                                        &check_parsed_data
                                                                            .pool_address,
                                                                        None,
                                                                        None,
                                                                    );
                                                                    break;
                                                                } else {
                                                                    if cached_parsed_data_copy.len()
                                                                        == CACHE_LIMIT
                                                                    {
                                                                        cached_parsed_data
                                                                            .remove(0);
                                                                    }
                                                                    cached_parsed_data_copy
                                                                        .push(check_parsed_data)
                                                                }

                                                                let mut copy_data =
                                                                    copy_trade.unwrap();
                                                                let token_a = &copy_data.token_a;
                                                                let token_b = &copy_data.token_b;
                                                                if let Some((price_a, decimals_a)) =
                                                                    get_price_for_token(token_a)
                                                                        .await
                                                                {
                                                                    copy_data.token_a_price =
                                                                        Some(price_a);
                                                                    copy_data.decimal_a =
                                                                        Some(decimals_a);
                                                                }
                                                                if let Some((price_b, decimals_b)) =
                                                                    get_price_for_token(token_b)
                                                                        .await
                                                                {
                                                                    copy_data.token_b_price =
                                                                        Some(price_b);
                                                                    copy_data.decimal_b =
                                                                        Some(decimals_b);
                                                                }
                                                                match copy_trade_indexer_clone
                                                                    .send(copy_data.clone())
                                                                {
                                                                    Ok(_) => {
                                                                        tracing::debug!(txn_sig = %sig, "sent to ws");
                                                                    }
                                                                    Err(e) => {
                                                                        emit_copy_trade_processed(
                                                                            Outcome::Error,
                                                                            &sig,
                                                                            &owner,
                                                                            &copy_data.action,
                                                                            &copy_data
                                                                                .contract_address,
                                                                            &copy_data.pool_address,
                                                                            None,
                                                                            Some(&format!(
                                                                                "ws send: {e}"
                                                                            )),
                                                                        );
                                                                        store_err(
                                                                            "Error sending to ws",
                                                                        );
                                                                    }
                                                                };
                                                                let data = DbTaskStruct {
                                                                    copy_trade: Some(Ok(
                                                                        copy_data.clone()
                                                                    )),
                                                                    parsed_data: None,
                                                                    is_user: false,
                                                                    signer: owner.clone(),
                                                                };
                                                                match parse_sender.send(data) {
                                                                    Ok(_) => {
                                                                        tracing::debug!(txn_sig = %sig, "sent copy_trade to db task");
                                                                    }
                                                                    Err(e) => {
                                                                        emit_copy_trade_processed(
                                                                            Outcome::Error,
                                                                            &sig,
                                                                            &owner,
                                                                            &copy_data.action,
                                                                            &copy_data
                                                                                .contract_address,
                                                                            &copy_data.pool_address,
                                                                            None,
                                                                            Some(&format!(
                                                                                "db task send: {e:#?}"
                                                                            )),
                                                                        );
                                                                        match parse_sender.send(e.0)
                                                                        {
                                                                            Ok(_) => {}
                                                                            Err(e) => {
                                                                                store_err(
                                                                                    &format!(
                                                                                        "Unable to send to db task: {e:#?}"
                                                                                    ),
                                                                                );
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                            None => {
                                                                tracing::debug!(txn_sig = %sig, "copy_trade tx not parsed");
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    fee_rcvr if fee_rcvr.eq(FEE_RECEIVER_FILTER) => {
                                        if let Some(meta) = &tx_info.meta {
                                            let mut account_keys: Vec<String> = message
                                                .account_keys
                                                .iter()
                                                .map(|k| bs58::encode(k).into_string())
                                                .collect();

                                            if message.versioned
                                                && (!meta.loaded_readonly_addresses.is_empty()
                                                    || !meta.loaded_writable_addresses.is_empty())
                                            {
                                                tracing::debug!(txn_sig = %sig, "versioned program, adding loaded addresses");
                                                for k in meta.loaded_writable_addresses.iter() {
                                                    account_keys
                                                        .push(bs58::encode(k).into_string());
                                                }
                                                for k in meta.loaded_readonly_addresses.iter() {
                                                    account_keys
                                                        .push(bs58::encode(k).into_string());
                                                }
                                            }
                                            for ix in &message.instructions {
                                                let program_id =
                                                    &account_keys[ix.program_id_index as usize];
                                                let account_len = account_keys.len();

                                                if program_id.eq(SYSTEM_PROGRAM_ADDR) {
                                                    // THERE IS AN ISSUE THAT TRANSACTIONS ARE BEING SENT MULTIPLE TIMES,
                                                    // TO TACKLE THIS, WE'RE CACHING THE TXNS AND SEEING IF IT ALREADY EXISTS
                                                    if cached_fee_receiver_txns.contains(&sig) {
                                                        tracing::debug!(txn_sig = %sig, "skipping duplicate fee_receiver transaction");
                                                        continue;
                                                    }

                                                    let ix_accounts: Vec<String> = ix
                                                                            .accounts
                                                                            .iter()
                                                                            .map(|idx| {
                                                                                if *idx as usize >= account_len {
                                                                                    store_err(&format!("index out of bounds for signaturr: {sig} and index {idx}"));
                                                                                    "OUT_OF_BOUND".to_string()
                                                                                } else {
                                                                                    account_keys[*idx as usize].clone()
                                                                                }
                                                                            })
                                                                            .collect();

                                                    let data = ix.data.clone();
                                                    if data.len() <= 4 {
                                                        tracing::debug!(txn_sig = %sig, "invalid system transfer ix (len)");
                                                        continue;
                                                    }
                                                    let (disc, ix_data) = data.split_at(4);
                                                    if ![2, 0, 0, 0].eq(disc) || ix_data.len() != 8
                                                    {
                                                        tracing::debug!(txn_sig = %sig, "invalid system transfer ix (disc)");
                                                        continue;
                                                    }

                                                    let amount = u64::from_le_bytes(
                                                        ix_data.try_into().unwrap(),
                                                    );

                                                    let (price, _) = match get_price_for_token(
                                                                            "So11111111111111111111111111111111111111112",
                                                                        )
                                                                        .await
                                                                        {
                                                                            Some(v) => v,
                                                                            None => {
                                                                                tracing::warn!(txn_sig = %sig, "could not fetch token price");
                                                                                continue;
                                                                            }
                                                                        };

                                                    let from = &ix_accounts[0];
                                                    let to = &ix_accounts[1];

                                                    if from.ne(&fee_receiver)
                                                        && to.ne(&fee_receiver)
                                                    {
                                                        tracing::debug!(txn_sig = %sig, from = %from, to = %to, "fee receiver not in txn");
                                                        continue;
                                                    }

                                                    if let Err(e) = db_referral_clone
                                                        .read()
                                                        .await
                                                        .push_referral(
                                                            from, to, amount, &sig, price,
                                                        )
                                                        .await
                                                    {
                                                        emit_fee_receiver_processed(
                                                            Outcome::Error,
                                                            &sig,
                                                            from,
                                                            to,
                                                            amount,
                                                            Some(&e.to_string()),
                                                        );
                                                        continue;
                                                    }
                                                    emit_fee_receiver_processed(
                                                        Outcome::Success,
                                                        &sig,
                                                        from,
                                                        to,
                                                        amount,
                                                        None,
                                                    );

                                                    // Add to cache after successful processing
                                                    if cached_fee_receiver_txns.len() == CACHE_LIMIT
                                                    {
                                                        cached_fee_receiver_txns.remove(0);
                                                    }
                                                    cached_fee_receiver_txns.push(sig.clone());
                                                }
                                            }
                                        }
                                    }
                                    _ => {} // unreachable branch, since there's only 2 filters
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    tracing::error!(error = ?e, "stream error");
                }
            }
        }

        Ok(())
    }

    pub async fn get_token_info(mut parsed_data: ParseData) -> ParseData {
        if parsed_data
            .contract_address
            .eq(&METEORA_DAMM_V2_PUBKEY.to_string())
            && (parsed_data.action.eq("create_position") || parsed_data.action.eq("close_position"))
            && let Some((token_a, token_b)) =
                get_tokens_from_pool_damm(&parsed_data.pool_address).await
        {
            parsed_data.token_a = token_a;
            parsed_data.token_b = token_b;
        }
        if parsed_data
            .contract_address
            .eq(&METEORA_DLMM_PUBKEY.to_string())
            && parsed_data.action.eq("create_position")
            && let Some((token_a, token_b)) =
                get_tokens_from_pool_dlmm(&parsed_data.pool_address).await
        {
            parsed_data.token_a = token_a;
            parsed_data.token_b = token_b;
        }

        if parsed_data
            .contract_address
            .eq(&RAYDIUM_AMM_PUBKEY.to_string())
            && let Some((token_a, token_b)) =
                get_tokens_from_raydium_amm(&parsed_data.pool_address).await
        {
            parsed_data.token_a = token_a;
            parsed_data.token_b = token_b;
        }

        let token_a = &parsed_data.token_a;
        let token_b = &parsed_data.token_b;
        if let Some((price_a, decimals_a)) = get_price_for_token(token_a).await {
            parsed_data.token_a_price = Some(price_a);
            parsed_data.decimal_a = Some(decimals_a);
        }
        if let Some((price_b, decimals_b)) = get_price_for_token(token_b).await {
            parsed_data.token_b_price = Some(price_b);
            parsed_data.decimal_b = Some(decimals_b);
        }

        parsed_data
    }

    pub async fn get_token_info_copy(mut parsed_data: CopyTradeParseData) -> CopyTradeParseData {
        if parsed_data
            .contract_address
            .eq(&METEORA_DAMM_V2_PUBKEY.to_string())
            && (parsed_data.action.eq("create_position") || parsed_data.action.eq("close_position"))
            && let Some((token_a, token_b)) =
                get_tokens_from_pool_damm(&parsed_data.pool_address).await
        {
            parsed_data.token_a = token_a;
            parsed_data.token_b = token_b;
        }
        if parsed_data
            .contract_address
            .eq(&METEORA_DLMM_PUBKEY.to_string())
            && parsed_data.action.eq("create_position")
            && let Some((token_a, token_b)) =
                get_tokens_from_pool_dlmm(&parsed_data.pool_address).await
        {
            parsed_data.token_a = token_a;
            parsed_data.token_b = token_b;
        }

        if parsed_data
            .contract_address
            .eq(&RAYDIUM_AMM_PUBKEY.to_string())
            && let Some((token_a, token_b)) =
                get_tokens_from_raydium_amm(&parsed_data.pool_address).await
        {
            parsed_data.token_a = token_a;
            parsed_data.token_b = token_b;
        }

        let token_a = &parsed_data.token_a;
        let token_b = &parsed_data.token_b;
        if let Some((price_a, decimals_a)) = get_price_for_token(token_a).await {
            parsed_data.token_a_price = Some(price_a);
            parsed_data.decimal_a = Some(decimals_a);
        }
        if let Some((price_b, decimals_b)) = get_price_for_token(token_b).await {
            parsed_data.token_b_price = Some(price_b);
            parsed_data.decimal_b = Some(decimals_b);
        }

        parsed_data
    }

    pub fn get_parsed_data(
        program_id: &str,
        parse_config: ParseConfigs,
        integrated_protocols: &[String],
    ) -> Option<(PoolReturnType, String)> {
        match program_id {
            id if id.eq(&integrated_protocols[0]) => RaydiumCpmmProgram::new(parse_config)
                .set_action_type()
                .get_parsed_data(),
            id if id.eq(&integrated_protocols[1]) => RaydiumClmmProgram::new(parse_config)
                .set_action_type()
                .get_parsed_data(),
            id if id.eq(&integrated_protocols[2]) => RaydiumAmmProgram::new(parse_config)
                .set_action_type()
                .get_parsed_data(),
            id if id.eq(METEORA_DAMM_V2_PUBKEY) => MeteoraDammV2Program::new(parse_config)
                .set_action_type()
                .get_parsed_data(),
            id if id.eq(METEORA_DLMM_PUBKEY) => MeteoraDlmmProgram::new(parse_config)
                .set_action_type()
                .get_parsed_data(),
            _ => None,
        }
    }

    pub fn get_copy_parsed_data(
        program_id: &str,
        parse_config: ParseConfigsCopyTrading,
        integrated_protocols: &[String],
    ) -> Option<(CopyReturnType, String)> {
        let ParseConfigsCopyTrading {
            txn: sig,
            ix_accounts,
            token_transfers: token_transfer_for_ix,
            ix_data,
        } = parse_config;

        match program_id {
            id if id.eq(&integrated_protocols[0]) => RaydiumCpmmCopyTradingProgram::new(
                ix_accounts,
                sig.to_string(),
                ix_data,
                token_transfer_for_ix,
            )
            .set_action_type()
            .get_parsed_data(),
            id if id.eq(&integrated_protocols[1]) => RaydiumClmmCopyTradingProgram::new(
                ix_data,
                ix_accounts,
                sig.to_string(),
                token_transfer_for_ix,
            )
            .set_action_type()
            .get_parsed_data(),
            id if id.eq(&integrated_protocols[2]) => RaydiumAmmCopyProgram::new(
                ix_data,
                ix_accounts,
                sig.to_string(),
                token_transfer_for_ix,
            )
            .set_action_type()
            .get_parsed_data(),
            id if id.eq(&METEORA_DAMM_V2_PUBKEY.to_string()) => {
                MeteoraDammV2CopyTradingProgram::new(
                    ix_accounts,
                    ix_data,
                    sig.to_string(),
                    token_transfer_for_ix,
                )
                .set_action_type()
                .get_parsed_data()
            }
            id if id.eq(&METEORA_DLMM_PUBKEY.to_string()) => MeteoraDlmmCopyTradeProgram::new(
                ix_data.to_vec(),
                ix_accounts,
                sig.to_string(),
                token_transfer_for_ix,
            )
            .set_action_type()
            .get_parsed_data(),
            _ => None,
        }
    }
}
