use crate::{
    constants::to_program_type,
    parser::{CopyTradeParseData, ParseData},
};

use bigdecimal::BigDecimal;
use dotenv::dotenv;
use sqlx::{Error, FromRow, PgPool, Pool, Postgres};
use std::{
    env,
    ops::{Add, Div, Mul, Sub},
};

#[derive(Clone)]
pub struct DBOps {
    pub pool: Pool<Postgres>,
}

#[derive(FromRow, Debug)]
pub struct User {
    pub user_pubkey: String,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(FromRow, Debug)]
pub struct UserAction {
    pub id: i64,
    pub user_pubkey: String,
    pub contract_address: String,
    pub amount_b: BigDecimal,
    pub amount_a: BigDecimal,
    pub position_nft: Option<String>,
    pub token_a: String,
    pub token_b: String,
    pub pool_address: String,
    pub token_a_price: Option<f64>,
    pub token_b_price: Option<f64>,
    pub action: String,
    pub txn_sig: String,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub decimal_a: Option<i16>,
    pub decimal_b: Option<i16>,
}
#[derive(FromRow, Debug)]
pub struct Referrals {
    pub id: i64,
    pub from_pubkey: String,
    pub to_pubkey: String,
    pub amount: BigDecimal,
    pub txn_sig: String,
    pub sol_price: f64,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(FromRow, Debug)]
pub struct ActivityRow {
    pub id: i64,
    pub user_pubkey: String,
    pub pool_type: String,
    pub total_deposit: f64,
    pub total_withdraw: f64,
    pub claimed_fee: f64,
    pub position_nft: String,
    pub token_a: String,
    pub token_b: String,
    pub pool_address: String,
    pub txn: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub close_time: chrono::DateTime<chrono::Utc>,
    pub decimal_a: Option<i16>,
    pub decimal_b: Option<i16>,
}

#[derive(Debug)]
pub struct Activity {
    pub txn: String,
    pub total_deposit: f64,
    pub total_withdraw: f64,
    pub pnl: f64,
    pub create_time: chrono::DateTime<chrono::Utc>,
    pub pool_address: String,
    pub token_a: String,
    pub token_b: String,
    pub decimal_a: i16,
    pub decimal_b: i16,
    pub pool_type: String,
    pub user_pubkey: String,
    pub position_nft: String,
    pub claimed_fee: f64,
}

#[derive(FromRow, Debug)]
pub struct CopyTrading {
    pub id: i64,
    pub user_pubkey: String,
    pub contract_address: String,
    pub amount_b: BigDecimal,
    pub amount_a: BigDecimal,
    pub position_nft: Option<String>,
    pub token_a: String,
    pub token_b: String,
    pub pool_address: String,
    pub action: String,
    pub txn_sig: String,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub min_bin_id: i64,
    pub max_bin_id: i64,
    pub strategy: Option<i16>,
    pub token_a_price: Option<f64>,
    pub token_b_price: Option<f64>,
    pub decimal_a: Option<i16>,
    pub decimal_b: Option<i16>,
}

impl DBOps {
    pub async fn connect() -> Result<Self, Error> {
        dotenv().ok();

        let db = env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env");

        let pool = PgPool::connect(&db).await?;
        tracing::info!(service = "db", "connected to database");

        Ok(DBOps { pool })
    }

    pub async fn push_referral(
        &self,
        from_pubkey: &str,
        to_pubkey: &str,
        amount: u64,
        sig: &str,
        price: f64,
    ) -> Result<(), Error> {
        let amount_sol = BigDecimal::from(amount);

        match sqlx::query_scalar!(
            r#"
            INSERT INTO fee_history (
                from_pubkey,
                to_pubkey,
                txn_sig,
                amount,
                sol_price
            )
            VALUES ($1,$2,$3,$4,$5)
            RETURNING id
            "#,
            from_pubkey,
            to_pubkey,
            sig,
            amount_sol,
            price
        )
        .fetch_one(&self.pool)
        .await
        {
            Ok(_) => {
                tracing::debug!(operation = "push_referral", txn_sig = %sig, "row inserted");
                Ok(())
            }
            Err(e) => {
                tracing::error!(operation = "push_referral", error = ?e, "insert failed");
                Err(e)
            }
        }
    }
    pub async fn push_db(&self, data: &ParseData, user_pubkey: &str) -> Result<(), Error> {
        let decimal_a = if data.decimal_a.is_some() {
            Some(data.decimal_a.unwrap() as i16)
        } else {
            None
        };
        let decimal_b = if data.decimal_b.is_some() {
            Some(data.decimal_b.unwrap() as i16)
        } else {
            None
        };

        let amount_a = BigDecimal::from(data.amount_a);
        let amount_b = BigDecimal::from(data.amount_b);

        match sqlx::query_scalar!(
            r#"
            INSERT INTO user_actions (
                user_pubkey,
                contract_address,
                amount_b,
                amount_a,
                position_nft,
                token_a,
                token_b,
                pool_address,
                token_a_price,
                token_b_price,
                action,
                txn_sig,
                decimal_a,
                decimal_b
            )
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14)
            RETURNING id
            "#,
            user_pubkey,
            data.contract_address,
            amount_b,
            amount_a,
            data.position_nft,
            data.token_a,
            data.token_b,
            data.pool_address,
            data.token_a_price.map(|x| x as f64),
            data.token_b_price.map(|x| x as f64),
            data.action,
            data.txn_sig,
            decimal_a,
            decimal_b
        )
        .fetch_one(&self.pool)
        .await
        {
            Ok(_) => {
                tracing::debug!(operation = "push_user_actions", txn_sig = %data.txn_sig, "row inserted");
                Ok(())
            }
            Err(e) => {
                tracing::error!(operation = "push_user_actions", txn_sig = %data.txn_sig, error = ?e, "insert failed");
                Err(e)
            }
        }
    }

    pub async fn get_users(&self) -> Result<(Vec<String>, Vec<String>), &'static str> {
        #[derive(FromRow, Debug)]
        pub struct UserPubkey {
            pub user_pubkey: String,
        }
        let users = sqlx::query_as!(
            UserPubkey,
            r#"
            SELECT
                user_pubkey
            FROM users
            "#,
        )
        .fetch_all(&self.pool)
        .await;

        let copytrade_users = sqlx::query_as!(
            UserPubkey,
            r#"
            SELECT
                user_pubkey
            FROM copy_trade_users
            "#,
        )
        .fetch_all(&self.pool)
        .await;

        if users.is_err() || copytrade_users.is_err() {
            return Err("Unable to get users");
        }

        let users: Vec<String> = users
            .unwrap()
            .iter()
            .map(|item| item.user_pubkey.clone())
            .collect();

        let copytrade_users: Vec<String> = copytrade_users
            .unwrap()
            .iter()
            .map(|item| item.user_pubkey.clone())
            .collect();

        Ok((users, copytrade_users))
    }

    pub async fn push_db_activities(&self, data: &Activity) -> Result<(), Error> {
        let _ = sqlx::query_scalar!(
            r#"
            INSERT INTO activities (
                user_pubkey,
                pool_type,
                total_withdraw,
                total_deposit,
                position_nft,
                token_a,
                token_b,
                pool_address,
                txn_sig,
                decimal_a,
                decimal_b,
                claimed_fee
            )
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11, $12)
            RETURNING id
            "#,
            data.user_pubkey,
            data.pool_type,
            data.total_withdraw,
            data.total_deposit,
            data.position_nft,
            data.token_a,
            data.token_b,
            data.pool_address,
            data.txn,
            data.decimal_a,
            data.decimal_b,
            data.claimed_fee
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn push_db_copy(
        &self,
        data: &CopyTradeParseData,
        user_pubkey: &str,
    ) -> Result<(), Error> {
        let strategy = if data.strategy.is_some() {
            Some(data.strategy.unwrap() as i16)
        } else {
            None
        };

        let decimal_a = if data.decimal_a.is_some() {
            Some(data.decimal_a.unwrap() as i16)
        } else {
            None
        };
        let decimal_b = if data.decimal_b.is_some() {
            Some(data.decimal_b.unwrap() as i16)
        } else {
            None
        };

        let amount_a = BigDecimal::from(data.amount_a);
        let amount_b = BigDecimal::from(data.amount_b);

        let _ = sqlx::query_scalar!(
            r#"
            INSERT INTO copy_trading (
                user_pubkey,
                contract_address,
                amount_b,
                amount_a,
                position_nft,
                token_a,
                token_b,
                pool_address,
                action,
                txn_sig,
                min_bin_id,
                max_bin_id,
                strategy,
                token_b_price,
                token_a_price,
                decimal_b,
                decimal_a
            )
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17)
            RETURNING id
            "#,
            user_pubkey,
            data.contract_address,
            amount_b,
            amount_a,
            data.position_nft,
            data.token_a,
            data.token_b,
            data.pool_address,
            data.action,
            data.txn_sig,
            data.min_bin_id as i64,
            data.max_bin_id as i64,
            strategy,
            data.token_a_price.map(|x| x as f64),
            data.token_b_price.map(|x| x as f64),
            decimal_a,
            decimal_b
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_position_activity_row(
        &self,
        position: &str,
        txn_hash: String,
    ) -> Result<Activity, &'static str> {
        match sqlx::query_as!(
            UserAction,
            r#"
                SELECT *
                FROM user_actions
                WHERE action IN ('add_liquidity', 'remove_liquidity', 'claim_fee')
                AND position_nft = $1
            "#,
            position
        )
        .fetch_all(&self.pool)
        .await
        {
            Ok(rows) => {
                if rows.is_empty() {
                    tracing::warn!(position_nft = %position, txn_hash = %txn_hash, "no activity entry found");
                    return Err("Couldn't find any entry");
                }

                let mut total_deposit: f64 = 0.0;
                let mut total_withdraw: f64 = 0.0;
                let mut total_claim: f64 = 0.0;

                let decimal_a = rows[0].decimal_a.unwrap_or(6);
                let decimal_b = rows[0].decimal_b.unwrap_or(6);
                let contract_addr = rows[0].contract_address.clone();
                let token_a = rows[0].token_a.clone();
                let token_b = rows[0].token_b.clone();
                let pubkey = rows[0].user_pubkey.to_string();

                let pool_type = to_program_type(&contract_addr).unwrap();
                let pool_address = rows[0].pool_address.clone();

                let create_position_time = rows[0].created_at.unwrap(); // since the first entry will be for create_position
                let txn = txn_hash;

                for item in rows {
                    let amount_a: f64 = item.amount_a.to_string().parse().unwrap();
                    let amount_b: f64 = item.amount_b.to_string().parse().unwrap();

                    let amount_a_worth = amount_a
                        .mul(item.token_a_price.unwrap_or(0.0))
                        .div(10u64.pow(item.decimal_a.unwrap_or(6).try_into().unwrap()) as f64); // May need to resolve with better error handling

                    let amount_b_worth = amount_b
                        .mul(item.token_b_price.unwrap_or(0.0))
                        .div(10u64.pow(item.decimal_b.unwrap_or(6).try_into().unwrap()) as f64); // May need to resolve with better error handling

                    tracing::debug!(action = %item.action, txn_sig = %item.txn_sig, "activity row");

                    let total_amount_worth = amount_a_worth.add(amount_b_worth);
                    if item.action.eq("add_liquidity") {
                        total_deposit = total_deposit.add(total_amount_worth);
                    } else if item.action.eq("remove_liquidity") || item.action.eq("claim_fee") {
                        total_withdraw = total_withdraw.add(total_amount_worth);
                        if item.action.eq("claim_fee") {
                            total_claim = total_claim.add(total_amount_worth);
                        }
                    }
                }

                let pnl = total_withdraw.sub(total_deposit);

                Ok(Activity {
                    create_time: create_position_time,
                    decimal_a,
                    decimal_b,
                    pnl,
                    txn,
                    total_deposit,
                    total_withdraw,
                    token_a,
                    token_b,
                    pool_address,
                    pool_type: pool_type.to_string(),
                    claimed_fee: total_claim,
                    user_pubkey: pubkey,
                    position_nft: position.to_string(),
                })
            }
            Err(e) => {
                tracing::error!(error = ?e, position_nft = %position, "get_position_activity query failed");
                Err("invalid query")
            }
        }
    }

    pub async fn push_user_pubkey(&self, user_pubkey: &str) -> Result<(), &str> {
        let result = sqlx::query_scalar!(
            r#"
            INSERT INTO users (user_pubkey)
            VALUES ($1)
            "#,
            user_pubkey
        )
        .execute(&self.pool)
        .await;

        match result {
            Ok(_) => {
                tracing::debug!(operation = "push_user_pubkey", user_pubkey = %user_pubkey, "user added");
                Ok(())
            }
            Err(err) => {
                tracing::error!(operation = "push_user_pubkey", user_pubkey = %user_pubkey, error = ?err, "insert failed");
                Err("Unable to create user")
            }
        }
    }

    pub async fn push_copytrade_pubkey(&self, user_pubkey: &str) -> Result<(), &'static str> {
        let result = sqlx::query_scalar!(
            r#"
            INSERT INTO copy_trade_users (user_pubkey)
            VALUES ($1)
            "#,
            user_pubkey
        )
        .execute(&self.pool)
        .await;

        match result {
            Ok(_) => {
                tracing::debug!(operation = "push_copytrade_pubkey", user_pubkey = %user_pubkey, "copytrade user added");
                Ok(())
            }
            Err(e) => {
                tracing::error!(operation = "push_copytrade_pubkey", user_pubkey = %user_pubkey, error = ?e, "insert failed");
                Err("Unable to push user pubkey")
            }
        }
    }

    pub async fn get_action_by_user(&self, user_pubkey: &str) {
        let row = sqlx::query_as!(
            UserAction,
            r#"
            SELECT
                *
            FROM user_actions
            WHERE user_pubkey = $1
            "#,
            user_pubkey
        )
        .fetch_all(&self.pool)
        .await;

        match row {
            Ok(data) => {
                tracing::debug!(user_pubkey = %user_pubkey, count = data.len(), "get_action_by_user");
            }
            Err(err) => {
                tracing::error!(user_pubkey = %user_pubkey, error = ?err, "get_action_by_user failed");
            }
        }
    }

    pub async fn get_copy_trade_users(&self) {
        let row = sqlx::query_as!(
            User,
            r#"
            SELECT
                *
            FROM copy_trade_users
            "#
        )
        .fetch_all(&self.pool)
        .await;

        match row {
            Ok(data) => {
                tracing::debug!(count = data.len(), "get_copy_trade_users");
            }
            Err(err) => {
                tracing::error!(error = ?err, "get_copy_trade_users failed");
            }
        }
    }

    pub async fn get_action_by_copy(&self) {
        let row = sqlx::query_as!(
            CopyTrading,
            r#"
            SELECT
                id,
                user_pubkey,
                contract_address,
                amount_b,
                amount_a,
                position_nft,
                token_a,
                token_b,
                pool_address,
                action,
                txn_sig,
                created_at,
                min_bin_id,
                max_bin_id,
                strategy,
                token_a_price,
                token_b_price,
                decimal_a,
                decimal_b
            FROM copy_trading
            "#
        )
        .fetch_all(&self.pool)
        .await;

        match row {
            Ok(data) => {
                tracing::debug!(count = data.len(), "get_action_by_copy");
            }
            Err(err) => {
                tracing::error!(error = ?err, "get_action_by_copy failed");
            }
        }
    }
}
