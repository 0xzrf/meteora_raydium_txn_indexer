#[cfg(test)]
pub mod test_fetch_tokens {
    use llp_indexer::{helper::is_devnet, parser::api_parsers::*};

    #[tokio::test]
    pub async fn test_get_tokens_damm_v2() {
        let pool = if is_devnet() {
            "12bwsQFRee39NJup2pLw7xRoMeRmBDKNL8za26AZ6LqW"
        } else {
            "11BWLuxs8ow5x42hXjVPi55j9KLVa4SCn1MspbBepVQ"
        };

        match get_tokens_from_pool_damm(pool).await {
            Some((mint_a, mint_b)) => {
                println!("Fetched tokens from damm");
                println!("MintA: {mint_a}\nMintB: {mint_b}");
            }
            None => {
                println!("Couldn't fetch mints");
                panic!()
            }
        }
    }

    #[tokio::test]
    pub async fn test_get_tokens_dlmm() {
        let pool = if is_devnet() {
            "9s5jk7AMrQx8ELYWApkiMLm3UkiDpegf6eaYmkXmMzVC"
        } else {
            "Ammd26gvZEdmqJETLWiLqn3VpioUKWYJdFeC95vk46Yv"
        };

        match get_tokens_from_pool_dlmm(pool).await {
            Some((mint_a, mint_b)) => {
                println!("Fetched tokens from dlmm pool");
                println!("MintA: {mint_a}\nMintB: {mint_b}");
            }
            None => {
                println!("Couldn't fetch mints");
                panic!()
            }
        }
    }

    #[tokio::test]
    pub async fn test_get_tokens_amm() {
        let pool = if is_devnet() {
            ""
        } else {
            "8WwcNqdZjCY5Pt7AkhupAFknV2txca9sq6YBkGzLbvdt"
        };

        match get_tokens_from_raydium_amm(pool).await {
            Some((mint_a, mint_b)) => {
                println!("Fetched tokens from amm pool");
                println!("MintA: {mint_a}\nMintB: {mint_b}");
            }
            None => {
                println!("Couldn't fetch mints");
                panic!()
            }
        }
    }
}
