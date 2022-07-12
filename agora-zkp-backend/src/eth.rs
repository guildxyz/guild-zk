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
