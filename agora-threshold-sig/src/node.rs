use crate::encrypt::EncryptedShare;
use crate::hash::hash_to_fp;
use agora_interpolate::Polynomial;
use bls::{G1Affine, G2Affine, G2Projective, Scalar};
use ff::Field;
use rand_core::{CryptoRng, RngCore};

use std::collections::BTreeMap;

// TODO do this in stages (like a builder)
// instead of having one big node with option fields
// 1) DiscoveryPhase
// 2) ShareCollectionPhase
// 3) RecoveryPhase -> Actual Node

#[derive(Clone, Debug)]
pub struct Share {
    vk: G2Affine,
    esh: EncryptedShare,
}

// Scalar does not implement `Ord` so
// it cannot be used directly in a
// BTreeMap
#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq)]
pub struct Address([u8; 32]);

/// From public key
impl From<G2Affine> for Address {
    fn from(pubkey: G2Affine) -> Self {
        Self::from(&pubkey)
    }
}

impl From<&G2Affine> for Address {
    fn from(pubkey: &G2Affine) -> Self {
        let address_bytes = hash_to_fp(&pubkey.to_compressed()).to_bytes();
        Self(address_bytes)
    }
}

impl Address {
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

pub struct Keypair {
    private: Scalar,
    public: G2Affine,
}

impl Keypair {
    pub fn new(private: Scalar) -> Self {
        Self {
            private,
            public: G2Affine::from(G2Affine::generator() * private),
        }
    }

    pub fn random<R: RngCore + CryptoRng>(rng: &mut R) -> Self {
        let private = Scalar::random(rng);
        Self::new(private)
    }

    pub fn pubkey(&self) -> &G2Affine {
        &self.public
    }

    pub fn sign(&self, msg: &[u8]) -> G1Affine {
        let msg_hash_g1 = crate::hash::hash_to_g1(msg);
        (msg_hash_g1 * self.private).into()
    }
}

pub struct Parameters {
    nodes: usize,
    threshold: usize,
}

impl Parameters {
    pub fn new(nodes: usize, threshold: usize) -> Self {
        assert!(
            nodes >= threshold,
            "threshold is greater than the total number of participants"
        );
        Self { nodes, threshold }
    }
}

pub struct Node<P> {
    parameters: Parameters,
    address: Address,
    keypair: Keypair,
    phase: P,
}

pub struct Discovery {
    participants: BTreeMap<Address, G2Affine>,
}

pub struct ShareCollection {
    participants: BTreeMap<Address, G2Affine>,
    shares: BTreeMap<Address, Vec<Share>>,
    poly_secret: Scalar,
}

pub struct Finalized {
    participants: BTreeMap<Address, G2Affine>,
    signature_shares: Vec<G1Affine>,
    share_keypair: Keypair,
    global_vk: G2Affine,
    poly_secret: Scalar,
}

impl Node<Discovery> {
    // Copy is required due to Scalar::random(r: impl RngCore) which will
    // reborrow &mut R as &mut *rng, meaning that rng is dereferenced and
    // thus moved if it's not Copy
    pub fn new(parameters: Parameters, keypair: Keypair) -> Node<Discovery> {
        let address = Address::from(&keypair.public);
        let mut participants = BTreeMap::new();
        participants.insert(address, keypair.public);

        Self {
            parameters,
            address,
            keypair,
            phase: Discovery { participants },
        }
    }

    pub fn pubkey(&self) -> G2Affine {
        self.keypair.public
    }

    pub fn collect_participant(&mut self, pubkey: G2Affine) {
        let address = Address::from(&pubkey);
        if self.phase.participants.get(&address).is_none() {
            self.phase.participants.insert(address, pubkey);
        }
    }
}

impl Node<ShareCollection> {
    pub fn new<R: CryptoRng + RngCore + Copy>(rng: R, node: Node<Discovery>) -> Result<Self, String> {
        if node.phase.participants.len() < node.parameters.nodes {
            return Err("not enough participants collected".to_string());
        }

        // generate own share in this step
        let private_coeffs = 
            (0..node.parameters.threshold).map(|_| Scalar::random(rng)).collect::<Vec<Scalar>>();
        let private_poly = Polynomial::new(private_coeffs);

        let shares = node.phase 
            .participants
            .iter()
            .map(|(address, pubkey)| {
                let secret_share = private_poly.evaluate(Scalar::from_bytes(address.as_bytes()).unwrap());
                let public_share = G2Affine::from(G2Affine::generator() * secret_share);
                let esh = todo!();//EncryptedShare::new(rng, address, pubkey, &secret_share);
                Share {
                    vk: public_share,
                    esh,
                }
            })
            .collect::<Vec<Share>>();
        let mut shares_map = BTreeMap::new();
        shares_map.insert(node.address, shares);

        Ok(Self {
            parameters: node.parameters,
            address: node.address,
            keypair: node.keypair,
            phase: ShareCollection {
                participants: node.phase.participants,
                shares: shares_map,
                poly_secret: private_poly.coeffs()[0],
            }
        })
    }
}



/*
impl TryFrom<Node<GenerateShare>>
    pub fn generate_share<R: RngCore + CryptoRng>(&mut self, rng: &mut R) -> Result<(), String> {

        let shares = self
            .participants
            .iter()
            .map(|(id, pubkey)| {
                let secret_share = self.private_poly.evaluate(Scalar::from_bytes(id).unwrap());
                let public_share = G2Affine::from(G2Affine::generator() * secret_share);
                let esh = EncryptedShare::new(rng, id, pubkey, &secret_share);
                Share {
                    vk: public_share,
                    esh,
                }
            })
            .collect::<Vec<Share>>();
        self.shares.insert(self.address, shares);
        Ok(())
    }

    pub fn publish_share(&self) -> Option<Vec<Share>> {
        self.shares.get(&self.address).cloned()
    }

    pub fn collect_share(&mut self, address: Address, shares: Vec<Share>) {
        if self.shares.get(&address).is_none() {
            self.shares.insert(address, shares);
        }
    }

    pub fn verify_shares(&self) -> bool {
        for shares in self.shares.values() {
            for (id, share) in self.shares.keys().zip(shares) {
                if !share.esh.verify(id, &share.vk) {
                    return false;
                }
            }
        }
        true
    }

    pub fn recover_keys(&mut self) -> Result<(), String> {
        if self.shares.len() < self.nodes {
            return Err("not enough shares collected".to_string());
        } else if self.shsk.is_some() || self.shvk.is_some() || self.gshvk.is_some() {
            return Err("share signing key already recovered".to_string());
        }

        let mut self_index = None;
        let id_scalars = self
            .shares
            .keys()
            .enumerate()
            .map(|(i, address)| {
                if address == &self.address {
                    self_index = Some(i)
                }
                // NOTE unwrap is fine because all stored address
                // come from Scalars originally
                Scalar::from_bytes(address).unwrap()
            })
            .collect::<Vec<Scalar>>();

        // NOTE unwrap is fine because self_index is already in
        // the storage at this point
        let self_index = self_index.unwrap();
        let decrypted_shsks = self.decrypted_shsks(self_index);
        let interpolated_shvks = self.interpolated_shvks(&id_scalars)?;

        let shsk_poly =
            Polynomial::interpolate(&id_scalars, &decrypted_shsks).map_err(|e| e.to_string())?;
        let gshvk_poly =
            Polynomial::interpolate(&id_scalars, &interpolated_shvks).map_err(|e| e.to_string())?;
        self.shsk = Some(shsk_poly.coeffs()[0]);
        self.shvk = Some(interpolated_shvks[self_index].into());
        self.gshvk = Some(gshvk_poly.coeffs()[0].into());
        Ok(())
    }

    fn interpolated_shvks(&self, id_scalars: &[Scalar]) -> Result<Vec<G2Projective>, String> {
        let mut interpolated_shvks = Vec::<G2Projective>::with_capacity(self.participants.len());

        for i in 0..self.shares.len() {
            let shvks = self
                .shares
                .values()
                .map(|vec| vec[i].vk.into())
                .collect::<Vec<G2Projective>>();
            let poly = Polynomial::interpolate(&id_scalars, &shvks).map_err(|e| e.to_string())?;
            interpolated_shvks.push(poly.coeffs()[0]);
        }

        Ok(interpolated_shvks)
    }

    fn decrypted_shsks(&self, self_index: usize) -> Vec<Scalar> {
        let mut decrypted_shares_for_self = Vec::<Scalar>::with_capacity(self.participants.len());
        // NOTE unwrap is fine because...
        for share_vec in self.shares.values() {
            decrypted_shares_for_self.push(
                share_vec[self_index]
                    .esh
                    .decrypt(&self.address, &self.private_key),
            );
        }
        decrypted_shares_for_self
    }

    pub fn sign(&self, msg: &[u8]) -> G1Affine {
        let msg_hash_g1 = crate::hash::hash_to_g1(msg);
        // TODO unwrap -> fix with phases
        (msg_hash_g1 * self.shsk.unwrap()).into()
    }

    // TODO reshare keys
    // TODO sign message
}

pub fn sig_verify(msg: &[u8], vk: &G2Affine, sig: &G1Affine) -> bool {
    let msg_hash_g1 = crate::hash::hash_to_g1(msg);
    bls::pairing(&msg_hash_g1, vk) == bls::pairing(sig, &G2Affine::generator())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn dkg_23() {
        let mut rng = rand_core::OsRng;
        let nodes = 3_usize;
        let threshold = 2_usize;
        // spin up nodes
        let mut node_0 = Node::new(rng, nodes, threshold);
        let mut node_1 = Node::new(rng, nodes, threshold);
        let mut node_2 = Node::new(rng, nodes, threshold);
        // collect participants
        node_0.collect_participant(node_1.address, node_1.public_key);
        node_0.collect_participant(node_2.address, node_2.public_key);
        node_1.collect_participant(node_0.address, node_0.public_key);
        node_1.collect_participant(node_2.address, node_2.public_key);
        node_2.collect_participant(node_0.address, node_0.public_key);
        node_2.collect_participant(node_1.address, node_1.public_key);
        // generate partial shares
        node_0.generate_share(&mut rng).unwrap();
        node_1.generate_share(&mut rng).unwrap();
        node_2.generate_share(&mut rng).unwrap();
        // publish and collect shares
        node_0.collect_share(node_1.address, node_1.publish_share().unwrap());
        node_0.collect_share(node_2.address, node_2.publish_share().unwrap());
        node_1.collect_share(node_0.address, node_0.publish_share().unwrap());
        node_1.collect_share(node_2.address, node_2.publish_share().unwrap());
        node_2.collect_share(node_0.address, node_0.publish_share().unwrap());
        node_2.collect_share(node_1.address, node_1.publish_share().unwrap());
        assert_eq!(node_0.participants.len(), nodes);
        assert_eq!(node_1.participants.len(), nodes);
        assert_eq!(node_2.participants.len(), nodes);
        assert_eq!(node_0.shares.len(), nodes);
        assert_eq!(node_1.shares.len(), nodes);
        assert_eq!(node_2.shares.len(), nodes);
        // verify collected shares
        assert!(node_0.verify_shares());
        assert!(node_1.verify_shares());
        assert!(node_2.verify_shares());
        // recover signing and verification keys
        node_0.recover_keys().unwrap();
        node_1.recover_keys().unwrap();
        node_2.recover_keys().unwrap();
        assert_eq!(node_0.gshvk, node_1.gshvk);
        assert_eq!(node_1.gshvk, node_2.gshvk);
        // assert that keypairs belong together
        assert_eq!(
            node_0.shvk.unwrap(),
            G2Affine::from(G2Affine::generator() * node_0.shsk.unwrap())
        );
        assert_eq!(
            node_1.shvk.unwrap(),
            G2Affine::from(G2Affine::generator() * node_1.shsk.unwrap())
        );
        assert_eq!(
            node_2.shvk.unwrap(),
            G2Affine::from(G2Affine::generator() * node_2.shsk.unwrap())
        );
    }
}
*/
