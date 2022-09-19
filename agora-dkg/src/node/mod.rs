mod parameters;
mod phase;
mod shares_map;
#[cfg(test)]
mod test;
mod utils;

pub use parameters::Parameters;
pub use phase::{Discovery, Finalized, ShareCollection, ShareGeneration};
pub use shares_map::SharesMap;

use crate::address::Address;
use crate::encryption::Encryption;
use crate::keypair::Keypair;
use crate::share::PublicShare;
use crate::signature::Signature;

use anyhow::ensure;
use bls::{G2Affine, G2Projective};
use zeroize::Zeroize;

use std::collections::BTreeMap;

pub struct Node<P> {
    parameters: Parameters,
    address: Address,
    keypair: Keypair,
    participants: BTreeMap<Address, G2Affine>,
    phase: P,
}

impl<P> Node<P> {
    pub fn address(&self) -> Address {
        self.address
    }

    pub fn pubkey(&self) -> G2Affine {
        self.keypair.pubkey()
    }

    pub fn collect_participant(&mut self, pubkey: G2Affine) {
        let address = Address::from(&pubkey);
        if self.participants.get(&address).is_none() {
            self.participants.insert(address, pubkey);
        }
    }
}

impl Node<Discovery> {
    pub fn new(parameters: Parameters, keypair: Keypair) -> Node<Discovery> {
        let address = Address::from(&keypair.pubkey());
        let mut participants = BTreeMap::new();
        participants.insert(address, keypair.pubkey());

        Self {
            parameters,
            address,
            keypair,
            participants,
            phase: Discovery,
        }
    }
}

impl TryFrom<Node<ShareGeneration>> for Node<ShareCollection> {
    type Error = anyhow::Error;
    fn try_from(node: Node<ShareGeneration>) -> Result<Self, Self::Error> {
        // generate own shares first
        let mut shares_map = node.phase.shares_map;
        let mut private_poly =
            utils::random_polynomial(node.parameters.threshold(), node.phase.private_share);
        let shares = utils::generate_shares(&node.participants, &private_poly);
        shares_map.insert(node.address, shares)?;
        private_poly.zeroize();
        Ok(Node {
            parameters: node.parameters,
            address: node.address,
            keypair: node.keypair,
            participants: node.participants,
            phase: ShareCollection { shares_map },
        })
    }
}

impl TryFrom<Node<Discovery>> for Node<ShareGeneration> {
    type Error = anyhow::Error;
    fn try_from(node: Node<Discovery>) -> Result<Self, Self::Error> {
        ensure!(
            node.participants.len() == node.parameters.nodes(),
            "not enough participants collected"
        );
        Ok(Self {
            parameters: node.parameters,
            address: node.address,
            keypair: node.keypair,
            participants: node.participants,
            phase: ShareGeneration {
                private_share: None,
                shares_map: SharesMap::new(node.parameters.nodes()),
            },
        })
    }
}

impl TryFrom<Node<Discovery>> for Node<ShareCollection> {
    type Error = anyhow::Error;
    fn try_from(node: Node<Discovery>) -> Result<Self, Self::Error> {
        ensure!(
            node.participants.len() == node.parameters.nodes(),
            "not enough participants collected"
        );
        Ok(Self {
            parameters: node.parameters,
            address: node.address,
            keypair: node.keypair,
            participants: node.participants,
            phase: ShareCollection {
                shares_map: SharesMap::new(node.parameters.nodes()),
            },
        })
    }
}

impl Node<ShareCollection> {
    pub fn publish_share(&self) -> Option<Vec<PublicShare>> {
        self.phase.shares_map.map().get(&self.address).cloned()
    }

    pub fn collect_share(
        &mut self,
        address: Address,
        shares: Vec<PublicShare>,
    ) -> Result<(), anyhow::Error> {
        ensure!(
            self.participants.get(&address).is_some(),
            "no such participant registered"
        );
        self.phase.shares_map.insert(address, shares)?;
        Ok(())
    }

    fn finalize(self) -> Result<Node<Finalized>, anyhow::Error> {
        let phase = self.phase.shares_map.recover_keys(
            &self.address,
            self.keypair.privkey(),
            &self.participants,
        )?;

        Ok(Node {
            parameters: self.parameters,
            address: self.address,
            keypair: self.keypair,
            participants: self.participants,
            phase,
        })
    }
}

impl TryFrom<Node<ShareCollection>> for Node<Finalized> {
    type Error = anyhow::Error;
    fn try_from(node: Node<ShareCollection>) -> Result<Self, Self::Error> {
        ensure!(
            node.phase.shares_map.map().len() >= node.parameters.threshold(),
            "not enough shares collected"
        );
        node.finalize()
    }
}

impl Node<Finalized> {
    pub fn global_verifying_key(&self) -> G2Affine {
        self.phase.global_vk
    }

    pub fn verifying_key(&self) -> G2Affine {
        self.phase.share_keypair.pubkey()
    }

    pub fn sign(&self, msg: &[u8]) -> Signature {
        self.phase.share_keypair.sign(msg)
    }

    pub fn decryption_share(&self, encryption: &Encryption) -> G2Projective {
        encryption.ephemeral_pubkey * self.phase.share_keypair.privkey()
    }

    pub fn initiate_resharing(
        self,
        parameters: Parameters,
    ) -> Result<Node<ShareGeneration>, String> {
        // TODO check parameters (or auto-generate them?)
        if parameters.nodes() != self.participants.len() {
            return Err("not enough participants collected".to_string());
        }

        Ok(Node {
            parameters,
            address: self.address,
            keypair: self.keypair,
            participants: self.participants,
            phase: ShareGeneration {
                private_share: Some(*self.phase.share_keypair.privkey()),
                shares_map: SharesMap::new(parameters.nodes()),
            },
        })
    }
}
