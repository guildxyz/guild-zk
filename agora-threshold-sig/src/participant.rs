use crate::{FP_BYTES, G2_BYTES};
use bls::{G2Affine, Scalar};
use ff::Field;

#[derive(Clone, Debug)]
pub struct Participant {
    pub id: Scalar,
    pub pubkey: G2Affine,
}

impl Participant {
    pub fn to_bytes(&self) -> [u8; FP_BYTES + G2_BYTES] {
        let mut bytes = [0u8; FP_BYTES + G2_BYTES];
        bytes[0..FP_BYTES].copy_from_slice(&self.id.to_bytes());
        bytes[FP_BYTES..].copy_from_slice(&self.pubkey.to_compressed());
        bytes
    }
}
