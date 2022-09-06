mod parameters;
mod phase;
#[cfg(test)]
mod test;

pub use parameters::Parameters;
pub use phase::{Discovery, Finalized, ShareCollection};

use crate::address::Address;
use crate::keypair::Keypair;
use crate::share::{EncryptedShare, PublicShare};
use crate::signature::Signature;

use agora_interpolate::Polynomial;
use bls::{G2Affine, G2Projective, Scalar};
use ff::Field;
use zeroize::Zeroize;

use std::collections::BTreeMap;

pub struct Node<P> {
    parameters: Parameters,
    address: Address,
    keypair: Keypair,
    phase: P,
}

impl<P> Node<P> {
    pub fn address(&self) -> Address {
        self.address
    }

    pub fn pubkey(&self) -> G2Affine {
        self.keypair.pubkey()
    }
}

impl Node<Discovery> {
    // Copy is required due to Scalar::random(r: impl RngCore) which will
    // reborrow &mut R as &mut *rng, meaning that rng is dereferenced and
    // thus moved if it's not Copy
    pub fn new(parameters: Parameters, keypair: Keypair) -> Node<Discovery> {
        let address = Address::from(&keypair.pubkey());
        let mut participants = BTreeMap::new();
        participants.insert(address, keypair.pubkey());

        Self {
            parameters,
            address,
            keypair,
            phase: Discovery { participants },
        }
    }

    pub fn collect_participant(&mut self, pubkey: G2Affine) {
        let address = Address::from(&pubkey);
        if self.phase.participants.get(&address).is_none() {
            self.phase.participants.insert(address, pubkey);
        }
    }
}

impl TryFrom<Node<Discovery>> for Node<ShareCollection> {
    type Error = String;
    fn try_from(node: Node<Discovery>) -> Result<Self, Self::Error> {
        if node.phase.participants.len() < node.parameters.nodes() {
            return Err("not enough participants collected".to_string());
        }

        // generate own share in this step
        // TODO private coeff_0 could be the private key of the node
        let private_coeffs = (0..node.parameters.threshold())
            .map(|_| Scalar::random(rand_core::OsRng))
            .collect::<Vec<Scalar>>();
        let mut private_poly = Polynomial::new(private_coeffs);
        let poly_secret = private_poly.coeffs()[0];

        let shares = node
            .phase
            .participants
            .iter()
            .map(|(address, pubkey)| {
                let secret_share = private_poly.evaluate(address.as_scalar());
                let public_share = G2Affine::from(G2Affine::generator() * secret_share);
                let esh = EncryptedShare::new(
                    &mut rand_core::OsRng,
                    address.as_bytes(),
                    pubkey,
                    &secret_share,
                );
                PublicShare {
                    vk: public_share,
                    esh,
                }
            })
            .collect::<Vec<PublicShare>>();
        private_poly.zeroize();
        let mut shares_map = BTreeMap::new();
        shares_map.insert(node.address, shares);

        Ok(Self {
            parameters: node.parameters,
            address: node.address,
            keypair: node.keypair,
            phase: ShareCollection {
                participants: node.phase.participants,
                shares: shares_map,
                poly_secret,
            },
        })
    }
}

impl Node<ShareCollection> {
    pub fn publish_share(&self) -> Vec<PublicShare> {
        // NOTE unwrap is fine because at this phase, we have
        // definitely generated our own share when converting
        // from Discovery phase
        self.phase.shares.get(&self.address).cloned().unwrap()
    }

    pub fn collect_share(
        &mut self,
        address: Address,
        shares: Vec<PublicShare>,
    ) -> Result<(), String> {
        if self.phase.participants.get(&address).is_none() {
            Err("no such participant registered".to_string())
        } else if self.phase.shares.get(&address).is_none() {
            self.phase.shares.insert(address, shares);
            Ok(())
        } else {
            Err("share already collected from this participant".to_string())
        }
    }

    fn verify_shares(&self) -> bool {
        for shares in self.phase.shares.values() {
            for (address, share) in self.phase.shares.keys().zip(shares) {
                if !share.esh.verify(address.as_bytes(), &share.vk) {
                    return false;
                }
            }
        }
        true
    }

    fn interpolated_shvks(&self, address_scalars: &[Scalar]) -> Result<Vec<G2Projective>, String> {
        let mut interpolated_shvks =
            Vec::<G2Projective>::with_capacity(self.phase.participants.len());

        for i in 0..self.phase.shares.len() {
            let shvks = self
                .phase
                .shares
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
            Vec::<Scalar>::with_capacity(self.phase.participants.len());
        for share_vec in self.phase.shares.values() {
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
            .shares
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
            phase: Finalized {
                participants: self.phase.participants,
                poly_secret: self.phase.poly_secret,
                share_keypair: Keypair::new_checked(
                    shsk,
                    interpolated_shvks[self_index].into(),
                )?,
                global_vk: gshvk_poly.coeffs()[0].into(),
            },
        })
    }
}

impl TryFrom<Node<ShareCollection>> for Node<Finalized> {
    type Error = String;
    fn try_from(node: Node<ShareCollection>) -> Result<Self, Self::Error> {
        if node.phase.shares.len() < node.parameters.nodes() {
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
}

// TODO reshare keys
