use crate::{FP_BYTES, G2_BYTES};
use bls::{G2Affine, Scalar};
use ff::Field;

#[derive(Clone, Debug)]
pub struct Participant {
    pub id: Scalar,
    pub pubkey: G2Affine,
}
