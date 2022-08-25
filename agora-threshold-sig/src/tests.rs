use crate::encrypt::EncryptedShare;
use crate::hash::hash_to_fp;
use agora_interpolate::Polynomial;
use bls::{G2Affine, G2Projective, Scalar};
use ff::Field;
use rand_core::{CryptoRng, RngCore};

use std::collections::BTreeMap;

#[derive(Clone, Debug)]
pub struct Share {
    vk: G2Affine,
    esh: EncryptedShare,
}

pub type IdBytes = [u8; 32];

pub struct Node {
    nodes: usize,
    threshold: usize,
    id_bytes: IdBytes,
    public_key: G2Affine,
    private_key: Scalar,
    private_poly: Polynomial<Scalar>,
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
        if self.participants.len() < self.private_poly.coeffs().len() {
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

    // TODO recover keys
    // TODO reshare keys
    // TODO sign message
}
// methods?
//shsk: Scalar, // share signing key
//shvk: G2Affine, // share verification key

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

    assert!(true);
}
