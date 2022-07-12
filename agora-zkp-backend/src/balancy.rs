#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ReqXyzHolders {
    pub logic: String,
    pub requirements: Vec<XyzHoldersRequirement>,
    pub limit: i32,
    pub offset: i32,
}

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct XyzHoldersRequirement {
    pub token_address: String,
    pub amount: String,
}

#[derive(serde::Deserialize, Debug)]
pub struct RespXyzHolders {
    pub addresses: Vec<String>,
    pub count: i32,
    pub limit: i32,
    pub offset: i32,
}

#[derive(serde::Serialize, Debug)]
pub struct ReqPubkey {
    pub addresses: Vec<String>,
}

#[derive(serde::Deserialize, Debug)]
pub struct RespPubkey {
    pub pubkeys: Vec<String>,
}

pub async fn get_xyz_holders_addresses(
    base_url: String,
    apikey: String,
    req_xyz_holders: ReqXyzHolders,
) -> Result<Vec<String>, reqwest::Error> {
    let url = format!("{}/xyzHolders", base_url);
    let client = reqwest::Client::new();
    let resp = client
        .post(url)
        .header("X-API-Key", apikey)
        .header("Content-Type", "application/json")
        .json(&req_xyz_holders)
        .send()
        .await?;
    let resp_body: RespXyzHolders = resp.json().await?;
    Ok(resp_body.addresses)
}

pub async fn get_pubkeys(
    base_url: &str,
    apikey: &str,
    addresses: Vec<String>,
) -> Result<Vec<String>, reqwest::Error> {
    let url = format!("{}/pubkey", base_url);
    let client = reqwest::Client::new();
    let req_body = ReqPubkey { addresses };
    let resp = client
        .post(url)
        .header("X-API-Key", apikey)
        .header("Content-Type", "application/json")
        .json(&req_body)
        .send()
        .await?;

    let resp_body: RespPubkey = resp.json().await?;
    Ok(resp_body.pubkeys)
}
