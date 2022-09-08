mod parameters;
mod phase;
#[cfg(test)]
mod test;
mod utils;

pub use parameters::Parameters;
pub use phase::{Discovery, Finalized, ShareCollection};

use crate::address::Address;
use crate::keypair::Keypair;
use crate::share::PublicShare;
use crate::signature::Signature;

use agora_interpolate::Polynomial;
use bls::{G2Affine, G2Projective, Scalar};
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

    fn generate_shares(&self, shares_map: &mut BTreeMap<Address, Vec<PublicShare>>) {
        let mut private_poly = utils::random_polynomial(self.parameters.threshold(), None);
        let shares = utils::generate_shares(&self.participants, &private_poly);
        shares_map.insert(self.address, shares);
        private_poly.zeroize();
    }
}

impl TryFrom<Node<Discovery>> for Node<ShareCollection> {
    type Error = String;
    fn try_from(mut node: Node<Discovery>) -> Result<Self, Self::Error> {
        // TODO do we need this check when resharing
        if node.participants.len() < node.parameters.nodes() {
            return Err("not enough participants collected".to_string());
        }

        let mut shares_map = BTreeMap::new();
        // generate own shares initially, but not when resharing
        if node.parameters.is_original_member() {
            node.generate_shares(&mut shares_map);
        }
        // switch the original member flag
        // because it will be an original member
        // in the next resharing phase
        node.parameters.set_original_member();

        Ok(Self {
            parameters: node.parameters,
            address: node.address,
            keypair: node.keypair,
            participants: node.participants,
            phase: ShareCollection { shares_map },
        })
    }
}

impl Node<ShareCollection> {
    pub fn publish_share(&self) -> Option<Vec<PublicShare>> {
        self.phase.shares_map.get(&self.address).cloned()
    }

    pub fn collect_share(
        &mut self,
        address: Address,
        shares: Vec<PublicShare>,
    ) -> Result<(), String> {
        if self.participants.get(&address).is_none() {
            Err("no such participant registered".to_string())
        } else if self.phase.shares_map.get(&address).is_none() {
            self.phase.shares_map.insert(address, shares);
            Ok(())
        } else {
            Err("share already collected from this participant".to_string())
        }
    }

    fn verify_shares(&self) -> bool {
        for shares in self.phase.shares_map.values() {
            for (address, share) in self.phase.shares_map.keys().zip(shares) {
                if !share.esh.verify(address.as_bytes(), &share.vk) {
                    return false;
                }
            }
        }
        true
    }

    fn interpolated_shvks(&self, address_scalars: &[Scalar]) -> Result<Vec<G2Projective>, String> {
        let mut interpolated_shvks =
            Vec::<G2Projective>::with_capacity(self.participants.len());

        for i in 0..self.phase.shares_map.len() {
            let shvks = self
                .phase
                .shares_map
                .values()
                .map(|vec| vec[i].vk.into())
                .collect::<Vec<G2Projective>>();
            let poly =
                Polynomial::interpolate(address_scalars, &shvks).map_err(|e| e.to_string())?;
            interpolated_shvks.push(poly.coeffs()[0]);
        }

        Ok(interpolated_shvks)
    }

    fn decrypted_shsks(&self, self_index: usize) -> Vec<Scalar> {
        let mut decrypted_shares_for_self =
            Vec::<Scalar>::with_capacity(self.participants.len());
        for share_vec in self.phase.shares_map.values() {
            decrypted_shares_for_self.push(
                share_vec[self_index]
                    .esh
                    .decrypt(self.address.as_bytes(), self.keypair.privkey()),
            );
        }
        decrypted_shares_for_self
    }

    fn recover_keys(self) -> Result<Node<Finalized>, String> {
        let mut self_index = None;
        let id_scalars = self
            .phase
            .shares_map
            .keys()
            .enumerate()
            .map(|(i, address)| {
                if address == &self.address {
                    self_index = Some(i)
                }
                address.as_scalar()
            })
            .collect::<Vec<Scalar>>();

        let self_index = self_index.ok_or_else(|| "self index not found in storage".to_string())?;

        let mut decrypted_shsks = self.decrypted_shsks(self_index);
        let interpolated_shvks = self.interpolated_shvks(&id_scalars)?;

        let mut shsk_poly =
            Polynomial::interpolate(&id_scalars, &decrypted_shsks).map_err(|e| e.to_string())?;
        let shsk = shsk_poly.coeffs()[0];
        shsk_poly.zeroize();
        decrypted_shsks.zeroize();

        let gshvk_poly =
            Polynomial::interpolate(&id_scalars, &interpolated_shvks).map_err(|e| e.to_string())?;

        Ok(Node {
            parameters: self.parameters,
            address: self.address,
            keypair: self.keypair,
            participants: self.participants,
            phase: Finalized {
                share_keypair: Keypair::new_checked(shsk, interpolated_shvks[self_index].into())?,
                global_vk: gshvk_poly.coeffs()[0].into(),
            },
        })
    }
}

impl TryFrom<Node<ShareCollection>> for Node<Finalized> {
    type Error = String;
    fn try_from(node: Node<ShareCollection>) -> Result<Self, Self::Error> {
        if node.phase.shares_map.len() < node.parameters.nodes() {
            return Err("not enough shares collected".to_string());
        } else if !node.verify_shares() {
            return Err("invalid shares collected".to_string());
        }
        node.recover_keys()
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

    pub fn initiate_resharing(
        self,
        parameters: Parameters,
    ) -> Node<ShareCollection> {
        // TODO check parameters (or auto-generate them?)
        let mut shares_map = BTreeMap::new();
        let mut private_poly = utils::random_polynomial(
            self.parameters.threshold(),
            Some(*self.phase.share_keypair.privkey()),
        );
        let shares = utils::generate_shares(&self.participants, &private_poly);
        shares_map.insert(self.address, shares);
        private_poly.zeroize();
        Node {
            parameters,
            address: self.address,
            keypair: self.keypair,
            participants: self.participants,
            phase: ShareCollection {
                shares_map,
            },
        }
    }
    // TODO collect new participants
    // TODO initiate reshare process
}
