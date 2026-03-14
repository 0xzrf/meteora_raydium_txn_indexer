use crate::helper::*;
use serde::Deserialize;
#[derive(Deserialize, Debug)]
struct ApiResponseDamm {
    data: PoolEntryDamm,
}

#[derive(Deserialize, Debug)]
struct PoolEntryDamm {
    token_a_mint: String,
    token_b_mint: String,
}

pub async fn get_tokens_from_pool_damm(pool: &str) -> Option<(String, String)> {
    let url = get_damm_api_url();

    println!("Fetchin mints from pool: {pool}");
    let req_url = format!("{url}/pools/{pool} in DAMM-V2");

    let res = reqwest::get(req_url)
        .await
        .map_err(|_| Option::<f64>::None)
        .ok()?
        .text()
        .await
        .map_err(|_| Option::<f64>::None)
        .ok()?;

    let parsed: Result<ApiResponseDamm, ()> = serde_json::from_str(&res).map_err(|_| ());

    match parsed {
        Ok(parsed) => {
            let mint_a = parsed.data.token_a_mint.clone();

            let mint_b = parsed.data.token_b_mint.clone();

            Some((mint_a, mint_b))
        }
        Err(_) => None,
    }
}

#[derive(Deserialize, Debug)]
struct ApiResponseDlmm {
    mint_x: String,
    mint_y: String,
}

pub async fn get_tokens_from_pool_dlmm(pool: &str) -> Option<(String, String)> {
    let url = get_dlmm_api_url();
    let req_url = format!("{url}/pair/{pool}");

    println!("Fetchin mints from pool: {pool} in DLMM");
    let res = reqwest::get(req_url)
        .await
        .map_err(|_| Option::<f64>::None)
        .ok()?
        .text()
        .await
        .map_err(|_| Option::<f64>::None)
        .ok()?;

    let parsed: Result<ApiResponseDlmm, ()> = serde_json::from_str(&res).map_err(|_| ());

    match parsed {
        Ok(parsed) => {
            let mint_a = parsed.mint_x.clone();

            let mint_b = parsed.mint_y.clone();

            Some((mint_a, mint_b))
        }
        Err(_) => None,
    }
}
