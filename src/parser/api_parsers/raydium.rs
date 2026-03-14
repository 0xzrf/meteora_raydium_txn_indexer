use crate::helper::get_raydium_amm_api;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct ApiResponseRaydium {
    data: Vec<PoolEntryRaydium>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct PoolEntryRaydium {
    mint_a: MintInfo,
    mint_b: MintInfo,
}

#[derive(Deserialize, Debug)]
struct MintInfo {
    address: String,
}

pub async fn get_tokens_from_raydium_amm(pool: &str) -> Option<(String, String)> {
    let url = get_raydium_amm_api();

    let req_url = format!("{url}/pools/key/ids?ids={pool}");

    println!("Fetchin mints from pool: {pool}");
    let res = reqwest::get(req_url)
        .await
        .map_err(|_| Option::<f64>::None)
        .ok()?
        .text()
        .await
        .map_err(|_| Option::<f64>::None)
        .ok()?;

    let parsed: Result<ApiResponseRaydium, ()> = serde_json::from_str(&res).map_err(|_| ());

    match parsed {
        Ok(parsed) => {
            // Get the first pool entry from the data array
            let pool_entry = parsed.data.first()?;

            let mint_a = pool_entry.mint_a.address.clone();
            let mint_b = pool_entry.mint_b.address.clone();

            Some((mint_a, mint_b))
        }
        Err(_) => None,
    }
}
