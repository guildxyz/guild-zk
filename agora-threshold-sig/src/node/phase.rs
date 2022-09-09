use crate::address::Address;
use crate::keypair::Keypair;
use crate::share::PublicShare;
use bls::{G2Affine, Scalar};
use std::collections::BTreeMap;

pub struct Discovery;

pub struct ShareGeneration {
    pub private_share: Option<Scalar>,
    pub shares_map: BTreeMap<Address, Option<Vec<PublicShare>>>,
}

pub struct ShareCollection {
    pub shares_map: BTreeMap<Address, Option<Vec<PublicShare>>>,
}

pub struct Finalized {
    pub share_keypair: Keypair,
    pub global_vk: G2Affine,
}
