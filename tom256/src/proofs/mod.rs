mod equality;
mod exp;
mod membership;
mod multiplication;
mod point_add;
mod utils;

// TODO these does not need to be public
pub use exp::{ExpCommitmentPoints, ExpCommitments, ExpProof, ExpSecrets};
pub use membership::MembershipProof;

use crate::arithmetic::{Modular, Point, Scalar};
use crate::curve::{Curve, Cycle};
use crate::parse::{ParsedProofInput, ParsedRing};
use crate::pedersen::PedersenCycle;

use rand_core::{CryptoRng, RngCore};
use serde::{Deserialize, Serialize};

// NOTE 80 is conservative but slow, 40 is faster but quite low security
#[cfg(not(test))]
const SEC_PARAM: usize = 60;
#[cfg(test)]
const SEC_PARAM: usize = 10;

/// Zero-knowledge proof consisting of an ECDSA and a Groth-Kohlweiss
/// membership proof.
///
/// Note, that the ring on which the membership proof is generated is not
/// explicitly part of this proof because the backend does additional checks on
/// its integrity before passing it to the veriication function.
#[derive(Deserialize, Serialize)]
pub struct ZkAttestProof<C: Curve, CC: Cycle<C>> {
    pub pedersen: PedersenCycle<C, CC>,
    pub msg_hash: Scalar<C>,
    pub r_point: Point<C>,
    pub exp_commitments: ExpCommitmentPoints<C, CC>, // s1, pkx, pxy
    pub signature_proof: ExpProof<C, CC>,
    pub membership_proof: MembershipProof<CC>,
}

impl<C: Curve, CC: Cycle<C>> ZkAttestProof<C, CC> {
    pub fn construct<R: CryptoRng + RngCore>(
        rng: &mut R,
        pedersen: PedersenCycle<C, CC>,
        input: ParsedProofInput<C>,
        ring: &ParsedRing<CC>,
    ) -> Result<Self, String> {
        let s_inv = input.signature.s.inverse();
        let r_inv = input.signature.r.inverse();
        let u1 = s_inv * input.msg_hash;
        let u2 = s_inv * input.signature.r;
        let r_point = Point::<C>::GENERATOR.double_mul(&u1, &Point::from(&input.pubkey), &u2);
        let s1 = r_inv * input.signature.s;
        let z1 = r_inv * input.msg_hash;
        let q_point = &Point::<C>::GENERATOR * z1;

        let commitment_to_s1 = pedersen.base().commit_with_generator(rng, s1, &r_point);
        let commitment_to_pk_x = pedersen
            .cycle()
            .commit(rng, input.pubkey.x().to_cycle_scalar());
        let commitment_to_pk_y = pedersen
            .cycle()
            .commit(rng, input.pubkey.y().to_cycle_scalar());

        // generate membership proof on pubkey x coordinate
        let membership_proof = MembershipProof::construct(
            rng,
            pedersen.cycle(),
            &commitment_to_pk_x,
            input.index,
            ring,
        )?;

        // generate ECDSA proof on signature
        let exp_secrets = ExpSecrets::new(s1, input.pubkey);
        let exp_commitments = ExpCommitments {
            px: commitment_to_pk_x,
            py: commitment_to_pk_y,
            exp: commitment_to_s1,
        };

        let signature_proof = ExpProof::construct(
            rng,
            &r_point,
            &pedersen,
            &exp_secrets,
            &exp_commitments,
            SEC_PARAM,
            Some(q_point),
        )?;

        Ok(Self {
            pedersen,
            msg_hash: input.msg_hash,
            r_point,
            exp_commitments: exp_commitments.into_commitments(),
            signature_proof,
            membership_proof,
        })
    }

    pub fn verify<R: CryptoRng + RngCore>(
        &self,
        rng: &mut R,
        ring: &ParsedRing<CC>,
    ) -> Result<(), String> {
        let r_point_affine = self.r_point.to_affine();
        if r_point_affine.is_identity() {
            return Err("R is at infinity".to_string());
        }

        // NOTE weird: a field element Rx is converted
        // directly into a scalar
        let r_inv = Scalar::<C>::new(*r_point_affine.x().inner()).inverse();
        let z1 = r_inv * self.msg_hash;
        let q_point = &Point::<C>::GENERATOR * z1;

        self.membership_proof
            .verify(rng, self.pedersen.cycle(), &self.exp_commitments.px, ring)?;

        self.signature_proof.verify(
            rng,
            &self.r_point,
            &self.pedersen,
            &self.exp_commitments,
            SEC_PARAM,
            Some(q_point),
        )?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::ZkAttestProof;

    use crate::curve::{Secp256k1, Tom256k1};
    use crate::parse::{parse_ring, ParsedProofInput, ProofInput};
    use crate::pedersen::PedersenCycle;

    use rand::rngs::StdRng;
    use rand_core::SeedableRng;

    #[test]
    fn zkp_attest_valid() {
        let mut rng = StdRng::from_seed([14; 32]);
        let pedersen_cycle = PedersenCycle::<Secp256k1, Tom256k1>::new(&mut rng);

        let msg_hash =
            "0xb42062702a4acb9370edf5c571f2c7a6f448f8c42f3bfa59e622c1c064a94a14".to_string();
        let signature = "0xb2a7ff958cd78c8e896693b7b76550c8942d6499fb8cd621efb54909f9d51da02bfaadf918f09485740ba252445d40d44440fd810dbf8a9a18049157adcdaa8c1c".to_string();
        let pubkey = "0x0418a30afe39c280d2f43f05c070988dae7fbae9cdfd5fb6461acd7657e765e172fd55b3589c74fd4987b6004465afff77b039e631a68cdc7df9cd8cfd5cbe2887".to_string();

        let ring = vec![
            "ddd40afe39c280d2f43f05c070988dae7fbae9cdfd5fb6461acd7657e765e172".to_string(), // our pubkey x
            "ccc50afe39c280d2f43f05c070988dae7fbae9cdfd5fb6461acd7657e765e172".to_string(), // our pubkey x
            "18a30afe39c280d2f43f05c070988dae7fbae9cdfd5fb6461acd7657e765e172".to_string(), // our pubkey x
            "aaa70afe39c280d2f43f05c070988dae7fbae9cdfd5fb6461acd7657e765e172".to_string(), // our pubkey x
            "bbb80afe39c280d2f43f05c070988dae7fbae9cdfd5fb6461acd7657e765e172".to_string(), // our pubkey x
        ];

        let index = 2;

        let proof_input = ProofInput {
            msg_hash,
            pubkey,
            signature,
            index,
        };

        let parsed_input: ParsedProofInput<Secp256k1> = proof_input.try_into().unwrap();
        let parsed_ring = parse_ring(ring).unwrap();

        let zkattest_proof = ZkAttestProof::<Secp256k1, Tom256k1>::construct(
            &mut rng,
            pedersen_cycle,
            parsed_input,
            &parsed_ring,
        )
        .unwrap();
        assert!(zkattest_proof.verify(&mut rng, &parsed_ring).is_ok());
    }
}
