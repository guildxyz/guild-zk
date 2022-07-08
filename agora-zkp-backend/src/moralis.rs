#[derive(serde::Deserialize, Debug)]
pub struct Resp {
    pub result: Vec<TxResult>,
}

#[derive(serde::Deserialize, Debug)]
pub struct TxResult {
    pub from_address: String,
    pub to_address: String,
    pub hash: String,
}

#[derive(Debug)]
pub enum MoralisError {
    ReqError(reqwest::Error),
    NotFoundError,
}

pub async fn get_txhash_by_sender_addr(
    apikey: String,
    address: String,
) -> Result<String, MoralisError> {
    let resp = get_txs_by_addr(apikey, address.clone())
        .await
        .map_err(|e| MoralisError::ReqError(e))?;
    match resp.find_from_tx_hash(address) {
        None => Err(MoralisError::NotFoundError),
        Some(hash) => Ok(hash),
    }
}

async fn get_txs_by_addr(apikey: String, address: String) -> Result<Resp, reqwest::Error> {
    let url = format!(
        "https://deep-index.moralis.io/api/v2/{}?chain=eth&from_block=0",
        address
    );
    let client = reqwest::Client::new();
    client
        .get(url)
        .header("X-API-Key", apikey)
        .send()
        .await?
        .json()
        .await
}

impl Resp {
    fn find_from_tx_hash(&self, from_addr: String) -> Option<String> {
        for tx in &self.result {
            if tx.from_address == from_addr {
                return Some(tx.hash.clone());
            }
        }
        None
    }
}
