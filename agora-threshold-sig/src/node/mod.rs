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

impl Node<ShareGeneration> {
    pub fn initiate_share_collection(self) -> Node<ShareCollection> {
        // generate own shares first
        let mut sh_map = BTreeMap::new();
        let mut private_poly =
            utils::random_polynomial(self.parameters.threshold(), self.phase.private_share);
        let shares = utils::generate_shares(&self.participants, &private_poly);
        sh_map.insert(self.address, shares);
        private_poly.zeroize();
        Node {
            parameters: self.parameters,
            address: self.address,
            keypair: self.keypair,
            participants: self.participants,
            phase: ShareCollection {
                shares_map: SharesMap::new(sh_map),
            },
        }
    }
}

impl TryFrom<Node<Discovery>> for Node<ShareGeneration> {
    type Error = String;
    fn try_from(node: Node<Discovery>) -> Result<Self, Self::Error> {
        if node.participants.len() < node.parameters.nodes() {
            return Err("not enough participants collected".to_string());
        }
        Ok(Self {
            parameters: node.parameters,
            address: node.address,
            keypair: node.keypair,
            participants: node.participants,
            phase: ShareGeneration {
                private_share: None,
                shares_map: SharesMap::new(BTreeMap::new()),
            },
        })
    }
}

impl TryFrom<Node<Discovery>> for Node<ShareCollection> {
    type Error = String;
    fn try_from(node: Node<Discovery>) -> Result<Self, Self::Error> {
        if node.participants.len() < node.parameters.nodes() {
            return Err("not enough participants collected".to_string());
        }
        Ok(Self {
            parameters: node.parameters,
            address: node.address,
            keypair: node.keypair,
            participants: node.participants,
            phase: ShareCollection {
                shares_map: SharesMap::new(BTreeMap::new()),
            },
        })
    }
}

impl Node<ShareCollection> {
    pub fn publish_share(&self) -> Option<Vec<PublicShare>> {
        // NOTE unwrap is fine because at this point we definitely have
        // a share inserted in the map
        self.phase.shares_map.inner().get(&self.address).cloned()
    }

    pub fn collect_share(
        &mut self,
        address: Address,
        shares: Vec<PublicShare>,
    ) -> Result<(), String> {
        if self.participants.get(&address).is_none() {
            Err("no such participant registered".to_string())
        } else if self.phase.shares_map.inner().get(&address).is_none() {
            self.phase.shares_map.inner_mut().insert(address, shares);
            Ok(())
        } else {
            Err("share already collected from this participant".to_string())
        }
    }

    fn verify_shares(&self) -> bool {
        for shares in self.phase.shares_map.inner().values() {
            for (address, share) in self.participants.keys().zip(shares) {
                if !share.esh.verify(address.as_bytes(), &share.vk) {
                    return false;
                }
            }
        }
        true
    }

    fn finalize(self) -> Result<Node<Finalized>, String> {
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
    type Error = String;
    fn try_from(node: Node<ShareCollection>) -> Result<Self, Self::Error> {
        if node.phase.shares_map.inner().len() < node.parameters.threshold() {
            return Err("not enough shares collected".to_string());
        } else if !node.verify_shares() {
            return Err("invalid shares collected".to_string());
        }
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
                shares_map: SharesMap::new(BTreeMap::new()),
            },
        })
    }
}
