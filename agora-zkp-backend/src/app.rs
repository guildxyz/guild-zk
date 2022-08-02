use super::balancy::{BalancyClient, ReqXyzHolders};
use super::config;
use super::signer::{SignedResponse, Signer};
use anyhow::Error;

pub struct Application {
    balancy_client: BalancyClient,
    signer: Signer,
}

impl Application {
    pub fn new(conf: config::Settings) -> Application {
        let signer = Signer::new(conf.private_key);
        let balancy_client = BalancyClient::new(
            conf.url_balancy,
            conf.apikey_balancy,
            conf.url_pubkey,
            conf.apikey_pubkey,
            180,
        );
        Application {
            signer,
            balancy_client,
        }
    }

    pub async fn get_signed_xyz_holders_pubkeys(
        &self,
        req: ReqXyzHolders,
    ) -> Result<SignedResponse, Error> {
        let addresses = self.balancy_client.get_xyz_holders_addresses(req).await?;
        let pubkeys = self.balancy_client.get_pubkeys(addresses).await?;
        let signed = self.signer.sign_pubkeys(pubkeys);
        Ok(signed)
    }

    pub async fn verify_signed_xyz_holders(&self, signed: &SignedResponse) -> Result<bool, Error> {
        Ok(self.signer.verify(signed))
    }
}
