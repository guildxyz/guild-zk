use agora_zkp_backend::eth::*;
use agora_zkp_backend::config::get_test_config;
use std::io::{self, Write};
use core::str::FromStr;
use web3::types;

#[tokio::test]
async fn test_get_tx_by_hash() {
    let config = get_test_config();
    if config.url_rpc == "" {
        write!(
            &mut io::stdout(),
            "WARNING-test_get_tx_by_hash- no test url set, skipping test\n"
        )
        .ok();
        return;

    }
    let txhash = "0xcdf9807a415da29aedf98f9d99e3f47bdcf6cdacb46d3a167898a5bdd3ce8c41";
    let tx = get_tx_by_hash(config.url_rpc.as_str(), txhash).await.unwrap();

    let want_from = types::H160::from_str("0x18e224ad21b5a8d68a95895f16476ed4f7c2c467")
        .expect("invalid want_from hex str");
    let got_from = tx.from.unwrap();
    assert_eq!(want_from, got_from);
}
