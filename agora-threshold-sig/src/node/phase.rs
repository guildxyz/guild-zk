use super::shares_map::SharesMap;
use crate::keypair::Keypair;

use bls::{G2Affine, Scalar};

pub struct Discovery;

pub struct ShareGeneration {
    pub private_share: Option<Scalar>,
    pub shares_map: SharesMap,
}

pub struct ShareCollection {
    pub shares_map: SharesMap,
}

pub struct Finalized {
    pub share_keypair: Keypair,
    pub global_vk: G2Affine,
}
