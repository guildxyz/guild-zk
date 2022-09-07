use crate::address::Address;
use crate::keypair::Keypair;
use crate::share::PublicShare;
use bls::G2Affine;
use std::collections::BTreeMap;

pub struct Discovery {
    pub participants: BTreeMap<Address, G2Affine>,
}

pub struct ShareCollection {
    pub participants: BTreeMap<Address, G2Affine>,
    pub shares: BTreeMap<Address, Vec<PublicShare>>,
}

pub struct Finalized {
    pub participants: BTreeMap<Address, G2Affine>,
    pub share_keypair: Keypair,
    pub global_vk: G2Affine,
}
