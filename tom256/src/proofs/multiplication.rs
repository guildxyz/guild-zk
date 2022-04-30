use crate::arithmetic::{Point, Scalar};
use crate::pedersen::*;
use crate::utils::hash_points;
use crate::Curve;

use rand_core::{CryptoRng, RngCore};

#[derive(Clone, Debug)]
pub struct MultiplicationProof<C: Curve> {
    // TODO
    something: Scalar<C>,
}

impl<C: Curve> MultiplicationProof<C> {
    const HASH_ID: &'static [u8] = b"multiplication-proof";

    pub fn construct<R: CryptoRng + RngCore>(
        rng: &mut R,
        pedersen_generator: &PedersenGenerator<C>,
        commitment_to_x: &PedersenCommitment<C>,
        commitment_to_y: &PedersenCommitment<C>,
        commitment_to_z: &PedersenCommitment<C>,
        x: Scalar<C>,
        y: Scalar<C>,
        z: Scalar<C>,
    ) -> Self {
        todo!()
    }

    pub fn verify<R: CryptoRng + RngCore>(
        &self,
        rng: &mut R,
        pedersen_generator: &PedersenGenerator<C>,
        commitment_to_x: &Point<C>,
        commitment_to_y: &Point<C>,
        commitment_to_z: &Point<C>,
    ) -> bool {
        todo!()
    }
}
