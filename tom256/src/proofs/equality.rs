use crate::arithmetic::Scalar;
use crate::pedersen::*;
use crate::Curve;

pub struct EqualityProof<C: Curve> {
    commitment_to_random_1: PedersenCommitment<C>,
    commitment_to_random_2: PedersenCommitment<C>,
    mask_secret: Scalar<C>,
    mask_random_1: Scalar<C>,
    mask_random_2: Scalar<C>,
}
