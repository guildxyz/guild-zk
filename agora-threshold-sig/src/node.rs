use crate::encrypt::EncryptedShare;
use crate::hash::hash_to_fp;
use agora_interpolate::Polynomial;
use bls::{G1Affine, G2Affine, G2Projective, Scalar};
use ff::Field;
use rand_core::{CryptoRng, RngCore};

use std::collections::BTreeMap;

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

    pub fn to_scalar(&self) -> Scalar {
        // NOTE unwrap is fine because a valid address
        // can only be created from a Scalar type via
        // hash_to_fp
        Scalar::from_bytes(&self.0).unwrap()
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

    pub fn new_checked(private: Scalar, public: G2Affine) -> Result<Self, String> {
        if public == (G2Affine::generator() * private).into() {
            Ok(Self { private, public })
        } else {
            Err("invalid keypair".to_string())
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

#[derive(Debug, Clone, Copy)]
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
    poly_secret: Scalar,
    share_keypair: Keypair,
    global_vk: G2Affine,
}

impl<P> Node<P> {
    pub fn address(&self) -> Address {
        self.address
    }

    pub fn pubkey(&self) -> G2Affine {
        self.keypair.public
    }
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
        if node.phase.participants.len() < node.parameters.nodes {
            return Err("not enough participants collected".to_string());
        }

        // generate own share in this step
        let private_coeffs = (0..node.parameters.threshold)
            .map(|_| Scalar::random(rand_core::OsRng))
            .collect::<Vec<Scalar>>();
        let private_poly = Polynomial::new(private_coeffs);

        let shares = node
            .phase
            .participants
            .iter()
            .map(|(address, pubkey)| {
                let secret_share = private_poly.evaluate(address.to_scalar());
                let public_share = G2Affine::from(G2Affine::generator() * secret_share);
                let esh = EncryptedShare::new(
                    &mut rand_core::OsRng,
                    address.as_bytes(),
                    pubkey,
                    &secret_share,
                );
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
            },
        })
    }
}

impl Node<ShareCollection> {
    pub fn publish_share(&self) -> Vec<Share> {
        // NOTE unwrap is fine because at this phase, we have
        // definitely generated our own share when converting
        // from Discovery phase
        self.phase.shares.get(&self.address).cloned().unwrap()
    }

    pub fn collect_share(&mut self, address: Address, shares: Vec<Share>) {
        if self.phase.shares.get(&address).is_none() {
            self.phase.shares.insert(address, shares);
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
                Polynomial::interpolate(&address_scalars, &shvks).map_err(|e| e.to_string())?;
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
                    .decrypt(self.address.as_bytes(), &self.keypair.private),
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
                address.to_scalar()
            })
            .collect::<Vec<Scalar>>();

        let self_index = self_index.ok_or("self index not found in storage".to_string())?;

        let decrypted_shsks = self.decrypted_shsks(self_index);
        let interpolated_shvks = self.interpolated_shvks(&id_scalars)?;

        let shsk_poly =
            Polynomial::interpolate(&id_scalars, &decrypted_shsks).map_err(|e| e.to_string())?;
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
                    shsk_poly.coeffs()[0],
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
        if node.phase.shares.len() < node.parameters.nodes {
            return Err("not enough shares collected".to_string());
        } else if !node.verify_shares() {
            return Err("invalid shares collected".to_string());
        }
        node.recover_keys()
    }
}

impl Node<Finalized> {
    pub fn verifying_key(&self) -> G2Affine {
        self.phase.share_keypair.public
    }

    pub fn partial_sign(&self, msg: &[u8]) -> G1Affine {
        self.phase.share_keypair.sign(msg)
    }
}

// TODO reshare keys

pub fn partial_sig_verify(msg: &[u8], vk: &G2Affine, sig: &G1Affine) -> bool {
    let msg_hash_g1 = crate::hash::hash_to_g1(msg);
    bls::pairing(&msg_hash_g1, vk) == bls::pairing(sig, &G2Affine::generator())
}

pub fn global_sig_verify() -> bool {
    todo!();
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn dkg_23() {
        let mut rng = rand_core::OsRng;
        let parameters = Parameters::new(3, 2);
        // spin up nodes
        let mut node_0 = Node::<Discovery>::new(parameters, Keypair::random(&mut rng));
        let mut node_1 = Node::<Discovery>::new(parameters, Keypair::random(&mut rng));
        let mut node_2 = Node::<Discovery>::new(parameters, Keypair::random(&mut rng));
        // collect participants
        node_0.collect_participant(node_1.pubkey());
        node_0.collect_participant(node_2.pubkey());
        node_1.collect_participant(node_0.pubkey());
        node_1.collect_participant(node_2.pubkey());
        node_2.collect_participant(node_0.pubkey());
        node_2.collect_participant(node_1.pubkey());
        // generate partial shares
        let mut node_0 = Node::<ShareCollection>::try_from(node_0).unwrap();
        let mut node_1 = Node::<ShareCollection>::try_from(node_1).unwrap();
        let mut node_2 = Node::<ShareCollection>::try_from(node_2).unwrap();
        // publish and collect shares
        node_0.collect_share(node_1.address(), node_1.publish_share());
        node_0.collect_share(node_2.address(), node_2.publish_share());
        node_1.collect_share(node_0.address(), node_0.publish_share());
        node_1.collect_share(node_2.address(), node_2.publish_share());
        node_2.collect_share(node_0.address(), node_0.publish_share());
        node_2.collect_share(node_1.address(), node_1.publish_share());
        assert_eq!(node_0.phase.participants.len(), parameters.nodes);
        assert_eq!(node_1.phase.participants.len(), parameters.nodes);
        assert_eq!(node_2.phase.participants.len(), parameters.nodes);
        assert_eq!(node_0.phase.shares.len(), parameters.nodes);
        assert_eq!(node_1.phase.shares.len(), parameters.nodes);
        assert_eq!(node_2.phase.shares.len(), parameters.nodes);
        // verify collected shares
        let node_0 = Node::<Finalized>::try_from(node_0).unwrap();
        let node_1 = Node::<Finalized>::try_from(node_1).unwrap();
        let node_2 = Node::<Finalized>::try_from(node_2).unwrap();
        assert_eq!(node_0.phase.global_vk, node_1.phase.global_vk);
        assert_eq!(node_1.phase.global_vk, node_2.phase.global_vk);
        // sign message
        let msg = b"hello world";
        let signatures = vec![
            node_0.partial_sign(msg),
            node_1.partial_sign(msg),
            node_2.partial_sign(msg),
        ];
        assert!(partial_sig_verify(
            msg,
            &node_0.verifying_key(),
            &signatures[0]
        ));
        assert!(partial_sig_verify(
            msg,
            &node_1.verifying_key(),
            &signatures[1]
        ));
        assert!(partial_sig_verify(
            msg,
            &node_2.verifying_key(),
            &signatures[2]
        ));
    }
}
