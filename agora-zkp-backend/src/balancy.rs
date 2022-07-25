#[derive(serde::Serialize, serde::Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ReqXyzHolders {
    pub logic: String,
    pub requirements: Vec<XyzHoldersRequirement>,
    pub limit: i32,
    pub offset: i32,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
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

pub struct BalancyClient {
    http_client: reqwest::Client,
    url_balancy: String,
    apikey_balancy: String,
    url_pubkey: String,
    apikey_pubkey: String,
}

impl BalancyClient {
    pub fn new(
        url_balancy: String,
        apikey_balancy: String,
        url_pubkey: String,
        apikey_pubkey: String,
        timeout_sec: u64,
    ) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(timeout_sec))
            .build()
            .expect("Failed to build http_client");
        BalancyClient {
            url_balancy,
            apikey_balancy,
            url_pubkey,
            apikey_pubkey,
            http_client,
        }
    }

    pub async fn get_xyz_holders_addresses(
        &self,
        req_xyz_holders: ReqXyzHolders,
    ) -> Result<Vec<String>, reqwest::Error> {
        let url = format!("{}/xyzHolders", self.url_balancy);
        let resp = self
            .http_client
            .post(url)
            .header("X-API-Key", &self.apikey_balancy)
            .header("Content-Type", "application/json")
            .json(&req_xyz_holders)
            .send()
            .await?
            .error_for_status()?;
        let resp_body: RespXyzHolders = resp.json().await?;
        Ok(resp_body.addresses)
    }

    pub async fn get_pubkeys(&self, addresses: Vec<String>) -> Result<Vec<String>, reqwest::Error> {
        let url = format!("{}/pubkey", self.url_pubkey);
        let req_body = ReqPubkey { addresses };
        let resp = self
            .http_client
            .post(url)
            .header("X-API-Key", &self.apikey_pubkey)
            .header("Content-Type", "application/json")
            .json(&req_body)
            .send()
            .await?
            .error_for_status()?;

        let resp_body: RespPubkey = resp.json().await?;
        Ok(resp_body.pubkeys)
    }
}
