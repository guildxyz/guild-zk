use crate::arithmetic::{Point, Scalar};
use crate::pedersen::PedersenGenerator;
use crate::{Curve, U256};

pub struct MembershipProof<CC, C, const SEC: usize> {
    cl: [Point<CC>; SEC],
    ca: [Point<CC>; SEC],
    cb: [Point<CC>; SEC],
    cd: [Point<CC>; SEC],
    f: [Scalar<CC>; SEC],
    za: [Scalar<CC>; SEC],
    zb: [Scalar<CC>; SEC],
    zd: Scalar<CC>,
    base_curve: PhantomData<C>,
}

impl<CC: Cycle<C>, C: Curve, const SEC: usize> MembershipProof<CC, C, SEC> {
    const HASH_ID: &'static [u8] = b"membership-proof";

    pub fn construct(
        pedersen_generator: &PedersenGenerator<CC>,
        commitment_to_key: &PedersenCommitment<CC>,
        index: usize,
        mut keys: Vec<U256>, // TODO figure out how to represent public keys
    ) -> Self {
        todo!();
    }
}
