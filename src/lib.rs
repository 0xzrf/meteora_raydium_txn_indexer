use db::DBOps;
use helius_laserstream::{LaserstreamError, subscribe};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub mod db;
pub mod parser;
use parser::Parser;
pub mod api_routes;
pub mod constants;
pub mod error_logging;
pub mod helper;
pub mod price_feed;
pub mod wide_event;
use helper::*;

/// Initialize structured logging (wide events). Call once at startup.
pub fn init_tracing() {
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(true)
        .with_thread_ids(false)
        .json()
        .with_current_span(false)
        .with_span_list(false);
    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .init();
}

pub async fn handler() -> Result<(), LaserstreamError> {
    init_tracing();

    let config = get_laserstream_subscription_config();

    if is_devnet() {
        tracing::info!(env = "devnet", "running in devnet");
    }

    let db_ops = DBOps::connect().await.expect("Unable to connect to db"); // Don't allow the program to run if db not connected

    let (users, copy_wallets) = get_users(&db_ops).await;

    let request = get_sub_req(users, copy_wallets);

    let (stream, handle) = subscribe(config, request);
    let parser = Parser;

    parser.handle_stream(stream, handle, db_ops).await?;
    Ok(())
}
