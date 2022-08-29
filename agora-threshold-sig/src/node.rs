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
pub type IdBytes = [u8; 32];

pub struct Node {
    nodes: usize,
    threshold: usize,
    id_bytes: IdBytes,
    public_key: G2Affine,
    private_key: Scalar,
    private_poly: Polynomial<Scalar>,
    shsk: Option<Scalar>,
    // own verification key
    shvk: Option<G2Affine>,
    // global verification key
    gshvk: Option<G2Affine>,
    shares: BTreeMap<IdBytes, Vec<Share>>,
    participants: BTreeMap<IdBytes, G2Affine>,
}

impl Node {
    // Copy is required due to Scalar::random(r: impl RngCore) which will
    // reborrow &mut R as &mut *rng, meaning that rng is dereferenced and
    // thus moved if it's not Copy
    pub fn new<R: RngCore + CryptoRng + Copy>(rng: R, nodes: usize, threshold: usize) -> Self {
        assert!(
            nodes >= threshold,
            "threshold is greater than the total number of participants"
        );
        let private_key = Scalar::random(rng);
        let public_key = G2Affine::from(G2Affine::generator() * private_key);
        let id_bytes = hash_to_fp(&public_key.to_compressed()).to_bytes();

        let mut private_coeffs = Vec::<Scalar>::with_capacity(threshold);
        let mut public_coeffs = Vec::<G2Projective>::with_capacity(threshold);
        for _ in 0..threshold {
            let private_coeff = Scalar::random(rng);
            private_coeffs.push(private_coeff);
            public_coeffs.push(G2Affine::generator() * private_coeff);
        }

        // TODO not necessarily needed
        let private_poly = Polynomial::new(private_coeffs);
        let mut participants = BTreeMap::new();
        participants.insert(id_bytes, public_key);

        Self {
            nodes,
            threshold,
            id_bytes,
            public_key,
            private_key,
            private_poly,
            shsk: None,
            shvk: None,
            gshvk: None,
            shares: BTreeMap::new(),
            participants,
        }
    }

    pub fn collect_participant(&mut self, id_bytes: IdBytes, pubkey: G2Affine) {
        if self.participants.get(&id_bytes).is_none() {
            self.participants.insert(id_bytes, pubkey);
        }
    }

    pub fn generate_share<R: RngCore + CryptoRng>(&mut self, rng: &mut R) -> Result<(), String> {
        if self.participants.len() < self.nodes {
            return Err("not enough participants collected".to_string());
        }

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
        self.shares.insert(self.id_bytes, shares);
        Ok(())
    }

    pub fn publish_share(&self) -> Option<Vec<Share>> {
        self.shares.get(&self.id_bytes).cloned()
    }

    pub fn collect_share(&mut self, id_bytes: IdBytes, shares: Vec<Share>) {
        if self.shares.get(&id_bytes).is_none() {
            self.shares.insert(id_bytes, shares);
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
            .map(|(i, id_bytes)| {
                if id_bytes == &self.id_bytes {
                    self_index = Some(i)
                }
                // NOTE unwrap is fine because all stored id_bytes
                // come from Scalars originally
                Scalar::from_bytes(id_bytes).unwrap()
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
                    .decrypt(&self.id_bytes, &self.private_key),
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
        node_0.collect_participant(node_1.id_bytes, node_1.public_key);
        node_0.collect_participant(node_2.id_bytes, node_2.public_key);
        node_1.collect_participant(node_0.id_bytes, node_0.public_key);
        node_1.collect_participant(node_2.id_bytes, node_2.public_key);
        node_2.collect_participant(node_0.id_bytes, node_0.public_key);
        node_2.collect_participant(node_1.id_bytes, node_1.public_key);
        // generate partial shares
        node_0.generate_share(&mut rng).unwrap();
        node_1.generate_share(&mut rng).unwrap();
        node_2.generate_share(&mut rng).unwrap();
        // publish and collect shares
        node_0.collect_share(node_1.id_bytes, node_1.publish_share().unwrap());
        node_0.collect_share(node_2.id_bytes, node_2.publish_share().unwrap());
        node_1.collect_share(node_0.id_bytes, node_0.publish_share().unwrap());
        node_1.collect_share(node_2.id_bytes, node_2.publish_share().unwrap());
        node_2.collect_share(node_0.id_bytes, node_0.publish_share().unwrap());
        node_2.collect_share(node_1.id_bytes, node_1.publish_share().unwrap());
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
