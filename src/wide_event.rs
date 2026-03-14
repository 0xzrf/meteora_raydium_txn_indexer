//! Wide events / canonical log lines per [loggingsucks.com](https://loggingsucks.com/).
//!
//! Emit **one context-rich event per logical operation** (e.g. per transaction processed)
//! with high cardinality (txn_sig, signer, request_id) and high dimensionality (many fields)
//! so logs are queryable and debuggable without grep.

use std::fmt;
use std::time::Instant;
use tracing::{Instrument, info_span};

/// Outcome of a logical operation for tail sampling and querying.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Success,
    Error,
    Slow,
    SkippedDuplicate,
}

impl fmt::Display for Outcome {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Outcome::Success => write!(f, "success"),
            Outcome::Error => write!(f, "error"),
            Outcome::Slow => write!(f, "slow"),
            Outcome::SkippedDuplicate => write!(f, "skipped_duplicate"),
        }
    }
}

/// Tail sampling: always keep errors and slow/important events; sample the rest.
/// Set via env `WIDE_EVENT_SAMPLE_RATE` (0.0–1.0, default 1.0 = keep all).
#[inline]
pub fn should_sample(outcome: Outcome, duration_ms: Option<u64>, slow_threshold_ms: u64) -> bool {
    // Always keep errors
    if outcome == Outcome::Error {
        return true;
    }
    // Always keep slow operations (above threshold)
    if let Some(ms) = duration_ms
        && ms >= slow_threshold_ms
    {
        return true;
    }
    // For success/skipped: sample based on rate (1.0 = keep all)
    let rate = std::env::var("WIDE_EVENT_SAMPLE_RATE")
        .ok()
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(1.0)
        .clamp(0.0, 1.0);
    if rate >= 1.0 {
        return true;
    }
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    Instant::now().elapsed().as_nanos().hash(&mut hasher);
    (hasher.finish() % 10000) as f64 / 10000.0 < rate
}

/// Emit a wide event for indexer transaction processing (user action path).
/// Call once per transaction when processing is done; all context in one event.
#[allow(clippy::too_many_arguments)]
pub fn emit_txn_processed(
    outcome: Outcome,
    txn_sig: &str,
    signer: &str,
    filter_type: &str,
    action: &str,
    contract_address: &str,
    pool_address: Option<&str>,
    duration_ms: Option<u64>,
    error_message: Option<&str>,
) {
    let slow_threshold_ms = 2000;
    if !should_sample(outcome, duration_ms, slow_threshold_ms) {
        return;
    }
    let outcome_str = match outcome {
        Outcome::Success => "success",
        Outcome::Error => "error",
        Outcome::Slow => "slow",
        Outcome::SkippedDuplicate => "skipped_duplicate",
    };
    let span = info_span!(
        "indexer.txn_processed",
        outcome = outcome_str,
        txn_sig = %txn_sig,
        signer = %signer,
        filter_type = %filter_type,
        action = %action,
        contract_address = %contract_address,
        pool_address = ?pool_address,
        duration_ms = ?duration_ms,
        error = ?error_message,
    );
    let msg = match outcome {
        Outcome::Error => "transaction processing failed",
        Outcome::SkippedDuplicate => "skipped duplicate transaction",
        Outcome::Slow => "transaction processed (slow)",
        Outcome::Success => "transaction processed",
    };
    tracing::info!(parent: span, "{}", msg);
}

/// Emit a wide event for copy-trade path (sent to WS + DB).
#[allow(clippy::too_many_arguments)]
pub fn emit_copy_trade_processed(
    outcome: Outcome,
    txn_sig: &str,
    signer: &str,
    action: &str,
    contract_address: &str,
    pool_address: &str,
    duration_ms: Option<u64>,
    error_message: Option<&str>,
) {
    let slow_threshold_ms = 2000;
    if !should_sample(outcome, duration_ms, slow_threshold_ms) {
        return;
    }
    let outcome_str = match outcome {
        Outcome::Success => "success",
        Outcome::Error => "error",
        Outcome::Slow => "slow",
        Outcome::SkippedDuplicate => "skipped_duplicate",
    };
    let span = info_span!(
        "indexer.copy_trade_processed",
        outcome = outcome_str,
        txn_sig = %txn_sig,
        signer = %signer,
        action = %action,
        contract_address = %contract_address,
        pool_address = %pool_address,
        duration_ms = ?duration_ms,
        error = ?error_message,
    );
    let msg = match outcome {
        Outcome::Error => "copy trade processing failed",
        Outcome::SkippedDuplicate => "skipped duplicate copy trade",
        Outcome::Slow => "copy trade processed (slow)",
        Outcome::Success => "copy trade processed",
    };
    tracing::info!(parent: span, "{}", msg);
}

/// Emit a wide event for subscription/wallet/API lifecycle (one event per operation).
pub fn emit_subscription_event(
    operation: &str,
    outcome: Outcome,
    wallets_count: Option<u32>,
    error_message: Option<&str>,
) {
    if outcome == Outcome::Error && error_message.is_some() {
        tracing::error!(
            operation = %operation,
            outcome = "error",
            wallets_count = ?wallets_count,
            error = ?error_message,
            "subscription event"
        );
    } else {
        tracing::info!(
            operation = %operation,
            outcome = %outcome,
            wallets_count = ?wallets_count,
            "subscription event"
        );
    }
}

/// Emit a wide event for DB operations (push user action, push copy trade, etc.).
pub fn emit_db_operation(
    operation: &str,
    outcome: Outcome,
    txn_sig: Option<&str>,
    signer: Option<&str>,
    error_message: Option<&str>,
) {
    if outcome == Outcome::Error {
        tracing::error!(
            operation = %operation,
            outcome = "error",
            txn_sig = ?txn_sig,
            signer = ?signer,
            error = ?error_message,
            "db operation"
        );
    } else {
        tracing::info!(
            operation = %operation,
            outcome = %outcome,
            txn_sig = ?txn_sig,
            signer = ?signer,
            "db operation"
        );
    }
}

/// Emit a wide event for fee/referral processing.
pub fn emit_fee_receiver_processed(
    outcome: Outcome,
    txn_sig: &str,
    from: &str,
    to: &str,
    amount: u64,
    error_message: Option<&str>,
) {
    if outcome == Outcome::Error {
        tracing::error!(
            outcome = "error",
            txn_sig = %txn_sig,
            from = %from,
            to = %to,
            amount = %amount,
            error = ?error_message,
            "fee receiver referral"
        );
    } else if should_sample(outcome, None, 0) {
        tracing::info!(
            outcome = %outcome,
            txn_sig = %txn_sig,
            from = %from,
            to = %to,
            amount = %amount,
            "fee receiver referral"
        );
    }
}

/// Run a future in a span so the whole operation is attributed (for async wide events).
pub fn with_span<F, T>(name: &'static str, f: F) -> impl std::future::Future<Output = T>
where
    F: std::future::Future<Output = T>,
{
    let span = tracing::info_span!("indexer.op", op = name);
    f.instrument(span)
}
