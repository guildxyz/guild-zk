use super::balancy::{BalancyClient, ReqXyzHolders};
use super::config;
use super::db::DBClient;
use super::signer::{SignedResponse, Signer, VerifyParams};
use anyhow::Error;

pub struct Application {
    balancy_client: BalancyClient,
    db_client: DBClient,
    signer: Signer,
}

impl Application {
    pub fn new(conf: config::Settings) -> anyhow::Result<Application> {
        let signer = Signer::new(conf.private_key);
        let balancy_client = BalancyClient::new(
            conf.url_balancy,
            conf.apikey_balancy,
            conf.url_pubkey,
            conf.apikey_pubkey,
            180,
        );
        let db_client = DBClient::new(conf.db_uri)?;
        Ok(Application {
            balancy_client,
            db_client,
            signer,
        })
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

    pub async fn verify_signed_xyz_holders(&self, params: &VerifyParams) -> Result<bool, Error> {
        let guildid = params.get_guild_id();
        let rpoint = params.get_rpoint();
        let exists = self
            .db_client
            .check_rpoint_guildid(rpoint.as_str(), guildid.as_str())
            .await?;
        if exists {
            return Err(Error::msg("Rpoint already exists"));
        }
        let verified = self.signer.verify(params);
        if verified {
            self.db_client.set_rpoint_guildid(rpoint.as_str(), guildid.as_str()).await?;
            Ok(true)
        } else {
            return Err(Error::msg("Signature verification failed"));
        }
    }
}
