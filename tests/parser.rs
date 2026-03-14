use met_ray_indexer::{
    constants::*,
    helper::{get_integrated_protocols, is_devnet},
    parser::{CopyTradeParseData, ParseConfigs, ParseConfigsCopyTrading, ParseData, Parser},
};

pub mod helper;

use helper::fetch_txn;

pub type CopyExpectedType = Vec<(
    ExpectedVals,
    &'static str,
    &'static str,
    i32,
    i32,
    Option<u8>,
)>;

#[derive(Debug)]
pub struct ExpectedVals {
    pub token_b: &'static str,
    pub token_a: &'static str,
    pub amount_a: u64,
    pub amount_b: u64,
    pub position_nft: Option<String>,
    pub pool_address: &'static str,
    pub owner: &'static str,
    pub contract_addr: &'static str,
}

impl PartialEq<ExpectedVals> for ParseData {
    fn eq(&self, other: &ExpectedVals) -> bool {
        self.amount_a == other.amount_a
            && self.amount_b == other.amount_b
            && self.token_a.eq(&other.token_a)
            && self.token_b.eq(&other.token_b)
            && self.position_nft.eq(&other.position_nft)
            && self.pool_address.eq(other.pool_address)
            && self.contract_address.eq(other.contract_addr)
    }
}

impl PartialEq<ExpectedVals> for CopyTradeParseData {
    fn eq(&self, other: &ExpectedVals) -> bool {
        self.amount_a == other.amount_a
            && self.amount_b == other.amount_b
            && self.token_a.eq(&other.token_a)
            && self.token_b.eq(&other.token_b)
            && self.position_nft.eq(&other.position_nft)
            && self.pool_address.eq(other.pool_address)
            && self.contract_address.eq(other.contract_addr)
    }
}

#[cfg(test)]
pub mod parser_tests {
    use super::*;

    #[tokio::test(flavor = "multi_thread")]
    pub async fn test_process_txn() {
        let integrated_protocols = get_integrated_protocols();

        let (sig, connection) = if is_devnet() {
            (
                "2sQ7zYB6NF727Pj3jVgjBrWH1QRhYhfpfvbL7mmw8De8Bbv7BFHT7KNdVWHaJPZJChQA2orGeVpygrTs9BBqPgsC",
                "https://api.devnet.solana.com",
            )
        } else {
            (
                "5VhDfoXTpiUyokwZXpZ3ub7HJnaDJGNK91e25mAVJeRySvnW46m6iigKemESppKNJiAwatgDBWpPHChrewrcxc4B",
                "https://api.mainnet-beta.solana.com/",
            )
        };

        let parsed_transactions = fetch_txn(sig, connection, &integrated_protocols);

        for txn in parsed_transactions {
            let configs = ParseConfigs {
                ix_data: txn.ix_data,
                ix_accounts: txn.ix_accounts,
                token_transfers: txn.token_transfers,
                txn: txn.txn,
            };
            let parsed_data =
                Parser::get_parsed_data(&txn.program_id, configs, &integrated_protocols);

            if let Some(parsed) = parsed_data
                && let Ok(mut data) = parsed.0
            {
                if !is_devnet() {
                    data = Parser::get_token_info(data).await;
                }

                println!("Parsed Data: {data:#?}\nOwner: {}", parsed.1);
            }
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    pub async fn process_txn_copy_wallet() {
        let integrated_protocols = get_integrated_protocols();

        let (sig, connection) = if is_devnet() {
            (
                "2rnph2kVgWNDjvPU9RzNvV3ri8tfJxKgbi3gEbVEAYuMMhGRMUg1wj39TJVDA2agRDaT55DiLVZcWeEqrBiafxMa",
                "https://api.devnet.solana.com",
            )
        } else {
            (
                "5VhDfoXTpiUyokwZXpZ3ub7HJnaDJGNK91e25mAVJeRySvnW46m6iigKemESppKNJiAwatgDBWpPHChrewrcxc4B",
                "https://api.mainnet-beta.solana.com/",
            )
        };

        let parsed_transactions = fetch_txn(sig, connection, &integrated_protocols);

        for txn in parsed_transactions {
            let configs = ParseConfigsCopyTrading {
                ix_accounts: txn.ix_accounts,
                token_transfers: txn.token_transfers,
                txn: txn.txn,
                ix_data: txn.ix_data,
            };
            let copytrade_data =
                Parser::get_copy_parsed_data(&txn.program_id, configs, &integrated_protocols);
            if let Some(parsed) = copytrade_data
                && let Ok(mut data) = parsed.0
            {
                if !is_devnet() {
                    data = Parser::get_token_info_copy(data).await;
                }
                println!("{data:#?}");
            }
        }
    }
    #[test]
    pub fn test_clmm_instructions() {
        let integrated_protocols = get_integrated_protocols();
        let url = "https://api.mainnet-beta.solana.com/";
        let contract_addr = RAYDIUM_CLMM_PUBKEY;

        let txn_configs: Vec<(ExpectedVals, &'static str, &'static str)> = vec![
            (
                ExpectedVals {
                    token_a: "So11111111111111111111111111111111111111112",
                    token_b: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
                    amount_a: 39756260,
                    amount_b: 7714037,
                    position_nft: Some(String::from(
                        "5S5vTqjXevtRWCPGowKe5VZcFKaL58p1WZYccgTmwVrm",
                    )),
                    owner: "DX7hwiYrCopv6RStR1s4c8bMg6chSL6XQoyUEhZQTtQN",
                    pool_address: "2QdhepnKRTLjjSqPL1PtKNwqrUkoLee5Gqs8bvZhRdMv",
                    contract_addr,
                },
                "nxj6yEV67ZhQFwaGwnSXGa71waGMKMoerfLptZX4NYbcpPqd3M4WE1AADDfLnxSwRovb97aW7oPgQpPYwrweV6p",
                "create_position",
            ),
            (
                ExpectedVals {
                    token_a: "So11111111111111111111111111111111111111112",
                    token_b: "USD1ttGY1N17NEEHLmELoaybftRBUSErhqYiQzvEmuB",
                    amount_a: 720000000,
                    amount_b: 106934859,
                    position_nft: Some(String::from(
                        "GBziTUt4uTmxFWnC2BcXrJwyAQh8c5CFrsBccwkZR1kv",
                    )),
                    owner: "BQPxyey6Byr1wmS52aNFwftxzR4xuuhdR9Qc6h938NPo",
                    pool_address: "AQAGYQsdU853WAKhXM79CgNdoyhrRwXvYHX6qrDyC1FS",
                    contract_addr,
                },
                "4dae4wRnwpPGBEZTVfUmYfGJ725mgJfufX6xHUJ98oCe4NhVUxEVoFfUNXYbCc34diw36oWqecgTEqu7vorQAsxD",
                "create_position",
            ),
            (
                ExpectedVals {
                    token_a: NOT_IN_TX,
                    token_b: NOT_IN_TX,
                    amount_a: 0,
                    amount_b: 0,
                    position_nft: Some(String::from(
                        "HN1N58kf3VEySRCW2k8UgfRUD72HZELiEJrs2RNaCtfp",
                    )),
                    owner: "3xbUc9cPwTX75E2Yqd4qSbZSqzud3ai3kxZuzi6rYmLw",
                    pool_address: NOT_IN_TX,
                    contract_addr,
                },
                "5mkKrx2vCFHpvTo8TU3dkkqA3oJbzdb8UHbF5XGnK6kqcfrtvRpTAJHsff2ThiGrcqfBdET5PUBhZTjey55Kq3Cb",
                "close_position",
            ),
            (
                ExpectedVals {
                    token_a: "So11111111111111111111111111111111111111112",
                    token_b: "EXF5v6r8UjLtuGCoBjgxgAeNVGtXSSqNvGFg9w9Tqtdp",
                    amount_a: 0,
                    amount_b: 0,
                    position_nft: None,
                    owner: "9nqKuDVnqH32sJDiG6aRdLYx3CKvwdRwfkQhP7PTjUQc",
                    pool_address: "DPmogfCzKPLjuFnNkaV5nuHyFifQmQVCDcoJA7tyM7zK",
                    contract_addr,
                },
                "2cFxV31RqzfvspzYjc4gjmGd2hZJTKnrTAEvE4tio7kdtNtNA4DLnn1dWTYsRBhmxz1PXZMhxcCxTG1R1qqisM4h",
                "create_pool",
            ),
            (
                ExpectedVals {
                    token_a: "So11111111111111111111111111111111111111112",
                    token_b: "USD1ttGY1N17NEEHLmELoaybftRBUSErhqYiQzvEmuB",
                    amount_a: 825880560,
                    amount_b: 51383372,
                    position_nft: Some(String::from(
                        "C7DxE5QWdJ88jnb3rd4avEVdt41uEaSHcYuL8g5eac1h",
                    )),
                    owner: "D88DvQowykbUmEMwfA3SXK48egR4pieNpVbNNrHG4pwz",
                    pool_address: "AQAGYQsdU853WAKhXM79CgNdoyhrRwXvYHX6qrDyC1FS",
                    contract_addr,
                },
                "2Bwqb2veibsgsbp4fqWgtQfJJDgY4wknLdW1HzCRg1D4qtRBJfchq5C2dr49oijLxxi76QtFrBP3dn4EXv88Ucpk",
                "add_liquidity",
            ),
            (
                ExpectedVals {
                    token_a: "3KXzESJAxePRZGHQfaoDpfo6hvCzAuNWU8ruknTAxTmq",
                    token_b: "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB",
                    amount_a: 1329993414748,
                    amount_b: 5000011566,
                    position_nft: Some(String::from(
                        "988FXsMCF17j6Ty7yteBwRbrShof5qLBSZSe1KsLViJ2",
                    )),
                    owner: "97PGZGc2h4EvpoSZqKiQFQMeHcb6cryUzjccHARHhrP2",
                    pool_address: "4oWrVoPWtN8acGts5T7bQMEwFWMEmniiZd75ogDvaiw3",
                    contract_addr,
                },
                "3PVP7jUZvCLpRXSRkMouzje3qFZCzYBBaYscDYLgaJgUHKD8KqnR3Axj2KtMj1MHyavmHm1p7hjjYgxqNP94vD9C",
                "remove_liquidity",
            ),
        ];

        for (expected, sig, ix_name) in txn_configs {
            let parsed_txn = fetch_txn(sig, url, &integrated_protocols);

            for txn in parsed_txn {
                let configs = ParseConfigs {
                    ix_data: txn.ix_data,
                    ix_accounts: txn.ix_accounts,
                    token_transfers: txn.token_transfers,
                    txn: txn.txn,
                };
                let parsed_data =
                    Parser::get_parsed_data(&txn.program_id, configs, &integrated_protocols);

                if let Some(result) = parsed_data
                    && let Ok(parsed) = result.0
                {
                    if parsed.action.eq(ix_name) {
                        if !parsed.eq(&expected) || result.1.ne(expected.owner) {
                            let err_message = &format!(
                                "expected parsed and expected to be same:\nParsed: {parsed:#?}\nExpected: {expected:#?}\nsig: {sig}\nix_name: {ix_name}\nowner: {}",
                                result.1
                            );

                            println!("{err_message}");
                            panic!()
                        }
                        if parsed.eq(&expected) && result.1.eq(expected.owner) {
                            println!("CLMM ix: {ix_name} ran successfully");
                        }
                    }
                } else {
                    println!("unable to parse the data")
                }
            }
        }
    }

    #[test]
    pub fn test_cpmm_instructions() {
        let integrated_protocols = get_integrated_protocols();
        let url = "https://api.mainnet-beta.solana.com/";
        let contract_addr = RAYDIUM_CPMM_PUBKEY;
        let txn_configs: Vec<(ExpectedVals, &'static str, &'static str)> = vec![
            (
                ExpectedVals {
                    token_a: "So11111111111111111111111111111111111111112",
                    token_b: "4FrfNaP7m6GGFM1ESoWLkPzXUYot6E6S5DaiZVPwayBt",
                    amount_a: 420174682,
                    amount_b: 476188486450605,
                    position_nft: None,
                    owner: "EneaCDvnkid46TGLbRLcdx21rEwEYPMqzD5uMe1RGfLs",
                    pool_address: "2Nn1tzsQ2hfe8Zszk43xwB4AQXwgcruWa6RVp98oeTru",
                    contract_addr,
                },
                "4F75QbVxHBxqrCwVy5fitq6siiybRQnbJuq2PY4P28m27x7SBojp9uhyxUWgUaYssWDZtxuUvDiggWj6BUkFNb36",
                "remove_liquidity",
            ),
            (
                ExpectedVals {
                    token_a: "So11111111111111111111111111111111111111112",
                    token_b: "HeLp6NuQkmYB4pYWo2zYs22mESHXPQYzXbB8n4V98jwC",
                    amount_a: 15638694169,
                    amount_b: 481649999999741,
                    position_nft: None,
                    owner: "6L9pMvL8QHrBsvXCdovmiUdLDbY2PkGrsCRyt1C3bgXV",
                    pool_address: "7qAVrzrbULwg1B13YseqA95Uapf8EVp9jQE5uipqFMoP",
                    contract_addr,
                },
                "2z35tvFdXBbN2Gs2sYnTBwWupgbw8QecwMV6Ro1MwbyBrfk72c7gXEzEsqhyUvcc7FZi2EBJQ6aN9CaKhACEFkpx",
                "add_liquidity",
            ),
            (
                ExpectedVals {
                    token_a: "So11111111111111111111111111111111111111112",
                    token_b: "HFdhZW5i1zf8UPT6Pr8HgjSbRTtfNVLeJqda2DVquSd1",
                    amount_a: 1000000000,
                    amount_b: 900000000000000,
                    position_nft: None,
                    owner: "HBNKeHDSFbGsMnKEVQfHYAUd9GWpc1qTi9EDVE2T6eMc",
                    pool_address: "D9dRAhzDcryiHdBNT3o7PYpnwfcSrcZaggFNtL95JVKP",
                    contract_addr,
                },
                "1rGhWsDGUGeNUUbPfvvqJLb9M1KJ39V9RLJqsJ7A32yhBdatttYd8NR92xQ5xBmtNeWmpgzqPqdETSkaC3ZUFFq",
                "create_pool",
            ),
        ];

        for (expected, sig, ix_name) in txn_configs {
            let parsed_txn = fetch_txn(sig, url, &integrated_protocols);

            for txn in parsed_txn {
                let configs = ParseConfigs {
                    ix_data: txn.ix_data,
                    ix_accounts: txn.ix_accounts,
                    token_transfers: txn.token_transfers,
                    txn: txn.txn,
                };
                let parsed_data =
                    Parser::get_parsed_data(&txn.program_id, configs, &integrated_protocols);

                if let Some(result) = parsed_data
                    && let Ok(parsed) = result.0
                {
                    if parsed.action.eq(ix_name) {
                        if !parsed.eq(&expected) || result.1.ne(expected.owner) {
                            let err_message = &format!(
                                "expected parsed and expected to be same:\nParsed: {parsed:#?}\nExpected: {expected:#?}\nsig: {sig}\nix_name: {ix_name}\nowner: {}",
                                result.1
                            );

                            println!("{err_message}");
                            panic!()
                        }
                        if parsed.eq(&expected) && result.1.eq(expected.owner) {
                            println!("CLMM ix: {ix_name} ran successfully");
                        }
                    }
                } else {
                    println!("unable to parse the data")
                }
            }
        }
    }

    #[test]
    pub fn test_damm_v2_instructions() {
        let integrated_protocols = get_integrated_protocols();
        let url = "https://api.mainnet-beta.solana.com/";
        let contract_addr = METEORA_DAMM_V2_PUBKEY;
        let txn_configs: Vec<(ExpectedVals, &'static str, &'static str)> = vec![
            (
                ExpectedVals {
                    token_a: "8zumDAvT34fDMvhMF6hF9qt8LXcnNT1eRK2BZFM5FUVs",
                    token_b: "So11111111111111111111111111111111111111112",
                    amount_a: 8669495437,
                    amount_b: 14574,
                    position_nft: Some(String::from(
                        "E2q2Y4M3fmwnkTcjJgut2ErSvbhRwcQzGaAbn71ZjkGc",
                    )),
                    owner: "77bbAcf2nA1tiCFpNjBUoj9uzCNx4LU28fKgnryzsqgF",
                    pool_address: "feaNAJtCRLzACQzWymAod2CBtymqFmbiZ24b2Zpf9Mi",
                    contract_addr,
                },
                "34XUtpAQ9YyNFALmxjcoZnu1Yh1GCBHFcgfwtWrx4G3SnnUTSboiBp4NcXYU1jjCfAoiQBA2mhZpYKj9xpWqpTH1",
                "claim_fee",
            ),
            (
                ExpectedVals {
                    token_a: NOT_IN_TX,
                    token_b: NOT_IN_TX,
                    amount_a: 0,
                    amount_b: 0,
                    position_nft: Some("3kHdAg92tDXmzbKStDNZoE79vMSJNvT4QDoDKVRUjwew".to_string()),
                    owner: "8Fd8JQ4uDc1fHMq7VJtGKEGXgfw4MLEZfJYQntY8kc17",
                    pool_address: "C7v2rQAbmSUtrCHFBcvNGPabebHhT2qtb5z1HNm2j8XB",
                    contract_addr,
                },
                "3b5dzNR8LmFB8hUoewtJRxon1X9jBgqtMmnQozDfcUTmix1mDiAfPYcfUbhXgkqPb24zWqozrtzMbGRqPkrzfJyD",
                "create_position",
            ),
            (
                ExpectedVals {
                    token_a: "3VW31dwix6k2EdzhDgZ2zB15J7FbHYQwAUqXgktRcJEX",
                    token_b: "So11111111111111111111111111111111111111112",
                    amount_a: 37748097700,
                    amount_b: 10000000000,
                    position_nft: Some(String::from(
                        "3kHdAg92tDXmzbKStDNZoE79vMSJNvT4QDoDKVRUjwew",
                    )),
                    owner: "8Fd8JQ4uDc1fHMq7VJtGKEGXgfw4MLEZfJYQntY8kc17",
                    pool_address: "C7v2rQAbmSUtrCHFBcvNGPabebHhT2qtb5z1HNm2j8XB",
                    contract_addr,
                },
                "3b5dzNR8LmFB8hUoewtJRxon1X9jBgqtMmnQozDfcUTmix1mDiAfPYcfUbhXgkqPb24zWqozrtzMbGRqPkrzfJyD",
                "add_liquidity",
            ),
            (
                ExpectedVals {
                    token_a: "DR9fWuj3XRFyNzxk1rvkwMMYA2WHj448HgVJduQLpump",
                    token_b: "So11111111111111111111111111111111111111112",
                    amount_a: 5736717437,
                    amount_b: 20843264,
                    position_nft: Some(String::from(
                        "HgAKQCVkkXvHfziyx7erd23RTRexXg3p54b7SpdUDFmk",
                    )),
                    owner: "CAkycfuzWE8SownBGZqVQud5MS5uZFSwg5frnuuZYJ3F",
                    pool_address: "GidSzNm3N57emtes8gpCum37YanDwnN4h6PaQ9W7XX9k",
                    contract_addr,
                },
                "4DuGB3tbdSeRuQaY5xvCXYhmgCmgFsMC8v9Pd3XJfHiL52PoN698h59M7JavccW2w7WzmckASJ4ZwK3mpVSBiTFH",
                "remove_liquidity",
            ),
            (
                ExpectedVals {
                    token_a: NOT_IN_TX,
                    token_b: NOT_IN_TX,
                    amount_a: 0,
                    amount_b: 0,
                    position_nft: Some(String::from(
                        "HgAKQCVkkXvHfziyx7erd23RTRexXg3p54b7SpdUDFmk",
                    )),
                    owner: "CAkycfuzWE8SownBGZqVQud5MS5uZFSwg5frnuuZYJ3F",
                    pool_address: "GidSzNm3N57emtes8gpCum37YanDwnN4h6PaQ9W7XX9k",
                    contract_addr,
                },
                "4DuGB3tbdSeRuQaY5xvCXYhmgCmgFsMC8v9Pd3XJfHiL52PoN698h59M7JavccW2w7WzmckASJ4ZwK3mpVSBiTFH",
                "close_position",
            ),
            (
                ExpectedVals {
                    token_a: "Guko5GPz4g6E2cjDHukGtdnDNJeQQvpEW9SCgRZcpump",
                    token_b: "So11111111111111111111111111111111111111112",
                    amount_a: 5723766674,
                    amount_b: 28322790,
                    position_nft: Some(String::from("BwWLe2yeEukUCnxZC9XebADNdbDPdQ4o9FwAjHFRiTv")),
                    owner: "BDfWUTnJ4SnNX6jfPgtAAnyzkANSSnwZKkxCSdvtn5iR",
                    pool_address: "DCYHrq1rp71NhdWKjjAy74kBWTnZxEdgmwLK9jaBZ7GF",
                    contract_addr,
                },
                "5DVKcgm4KqeAT5sfwGqSe1A55qv53i8jSHPwsN4t2aVdsNcLP5G39fSws6axmiuUyzgByXM2nhSYZTNpPGTA3FhS",
                "remove_liquidity",
            ),
            (
                ExpectedVals {
                    token_a: "So11111111111111111111111111111111111111112",
                    token_b: "7UAvzmyaYFbP4YeXyubTcYALDPSp1Dd3QuN54SQ1b2RH",
                    amount_a: 100000000,
                    amount_b: 50000000000000,
                    position_nft: Some(String::from(
                        "4qQ9BDRcBAKqMq4EYkV8KNgrv8k64Sqyy2mdDRdggavs",
                    )),
                    owner: "3UarxQFuHd6uxpCJQGWuSRFVbkJ5Vw8NmMrtZzBBC7ok",
                    pool_address: "5oP2dh7JQVdG6vbjr3AchtJyRJQkKTAav4TYvXZFo3Lj",
                    contract_addr,
                },
                "2Nw2zazhEhNUesxmyeiLsqym9wAx2pS69PsxXN6mEVkvbjYqtQdqeCn2B1gWT2pf7mNnn3TjCnEYBWVtbQofhP4g",
                "create_pool",
            ),
        ];

        for (expected, sig, ix_name) in txn_configs {
            let parsed_txn = fetch_txn(sig, url, &integrated_protocols);

            for txn in parsed_txn {
                let configs = ParseConfigs {
                    ix_data: txn.ix_data,
                    ix_accounts: txn.ix_accounts,
                    token_transfers: txn.token_transfers,
                    txn: txn.txn,
                };
                let parsed_data =
                    Parser::get_parsed_data(&txn.program_id, configs, &integrated_protocols);

                if let Some(result) = parsed_data
                    && let Ok(parsed) = result.0
                {
                    if parsed.action.eq(ix_name) {
                        if !parsed.eq(&expected) || result.1.ne(expected.owner) {
                            let err_message = &format!(
                                "expected parsed and expected to be same:\nParsed: {parsed:#?}\nExpected: {expected:#?}\nsig: {sig}\nix_name: {ix_name}\nowner: {}",
                                result.1
                            );

                            println!("{err_message}");
                            panic!()
                        }
                        if parsed.eq(&expected) && result.1.eq(expected.owner) {
                            println!("CLMM ix: {ix_name} ran successfully");
                        }
                    }
                } else {
                    println!("unable to parse the data")
                }
            }
        }
    }

    #[test]
    pub fn test_dlmm_instructions() {
        let integrated_protocols = get_integrated_protocols();
        let url = "https://api.mainnet-beta.solana.com/";
        let contract_addr = METEORA_DLMM_PUBKEY;
        let txn_configs: Vec<(ExpectedVals, &'static str, &'static str)> = vec![
            (
                ExpectedVals {
                    token_a: NOT_IN_TX,
                    token_b: NOT_IN_TX,
                    amount_a: 0,
                    amount_b: 0,
                    position_nft: Some(String::from("xgVMikWC2yuxgV3Ru8D9os5sikshsNXCPFFv6zpzuhp")),
                    owner: "GnnqGmjG5GhstDvjV7x5dHm27XqJuK8vp9Qh7uwgViqF",
                    pool_address: "8eybKAvjKJryVweQLg8SRgwUfdP7wHYJ5yyqgfE82DQA",
                    contract_addr,
                },
                "2xmNz8FmWrrGvatB59TcENVAWZQ5kPPJujKwYbrarRWzy3rHfF2Lb6w21sprNgkr4yHFYGtiuQi1afyJ6GTnweEg",
                "create_position",
            ),
            (
                ExpectedVals {
                    token_a: "Gbu7JAKhTVtGyRryg8cYPiKNhonXpUqbrZuCDjfUpump",
                    token_b: "So11111111111111111111111111111111111111112",
                    amount_a: 122538538,
                    amount_b: 0,
                    position_nft: Some("GXLk6vvKC8sXtF4zSaeS2g9zEaoojdWYeF6Ca2B7bRUa".to_string()),
                    owner: "Am5m2AcstNBXbBDqAe5kp711GPCjaMSGL9uha3a91aso",
                    pool_address: "7Y9uRQ7Q3tw2MfSwXTE9UPDUpA5WTd9BXNrTCgSN62uP",
                    contract_addr,
                },
                "5cXEWnru3sNzKy7mABc5gJ1eKtAw4fo417B7HJfDxUXuZ4UdF3qoFHvhvX5XjfmDkWQH28yCQPaK1H9wn1gT9uqV",
                "claim_fee",
            ),
            (
                ExpectedVals {
                    token_a: NOT_IN_TX,
                    token_b: NOT_IN_TX,
                    amount_a: 0,
                    amount_b: 0,
                    position_nft: Some("5t7vo33P6p2RRyQ9Cmx9Kka3zTBhuFdNUevg9WnwP3HK".to_string()),
                    owner: "ENzQm3HBoFtFairtKLDyoxgT8uBsRnZZuXyBh14XpUqj",
                    pool_address: NOT_IN_TX,
                    contract_addr,
                },
                "3gmMk7PdTqdvBB32cpAoEySEHurjZrUPZC4yy2tDa3orq5eEdDFHGwfvCKN5vsAmnUqvrtcR1YZh7HWNoCrgmhfV",
                "close_position",
            ),
            (
                ExpectedVals {
                    token_a: NOT_IN_TX,
                    token_b: NOT_IN_TX,
                    amount_a: 0,
                    amount_b: 0,
                    position_nft: Some("4AuQ8t3DKKNTco2kdWNaSSbNyFiYLLcohXK4KgMUH8ey".to_string()),
                    owner: "HG3VJwVbDm6w6Wuww2GxTn8ZCFsiYHaMzh7YEL4hie1p",
                    pool_address: NOT_IN_TX,
                    contract_addr,
                },
                "2dZWvNCwXPoD4u6rzrtdskXvvbhwX8zGgMAmjEJDasiGKspzTxKvAYfjcW3kAqGHxxvQEimMrgTJpgcghTaZNqke",
                "close_position",
            ),
            (
                ExpectedVals {
                    token_a: "GfoHcDe6342RCovwamp7reSX3ekURJauSdM5dhAqY9oK",
                    token_b: "So11111111111111111111111111111111111111112",
                    amount_a: 0,
                    amount_b: 0,
                    position_nft: None,
                    owner: "4qBthv1sKu1Pdpu8yrDeRXTuGFwUXmFhARk9R9DEsZcC",
                    pool_address: "GqbT9GERzPAbFPkeUiSaU9HtWnJCnhxFLHF3eCYmcfXv",
                    contract_addr,
                },
                "3UdYF3fUJmNPUKbqjcCAp8gVKVR8iGJgio8pc29VBZuRcb9LJ82ZpJHNBBFH86mgzrLSmtCE9Bduqey6sZzHv8AX",
                "create_pool",
            ),
            (
                ExpectedVals {
                    token_a: "Gbu7JAKhTVtGyRryg8cYPiKNhonXpUqbrZuCDjfUpump",
                    token_b: "So11111111111111111111111111111111111111112",
                    amount_a: 0,
                    amount_b: 999818097,
                    position_nft: Some("8SkXsNJDUyYBRE4T4DYKBhZCbFGFeqti57wHAsBBQmhP".to_string()),
                    owner: "2ubvt5sMSjn1NNDj4fVMP8zpkvcb1PEDQVj2XoBCD5NP",
                    pool_address: "AJxm68TtYrMBez1C1ghDLK3kDXkfyUrzfrfdntYj2X9b",
                    contract_addr,
                },
                "3m2AT3PiNpLktXpdH3siqNjSdngjGoYdz1TB6cpTt8Bx5Y4NWgJtgQwneH72PTd2U7AaFAGJxKDHohVvvLkSvaVJ",
                "remove_liquidity",
            ),
            (
                ExpectedVals {
                    token_a: "9AvytnUKsLxPxFHFqS6VLxaxt5p6BhYNr53SD2Chpump",
                    token_b: "So11111111111111111111111111111111111111112",
                    amount_a: 0,
                    amount_b: 2404319603,
                    position_nft: Some("6PyNLNv4vSmeYjdt5RqYrmV9VxT6MrDRcMrWjFUBCzWU".to_string()),
                    owner: "FtLaLStivmGLmxmykxQxLrCfyn6kZqkZGq9YbFyE2Dpc",
                    pool_address: "3d8m2UMTudV4kW1WbjFzRVV9fbBmWVCfbCiB3wMJapZw",
                    contract_addr,
                },
                "4hbW8Z4xP3RkyMRVx28qUoLgTddQ51cSWmwC8h7ea6HKdHP5zJAvJpXMmxKrRSBCBj2nYTVumDb2tgx2cwDXt2CA",
                "add_liquidity",
            ),
        ];

        for (expected, sig, ix_name) in txn_configs {
            let parsed_txn = fetch_txn(sig, url, &integrated_protocols);

            for txn in parsed_txn {
                let configs = ParseConfigs {
                    ix_data: txn.ix_data,
                    ix_accounts: txn.ix_accounts,
                    token_transfers: txn.token_transfers,
                    txn: txn.txn,
                };
                let parsed_data =
                    Parser::get_parsed_data(&txn.program_id, configs, &integrated_protocols);

                if let Some(result) = parsed_data
                    && let Ok(parsed) = result.0
                {
                    if parsed.action.eq(ix_name) {
                        if !parsed.eq(&expected) || result.1.ne(expected.owner) {
                            let err_message = &format!(
                                "expected parsed and expected to be same:\nParsed: {parsed:#?}\nExpected: {expected:#?}\nsig: {sig}\nix_name: {ix_name}\nowner: {}",
                                result.1
                            );

                            println!("{err_message}");
                            panic!()
                        }
                        if parsed.eq(&expected) && result.1.eq(expected.owner) {
                            println!("CLMM ix: {ix_name} ran successfully");
                        }
                    }
                } else {
                    println!("unable to parse the data")
                }
            }
        }
    }

    #[test]
    pub fn test_clmm_copy_instructions() {
        let integrated_protocols = get_integrated_protocols();
        let url = "https://api.mainnet-beta.solana.com/";
        let contract_addr = RAYDIUM_CLMM_PUBKEY;

        let txn_configs: CopyExpectedType = vec![
            (
                ExpectedVals {
                    token_a: "So11111111111111111111111111111111111111112",
                    token_b: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
                    amount_a: 39756260,
                    amount_b: 7714037,
                    position_nft: Some(String::from(
                        "5S5vTqjXevtRWCPGowKe5VZcFKaL58p1WZYccgTmwVrm",
                    )),
                    owner: "DX7hwiYrCopv6RStR1s4c8bMg6chSL6XQoyUEhZQTtQN",
                    pool_address: "2QdhepnKRTLjjSqPL1PtKNwqrUkoLee5Gqs8bvZhRdMv",
                    contract_addr,
                },
                "nxj6yEV67ZhQFwaGwnSXGa71waGMKMoerfLptZX4NYbcpPqd3M4WE1AADDfLnxSwRovb97aW7oPgQpPYwrweV6p",
                "create_position",
                -161190,
                0,
                None,
            ),
            (
                ExpectedVals {
                    token_a: "So11111111111111111111111111111111111111112",
                    token_b: "USD1ttGY1N17NEEHLmELoaybftRBUSErhqYiQzvEmuB",
                    amount_a: 720000000,
                    amount_b: 106934859,
                    position_nft: Some(String::from(
                        "GBziTUt4uTmxFWnC2BcXrJwyAQh8c5CFrsBccwkZR1kv",
                    )),
                    owner: "BQPxyey6Byr1wmS52aNFwftxzR4xuuhdR9Qc6h938NPo",
                    pool_address: "AQAGYQsdU853WAKhXM79CgNdoyhrRwXvYHX6qrDyC1FS",
                    contract_addr,
                },
                "4dae4wRnwpPGBEZTVfUmYfGJ725mgJfufX6xHUJ98oCe4NhVUxEVoFfUNXYbCc34diw36oWqecgTEqu7vorQAsxD",
                "create_position",
                -21540,
                -20040,
                None,
            ),
        ];

        for (expected, sig, ix_name, min_bin, max_bin, strategy) in txn_configs {
            let parsed_txn = fetch_txn(sig, url, &integrated_protocols);

            for txn in parsed_txn {
                let configs = ParseConfigsCopyTrading {
                    ix_accounts: txn.ix_accounts,
                    token_transfers: txn.token_transfers,
                    txn: txn.txn,
                    ix_data: txn.ix_data,
                };
                let parsed_data =
                    Parser::get_copy_parsed_data(&txn.program_id, configs, &integrated_protocols);

                if let Some(result) = parsed_data
                    && let Ok(parsed) = result.0
                {
                    if parsed.action.eq(ix_name) {
                        if !parsed.eq(&expected)
                            || result.1.ne(expected.owner)
                            || parsed.min_bin_id.ne(&min_bin)
                            || parsed.max_bin_id.ne(&max_bin)
                            || parsed.strategy.ne(&strategy)
                        {
                            let err_message = &format!(
                                "expected parsed and expected to be same:\nParsed: {parsed:#?}\nExpected: {expected:#?}\nmin: {min_bin}\nmax: {max_bin}\nStrategy: {strategy:#?}\nsig: {sig}\nix_name: {ix_name}\nowner: {}",
                                result.1
                            );

                            println!("{err_message}");
                            panic!()
                        }
                        if parsed.eq(&expected)
                            && result.1.eq(expected.owner)
                            && result.1.eq(expected.owner)
                            && parsed.min_bin_id.eq(&min_bin)
                            && parsed.max_bin_id.eq(&max_bin)
                            && parsed.strategy.eq(&strategy)
                        {
                            println!("CLMM ix: {ix_name} ran successfully");
                        } else {
                            println!("an unknown error occured");
                        }
                    }
                } else {
                    println!("unable to parse the data")
                }
            }
        }
    }

    #[test]
    pub fn test_dlmm_copy_instructions() {
        let integrated_protocols = get_integrated_protocols();
        let url = "https://api.mainnet-beta.solana.com/";
        let contract_addr = METEORA_DLMM_PUBKEY;
        let txn_configs: CopyExpectedType = vec![
            (
                ExpectedVals {
                    token_a: "Gbu7JAKhTVtGyRryg8cYPiKNhonXpUqbrZuCDjfUpump",
                    token_b: "So11111111111111111111111111111111111111112",
                    amount_a: 0,
                    amount_b: 999818097,
                    position_nft: Some("8SkXsNJDUyYBRE4T4DYKBhZCbFGFeqti57wHAsBBQmhP".to_string()),
                    owner: "2ubvt5sMSjn1NNDj4fVMP8zpkvcb1PEDQVj2XoBCD5NP",
                    pool_address: "AJxm68TtYrMBez1C1ghDLK3kDXkfyUrzfrfdntYj2X9b",
                    contract_addr,
                },
                "3m2AT3PiNpLktXpdH3siqNjSdngjGoYdz1TB6cpTt8Bx5Y4NWgJtgQwneH72PTd2U7AaFAGJxKDHohVvvLkSvaVJ",
                "remove_liquidity",
                -447,
                -378,
                None,
            ),
            (
                ExpectedVals {
                    token_a: "9AvytnUKsLxPxFHFqS6VLxaxt5p6BhYNr53SD2Chpump",
                    token_b: "So11111111111111111111111111111111111111112",
                    amount_a: 0,
                    amount_b: 2404319603,
                    position_nft: Some("6PyNLNv4vSmeYjdt5RqYrmV9VxT6MrDRcMrWjFUBCzWU".to_string()),
                    owner: "FtLaLStivmGLmxmykxQxLrCfyn6kZqkZGq9YbFyE2Dpc",
                    pool_address: "3d8m2UMTudV4kW1WbjFzRVV9fbBmWVCfbCiB3wMJapZw",
                    contract_addr,
                },
                "4hbW8Z4xP3RkyMRVx28qUoLgTddQ51cSWmwC8h7ea6HKdHP5zJAvJpXMmxKrRSBCBj2nYTVumDb2tgx2cwDXt2CA",
                "add_liquidity",
                -663,
                -663,
                Some(0),
            ),
        ];

        for (expected, sig, ix_name, min_bin, max_bin, strategy) in txn_configs {
            let parsed_txn = fetch_txn(sig, url, &integrated_protocols);

            for txn in parsed_txn {
                let configs = ParseConfigsCopyTrading {
                    ix_accounts: txn.ix_accounts,
                    token_transfers: txn.token_transfers,
                    txn: txn.txn,
                    ix_data: txn.ix_data,
                };
                let parsed_data =
                    Parser::get_copy_parsed_data(&txn.program_id, configs, &integrated_protocols);

                if let Some(result) = parsed_data
                    && let Ok(parsed) = result.0
                {
                    if parsed.action.eq(ix_name) {
                        if !parsed.eq(&expected)
                            || result.1.ne(expected.owner)
                            || parsed.min_bin_id.ne(&min_bin)
                            || parsed.max_bin_id.ne(&max_bin)
                            || parsed.strategy.ne(&strategy)
                        {
                            let err_message = &format!(
                                "expected parsed and expected to be same:\nParsed: {parsed:#?}\nExpected: {expected:#?}\nmin: {min_bin}\nmax: {max_bin}\nStrategy: {strategy:#?}\nsig: {sig}\nix_name: {ix_name}\nowner: {}",
                                result.1
                            );

                            println!("{err_message}");
                            panic!()
                        }
                        if parsed.eq(&expected)
                            && result.1.eq(expected.owner)
                            && result.1.eq(expected.owner)
                            && parsed.min_bin_id.eq(&min_bin)
                            && parsed.max_bin_id.eq(&max_bin)
                            && parsed.strategy.eq(&strategy)
                        {
                            println!("CLMM ix: {ix_name} ran successfully");
                        } else {
                            println!("an unknown error occured");
                        }
                    }
                } else {
                    println!("unable to parse the data")
                }
            }
        }
    }
}
