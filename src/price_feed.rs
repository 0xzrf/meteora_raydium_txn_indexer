use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct Token {
    usdPrice: f64, // Intentional
    decimals: u8,
}

pub async fn get_price_for_token(token: &str) -> Option<(f64, u8)> {
    println!("fetching price for: {token}");

    let res = reqwest::get(format!(
        "https://datapi.jup.ag/v1/assets/search?query={token}"
    ))
    .await
    .map_err(|_| Option::<f64>::None)
    .ok()?
    .text()
    .await
    .map_err(|_| Option::<f64>::None)
    .ok()?;

    let tokens: Vec<Token> = match serde_json::from_str(&res) {
        Ok(tokens) => tokens,
        Err(_) => return None,
    };

    if tokens.is_empty() {
        return None;
    }

    Some((tokens[0].usdPrice, tokens[0].decimals))
}

#[cfg(test)]
pub mod test_get_token {
    use super::*;

    #[tokio::test]
    pub async fn get_token_price_test() {
        match get_price_for_token("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v").await {
            Some((price, decimals)) => {
                println!("USD value for sol:: $ {price}\nDecimals: {decimals}")
            }
            None => println!("Couldn't fetch SOL value"),
        }
    }
}
