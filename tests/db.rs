use llp_indexer::{
    constants::METEORA_DLMM_PUBKEY,
    db::*,
    parser::{Actions, ParseData},
};
use sqlx::{Pool, Postgres};

#[cfg(test)]
pub struct NewParseArgs<'info> {
    position_nft: &'info str,
    action: Actions,
    amount_a: u64,
    amount_b: u64,
    token_a: &'info str,
    token_b: &'info str,
    pool_address: &'info str,
}

#[cfg(test)]
pub mod test_db {
    use super::*;
    use rand::prelude::*;
    use solana_sdk::pubkey::Pubkey;
    use std::ops::{Add, Div, Mul};

    pub fn new_parse_data(params: &NewParseArgs) -> ParseData {
        let NewParseArgs {
            position_nft,
            action,
            amount_a,
            amount_b,
            token_a,
            token_b,
            pool_address,
        } = params;

        let action = action.clone();

        ParseData {
            contract_address: METEORA_DLMM_PUBKEY.to_string(),
            amount_b: *amount_b,
            amount_a: *amount_a,
            position_nft: Some(position_nft.to_string()),
            token_a: token_a.to_string(),
            token_b: token_b.to_string(),
            pool_address: pool_address.to_string(),
            token_a_price: Some(10.0),
            token_b_price: Some(10.0),
            action: action.as_str().to_string(),
            txn_sig: String::from("TXN_SIG"),
            decimal_a: Some(1),
            decimal_b: Some(1), // for simplicity
        }
    }

    pub async fn get_users(pool: &Pool<Postgres>) {
        match sqlx::query_as!(
            UserAction,
            r#"
            SELECT
                *
            FROM user_actions
            "#,
        )
        .fetch_all(pool)
        .await
        {
            Ok(val) => {
                for user_action in val {
                    println!("{user_action:#?}");
                }
            }
            Err(_) => println!("Couldn't get user actions"),
        }
    }

    pub async fn get_copy(pool: &Pool<Postgres>) {
        match sqlx::query_as!(
            CopyTrading,
            r#"
            SELECT
                *
            FROM copy_trading
            "#,
        )
        .fetch_all(pool)
        .await
        {
            Ok(val) => {
                for user_action in val {
                    println!("{user_action:#?}");
                }
            }
            Err(_) => println!("Couldn't get copytrade values"),
        }
    }

    pub async fn delete_users_db(pool: &Pool<Postgres>) {
        match sqlx::query_as!(
            UserActions,
            r#"
        DELETE FROM user_actions
        "#
        )
        .execute(pool)
        .await
        {
            Ok(_) => println!("Success"),
            Err(_) => println!("Couldn't delete"),
        }
    }

    pub async fn delete_copytrade_users_db(pool: &Pool<Postgres>) {
        match sqlx::query_as!(
            UserActions,
            r#"
        DELETE FROM copy_trade_users
        "#
        )
        .execute(pool)
        .await
        {
            Ok(_) => println!("Success"),
            Err(_) => println!("Couldn't delete"),
        }
    }

    pub async fn get_fee_history(pool: &Pool<Postgres>) {
        match sqlx::query_as!(
            Referrals,
            r#"
            SELECT
                *
            FROM fee_history
            "#,
        )
        .fetch_all(pool)
        .await
        {
            Ok(val) => {
                for referral in val {
                    println!("{referral:#?}");
                }
            }
            Err(_) => println!("Couldn't delete"),
        }
    }

    #[tokio::test]
    pub async fn test_db_connect() {
        match DBOps::connect().await {
            Ok(_) => {
                println!("DB connected")
            }
            Err(err) => println!("Got error:: {err:#?}",),
        }
    }

    #[tokio::test]
    #[ignore = "Ignored since it clutters up the actual database"]
    pub async fn test_user_action_correct() {
        let mut rng = rand::rng();

        let random_number = rng.random::<u8>();

        let user = Pubkey::new_from_array([random_number; 32]).to_string();
        let position_nft = Pubkey::new_from_array([random_number + 1; 32]).to_string();
        let pool_address = Pubkey::new_unique().to_string();
        let token_a = Pubkey::new_unique().to_string();
        let token_b = Pubkey::new_unique().to_string();

        let db = DBOps::connect().await.unwrap();

        let add_liquidity_data = NewParseArgs {
            action: Actions::AddLiquidity,
            amount_a: 100,
            amount_b: 100,
            pool_address: &pool_address,
            position_nft: &position_nft,
            token_a: &token_a,
            token_b: &token_b,
        };
        let add_liquidity_parsed_data = new_parse_data(&add_liquidity_data);

        db.push_db(&add_liquidity_parsed_data, &user).await.unwrap();

        let remove_liquidity_data = NewParseArgs {
            action: Actions::RemoveLiquidity,
            amount_a: 50,
            amount_b: 50,
            pool_address: &pool_address,
            position_nft: &position_nft,
            token_a: &token_a,
            token_b: &token_b,
        };
        let remove_liquidity_parsed_data = new_parse_data(&remove_liquidity_data);

        db.push_db(&remove_liquidity_parsed_data, &user)
            .await
            .unwrap();

        let claim_fee_data = NewParseArgs {
            action: Actions::ClaimFee,
            amount_a: 25,
            amount_b: 25,
            pool_address: &pool_address,
            position_nft: &position_nft,
            token_a: &token_a,
            token_b: &token_b,
        };
        let claim_fee_parsed_data = new_parse_data(&claim_fee_data);

        db.push_db(&claim_fee_parsed_data, &user).await.unwrap();

        match db
            .get_position_activity_row(&position_nft, "TX_HASH".to_string())
            .await
        {
            Ok(data) => {
                println!("Got data: {data:#?}");

                let amount_a: f64 = add_liquidity_parsed_data
                    .amount_a
                    .to_string()
                    .parse()
                    .unwrap();
                let amount_b: f64 = add_liquidity_parsed_data
                    .amount_b
                    .to_string()
                    .parse()
                    .unwrap();

                let total_deposit_a = amount_a
                    .mul(add_liquidity_parsed_data.token_a_price.unwrap())
                    .div(10f64.powf(add_liquidity_parsed_data.decimal_a.unwrap() as f64));

                let total_deposit_b = amount_b
                    .mul(add_liquidity_parsed_data.token_b_price.unwrap())
                    .div(10f64.powf(add_liquidity_parsed_data.decimal_b.unwrap() as f64));

                let remove_amount_a: f64 = remove_liquidity_parsed_data
                    .amount_a
                    .to_string()
                    .parse()
                    .unwrap();
                let remove_amount_b: f64 = remove_liquidity_parsed_data
                    .amount_b
                    .to_string()
                    .parse()
                    .unwrap();

                let total_remove_a = remove_amount_a
                    .mul(remove_liquidity_parsed_data.token_a_price.unwrap())
                    .div(10f64.powf(remove_liquidity_parsed_data.decimal_a.unwrap() as f64));

                let total_remove_b = remove_amount_b
                    .mul(remove_liquidity_parsed_data.token_b_price.unwrap())
                    .div(10f64.powf(remove_liquidity_parsed_data.decimal_b.unwrap() as f64));

                let claim_amount_a: f64 =
                    claim_fee_parsed_data.amount_a.to_string().parse().unwrap();
                let claim_amount_b: f64 =
                    claim_fee_parsed_data.amount_b.to_string().parse().unwrap();

                let total_claim_a = claim_amount_a
                    .mul(claim_fee_parsed_data.token_a_price.unwrap())
                    .div(10f64.powf(claim_fee_parsed_data.decimal_a.unwrap() as f64));

                let total_claim_b = claim_amount_b
                    .mul(claim_fee_parsed_data.token_b_price.unwrap())
                    .div(10f64.powf(claim_fee_parsed_data.decimal_b.unwrap() as f64));

                let total_deposit = total_deposit_a.add(total_deposit_b);
                let total_remove = total_remove_a.add(total_remove_b);
                let total_claim = total_claim_a.add(total_claim_b);

                let total_withdraw = total_remove.add(total_claim);

                println!(
                    "total deposit: {total_deposit}\ntotal_withdraw: {total_withdraw}\ntotal_remove: {total_remove}\ntotal_claim: {total_claim}"
                );

                assert!(data.total_deposit.eq(&total_deposit));
                assert!(data.total_withdraw.eq(&total_withdraw));
            }
            Err(e) => {
                println!("{e}")
            }
        }
    }

    #[tokio::test]
    pub async fn test_db_fetch_works() {
        let user = Pubkey::new_from_array([0x3; 32]).to_string();

        println!("fetching rows for {user}");
        DBOps::connect()
            .await
            .unwrap()
            .get_action_by_user(&user)
            .await;
    }

    #[tokio::test]
    pub async fn test_db_fetch_works_copy() {
        DBOps::connect().await.unwrap().get_action_by_copy().await;
    }

    #[tokio::test]
    #[ignore = "WARNING: can delete user data"]
    pub async fn delete_users() {
        let db = DBOps::connect().await.unwrap();
        delete_users_db(&db.pool).await;
    }

    #[tokio::test]
    #[ignore = "WARNING: can delete user data"]
    pub async fn delete_copytrade_users() {
        let db = DBOps::connect().await.unwrap();
        delete_copytrade_users_db(&db.pool).await;
    }

    #[tokio::test]
    pub async fn test_get_users_work() {
        let db = DBOps::connect().await.unwrap();

        match db.get_users().await {
            Ok((users, copy_trade_users)) => {
                println!("---------------------Users-----------------------");
                for user in &users {
                    println!("{user}");
                }
                println!("-----------------Copytrade users------------------");
                for copytrade in copy_trade_users {
                    println!("{copytrade}");
                }
            }
            Err(e) => {
                println!("{e}");
            }
        }
    }

    #[tokio::test]
    pub async fn test_get_activities() {
        let db = DBOps::connect().await.unwrap();
        let position_pubkey = "";

        match db
            .get_position_activity_row(position_pubkey, "".to_string())
            .await
        {
            Ok(_) => {}
            Err(e) => println!("Error getting the activity: {e:#?}"),
        }
    }

    #[tokio::test]
    pub async fn test_get_fee_history() {
        let db = DBOps::connect().await.unwrap();

        get_fee_history(&db.pool).await;
    }

    #[tokio::test]
    pub async fn test_get_users() {
        let db = DBOps::connect().await.unwrap();

        get_users(&db.pool).await;
    }
}
