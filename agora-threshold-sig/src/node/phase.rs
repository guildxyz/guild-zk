use crate::address::Address;
use crate::keypair::Keypair;
use crate::share::PublicShare;
use bls::{G2Affine, Scalar};
use std::collections::BTreeMap;

pub struct Discovery {
    pub participants: BTreeMap<Address, G2Affine>,
}

pub struct ShareCollection {
    pub participants: BTreeMap<Address, G2Affine>,
    pub shares: BTreeMap<Address, Vec<PublicShare>>,
    pub poly_secret: Scalar,
}

pub struct Finalized {
    pub participants: BTreeMap<Address, G2Affine>,
    pub poly_secret: Scalar, // keep this for re-sharing?
    pub share_keypair: Keypair,
    pub global_vk: G2Affine,
}
