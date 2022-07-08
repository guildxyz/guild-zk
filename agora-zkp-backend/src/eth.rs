use core::str::FromStr;
use web3::types;

pub async fn get_tx_by_hash(rpc_uri: &str, txhash: &str) -> Option<types::Transaction> {
    // TODO replace expect with better error handling
    let transport = web3::transports::Http::new(rpc_uri).expect("connect to transport failed");
    let web3 = web3::Web3::new(transport);

    let txhash = types::H256::from_str(txhash).expect("invalid tx hash");
    let txid = types::TransactionId::Hash(txhash);
    let tx = web3
        .eth()
        .transaction(txid)
        .await
        .expect("failed to get tx by hash");
    tx
}

pub fn get_pubkey_from_tx(tx: types::Transaction) {
    let txstr = serde_json::to_string(&tx).expect("serializing transaction");
    let _rawtx: types::RawTransaction = serde_json::from_str(&txstr).unwrap();
    // web3::signing::recover();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_tx_by_hash() {
        let rpc_uri =
            "https://speedy-nodes-nyc.moralis.io/b9aed21e7bb7bdeb35972c9a/eth/mainnet/archive";
        let txhash = "0xcdf9807a415da29aedf98f9d99e3f47bdcf6cdacb46d3a167898a5bdd3ce8c41";
        let tx = get_tx_by_hash(rpc_uri, txhash).await.unwrap();
        let txstr = serde_json::to_string(&tx).expect("serializing transaction");

        let want_from = types::H160::from_str("0x18e224ad21b5a8d68a95895f16476ed4f7c2c467")
            .expect("invalid want_from hex str");
        let got_from = tx.from.unwrap();
        println!("want: {} got: {}", want_from, got_from);
        assert_eq!(want_from, got_from);
    }
}
