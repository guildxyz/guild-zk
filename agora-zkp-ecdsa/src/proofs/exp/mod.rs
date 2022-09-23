mod aux;

use crate::arithmetic::multimult::MultiMult;
use crate::arithmetic::{AffinePoint, Point, Scalar};
use crate::curve::{Curve, Cycle};
use crate::hasher::PointHasher;
use crate::pedersen::*;
use crate::proofs::point_add::{PointAddCommitmentPoints, PointAddProof, PointAddSecrets};
use crate::rng::CryptoCoreRng;

use bigint::{Encoding, U256};
use borsh::{BorshDeserialize, BorshSerialize};

use std::ops::Neg;
use std::sync::{Arc, Mutex};

#[allow(clippy::large_enum_variant)]
#[derive(BorshDeserialize, BorshSerialize)]
pub enum ExpProofVariant<C: Curve, CC: Cycle<C>> {
    Odd {
        alpha: Scalar<C>,
        r: Scalar<C>,
        tx_r: Scalar<CC>,
        ty_r: Scalar<CC>,
    },
    Even {
        z: Scalar<C>,
        r: Scalar<C>,
        t1_x: Scalar<CC>,
        t1_y: Scalar<CC>,
        add_proof: PointAddProof<CC, C>,
    },
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct SingleExpProof<C: Curve, CC: Cycle<C>> {
    a: Point<C>,
    tx_p: Point<CC>,
    ty_p: Point<CC>,
    variant: ExpProofVariant<C, CC>,
}

#[derive(Clone)]
pub struct ExpSecrets<C: Curve> {
    point: AffinePoint<C>,
    exp: Scalar<C>,
}

#[derive(Clone)]
pub struct ExpCommitments<C: Curve, CC: Cycle<C>> {
    pub(super) px: PedersenCommitment<CC>,
    pub(super) py: PedersenCommitment<CC>,
    pub(super) exp: PedersenCommitment<C>,
}

#[derive(Clone, BorshDeserialize, BorshSerialize)]
pub struct ExpCommitmentPoints<C: Curve, CC: Cycle<C>> {
    pub(super) px: Point<CC>,
    pub(super) py: Point<CC>,
    pub(super) exp: Point<C>,
}

impl<C: Curve> ExpSecrets<C> {
    pub fn new(exp: Scalar<C>, point: AffinePoint<C>) -> Self {
        Self { exp, point }
    }

    pub fn commit<R, CC>(
        &self,
        rng: &mut R,
        pedersen: &PedersenCycle<C, CC>,
    ) -> ExpCommitments<C, CC>
    where
        R: CryptoCoreRng,
        CC: Cycle<C>,
    {
        ExpCommitments {
            px: pedersen
                .cycle()
                .commit(rng, self.point.x().to_cycle_scalar()),
            py: pedersen
                .cycle()
                .commit(rng, self.point.y().to_cycle_scalar()),
            exp: pedersen.base().commit(rng, self.exp),
        }
    }
}

impl<C: Curve, CC: Cycle<C>> ExpCommitments<C, CC> {
    pub fn into_commitments(self) -> ExpCommitmentPoints<C, CC> {
        ExpCommitmentPoints {
            px: self.px.into_commitment(),
            py: self.py.into_commitment(),
            exp: self.exp.into_commitment(),
        }
    }
}

impl<C: Curve, CC: Cycle<C>> ExpCommitmentPoints<C, CC> {
    pub fn new(exp: Point<C>, px: Point<CC>, py: Point<CC>) -> Self {
        Self { exp, px, py }
    }
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct ExpProof<C: Curve, CC: Cycle<C>> {
    proofs: Vec<SingleExpProof<C, CC>>,
}

impl<CC: Cycle<C>, C: Curve> ExpProof<C, CC> {
    const HASH_ID: &'static [u8] = b"exp-proof";

    pub fn construct(
        //rng: &mut R,
        base_gen: &Point<C>,
        pedersen: &PedersenCycle<C, CC>,
        secrets: &ExpSecrets<C>,
        commitments: &ExpCommitments<C, CC>,
        q_point: Option<Point<C>>,
    ) -> Result<Self, String> {
        let aux_vec = aux::commitments_vector(base_gen, pedersen);

        let mut point_hasher = PointHasher::new(Self::HASH_ID);
        point_hasher.insert_point(commitments.px.commitment());
        point_hasher.insert_point(commitments.py.commitment());
        for aux in &aux_vec {
            point_hasher.insert_point(&aux.a);
            point_hasher.insert_point(aux.tx.commitment());
            point_hasher.insert_point(aux.ty.commitment());
        }

        let proofs = aux::proofs(
            aux_vec,
            point_hasher,
            base_gen,
            pedersen,
            secrets,
            commitments,
            q_point,
        )?;

        Ok(Self { proofs })
    }

    pub fn verify(
        &self,
        //rng: &mut R,
        base_gen: &Point<C>,
        pedersen: &PedersenCycle<C, CC>,
        commitments: &ExpCommitmentPoints<C, CC>,
        q_point: Option<Point<C>>,
    ) -> Result<(), String> {
        if super::SEC_PARAM > self.proofs.len() {
            return Err("security level not achieved".to_owned());
        }

        let mut tom_multimult = MultiMult::<CC>::new();
        let mut base_multimult = MultiMult::<C>::new();

        tom_multimult.add_known(Point::<CC>::GENERATOR);
        tom_multimult.add_known(pedersen.cycle().generator().clone());

        base_multimult.add_known(base_gen.clone());
        base_multimult.add_known(pedersen.base().generator().clone());
        base_multimult.add_known(commitments.exp.clone());

        let tom_multimult = Arc::new(Mutex::new(tom_multimult));
        let base_multimult = Arc::new(Mutex::new(base_multimult));

        let mut point_hasher = PointHasher::new(Self::HASH_ID);
        point_hasher.insert_point(&commitments.px);
        point_hasher.insert_point(&commitments.py);

        for i in 0..super::SEC_PARAM {
            point_hasher.insert_point(&self.proofs[i].a);
            point_hasher.insert_point(&self.proofs[i].tx_p);
            point_hasher.insert_point(&self.proofs[i].ty_p);
        }

        aux::aggregate_proofs(
            base_gen,
            pedersen,
            commitments,
            q_point,
            &self.proofs,
            point_hasher,
            &tom_multimult,
            &base_multimult,
        )?;

        let tom_res = Arc::try_unwrap(tom_multimult)
            .unwrap()
            .into_inner()
            .unwrap()
            .evaluate();
        let base_res = Arc::try_unwrap(base_multimult)
            .unwrap()
            .into_inner()
            .unwrap()
            .evaluate();

        if !(tom_res.is_identity() && base_res.is_identity()) {
            return Err("proof is invalid".to_owned());
        }
        Ok(())
    }
}

fn padded_bits(number: U256, length: usize) -> Vec<bool> {
    let mut ret = Vec::<bool>::with_capacity(length);

    let number_bytes = number.to_le_bytes();

    let mut current_idx = 0;
    for byte in number_bytes {
        let mut byte_copy = byte;
        for _ in 0..8 {
            ret.push(byte_copy % 2 == 1);
            byte_copy >>= 1;
            current_idx += 1;

            if current_idx >= length {
                return ret;
            }
        }
    }

    ret.truncate(length);
    ret
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::curve::{Secp256k1, Tom256k1};
    use rand::rngs::StdRng;
    use rand_core::SeedableRng;

    #[test]
    fn padded_bits_valid() {
        let test_u256 = U256::from_u32(1);
        assert_eq!(padded_bits(test_u256, 1), [true]);
        assert_eq!(padded_bits(test_u256, 4), [true, false, false, false]);

        let test_u256 = U256::from_u32(2);
        assert_eq!(padded_bits(test_u256, 1), [false]);
        assert_eq!(padded_bits(test_u256, 2), [false, true]);

        let test_u256 =
            U256::from_be_hex("000000000000000000000000000000000000000000000000FFFFFFFFFFFFFFFF");
        let true_64 = vec![true; 64];
        assert_eq!(padded_bits(test_u256, 64), true_64);
        assert_eq!(padded_bits(test_u256, 65), [true_64, vec![false]].concat());
    }

    #[test]
    fn exp_proof_valid_without_q() {
        let mut rng = StdRng::from_seed([2; 32]);
        let base_gen = Point::<Secp256k1>::GENERATOR;
        let pedersen = PedersenCycle::<Secp256k1, Tom256k1>::new(&mut rng);

        let exponent = Scalar::<Secp256k1>::random(&mut rng);
        let result = Point::<Secp256k1>::GENERATOR.scalar_mul(&exponent);

        let secrets = ExpSecrets::new(exponent, result.into());
        let commitments = secrets.commit(&mut rng, &pedersen);

        let exp_proof = ExpProof::construct(
            //&mut rng,
            &base_gen,
            &pedersen,
            &secrets,
            &commitments,
            None,
        )
        .unwrap();

        assert!(exp_proof
            .verify(
                //&mut rng,
                &base_gen,
                &pedersen,
                &commitments.into_commitments(),
                None
            )
            .is_ok())
    }

    #[test]
    fn exp_proof_valid_with_q() {
        let mut rng = StdRng::from_seed([2; 32]);
        let base_gen = Point::<Secp256k1>::GENERATOR;
        let pedersen = PedersenCycle::<Secp256k1, Tom256k1>::new(&mut rng);

        let q_point = Point::<Secp256k1>::GENERATOR.double();
        let exponent = Scalar::<Secp256k1>::random(&mut rng);
        let result = &Point::<Secp256k1>::GENERATOR.scalar_mul(&exponent) - &q_point;

        let secrets = ExpSecrets::new(exponent, result.into());
        let commitments = secrets.commit(&mut rng, &pedersen);

        let exp_proof = ExpProof::construct(
            //&mut rng,
            &base_gen,
            &pedersen,
            &secrets,
            &commitments,
            Some(q_point.clone()),
        )
        .unwrap();

        assert!(exp_proof
            .verify(
                //&mut rng,
                &base_gen,
                &pedersen,
                &commitments.into_commitments(),
                Some(q_point),
            )
            .is_ok())
    }

    #[test]
    fn exp_proof_invalid() {
        let mut rng = StdRng::from_seed([22; 32]);
        let base_gen = Point::<Secp256k1>::GENERATOR;
        let pedersen = PedersenCycle::<Secp256k1, Tom256k1>::new(&mut rng);

        let exponent = Scalar::<Secp256k1>::random(&mut rng);
        let result = Point::<Secp256k1>::GENERATOR.scalar_mul(&(exponent + Scalar::ONE));

        let secrets = ExpSecrets::new(exponent, result.into());
        let commitments = secrets.commit(&mut rng, &pedersen);

        let exp_proof = ExpProof::construct(
            //&mut rng,
            &base_gen,
            &pedersen,
            &secrets,
            &commitments,
            None,
        )
        .unwrap();

        assert!(exp_proof
            .verify(
                //&mut rng,
                &base_gen,
                &pedersen,
                &commitments.into_commitments(),
                None
            )
            .is_err());
    }
}
