mod equality;
mod exp;
mod membership;
mod multiplication;
mod point_add;
mod utils;

pub use exp::{ExpProof, PointExpSecrets};
pub use membership::MembershipProof;

use crate::arithmetic::{Modular, Point, Scalar};
use crate::curve::{Curve, Cycle};
use crate::parse::ParsedProofInput;
use crate::pedersen::{PedersenCommitment, PedersenCycle};

use rand_core::{CryptoRng, RngCore};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct ZkAttestProof<C: Curve, CC: Cycle<C>> {
    pub pedersen: PedersenCycle<C, CC>,
    pub msg_hash: Scalar<C>,
    pub r_point: Point<C>,
    pub commitment_to_s1: Point<C>,
    pub commitment_to_address: Point<CC>,
    pub commitment_to_pk_x: Point<CC>,
    pub commitment_to_pk_y: Point<CC>,
    pub signature_proof: ExpProof<C, CC>,
    pub membership_proof: MembershipProof<CC>,
    pub ring: Vec<Scalar<CC>>,
}

impl<C: Curve, CC: Cycle<C>> ZkAttestProof<C, CC> {
    pub fn construct<R: CryptoRng + RngCore>(
        rng: &mut R,
        pedersen: PedersenCycle<C, CC>,
        commitment_to_address: PedersenCommitment<CC>,
        input: ParsedProofInput<C, CC>,
    ) -> Result<Self, String> {
        let s_inv = input.signature.s.inverse();
        let r_inv = input.signature.r.inverse();
        let u1 = s_inv * input.msg_hash;
        let u2 = s_inv * input.signature.r;
        let r_point = Point::<C>::GENERATOR.double_mul(&u1, &input.pubkey, &u2);
        let s1 = r_inv * input.signature.s;
        let z1 = r_inv * input.msg_hash;
        let q_point = &Point::<C>::GENERATOR * z1;

        let commitment_to_s1 = pedersen.base().commit(rng, s1);
        let commitment_to_pk_x = pedersen
            .cycle()
            .commit(rng, input.pubkey.x().to_cycle_scalar());
        let commitment_to_pk_y = pedersen
            .cycle()
            .commit(rng, input.pubkey.y().to_cycle_scalar());

        let exp_secrets = PointExpSecrets::new(s1, input.pubkey);
        let exp_commitments = exp_secrets.commit(rng, &pedersen, Some(q_point));

        let signature_proof =
            ExpProof::construct(rng, &r_point, &pedersen, &exp_secrets, &exp_commitments, 60)?;
        let membership_proof = MembershipProof::construct(
            rng,
            pedersen.cycle(),
            &commitment_to_address,
            input.index,
            &input.ring,
        )?;

        Ok(Self {
            pedersen,
            msg_hash: input.msg_hash,
            r_point,
            commitment_to_s1: commitment_to_s1.into_commitment(),
            commitment_to_address: commitment_to_address.into_commitment(),
            commitment_to_pk_x: commitment_to_pk_x.into_commitment(),
            commitment_to_pk_y: commitment_to_pk_y.into_commitment(),
            signature_proof,
            membership_proof,
            ring: input.ring,
        })
    }
}
