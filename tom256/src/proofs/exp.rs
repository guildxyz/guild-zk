use super::PointAddProof;
use crate::arithmetic::multimult::{MultiMult, Relation};
use crate::arithmetic::{Modular, Point, Scalar};
use crate::pedersen::*;
use crate::utils::{hash_points, PointHasher};
use crate::{Curve, Cycle};

use std::ops::Neg;

use bigint::Integer;

use rand_core::{CryptoRng, RngCore};

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
        add_proof: PointAddProof<CC, C>,
        t1_x: Scalar<CC>,
        t1_y: Scalar<CC>,
    },
}

pub struct ExpProof<C: Curve, CC: Cycle<C>> {
    a: Point<C>,
    tx_p: Point<CC>,
    ty_p: Point<CC>,
    variant: ExpProofVariant<C, CC>,
}

#[derive(Clone)]
pub struct PointExpSecrets<C> {
    exp: Scalar<C>,
    point: Point<C>,
    q: Option<Point<C>>,
}

#[derive(Clone)]
pub struct PointExpCommitments<C, CC> {
    px: PedersenCommitment<CC>,
    py: PedersenCommitment<CC>,
    exp: PedersenCommitment<C>,
}

impl<C: Curve> PointExpSecrets<C> {
    // TODO: using `AffinePoint` this could be implied
    /// Ensures that the stored point is affine.
    pub fn new(exp: Scalar<C>, point: Point<C>, q: Option<Point<C>>) -> Self {
        // TODO debug_assert!(p + q = r) ?
        let q = if let Some(pt) = q {
            Some(pt.into_affine())
        } else {
            None
        };
        Self {
            point: point.into_affine(),
            exp,
            q,
        }
    }

    pub fn commit<R, CC>(
        &self,
        rng: &mut R,
        base_pedersen_generator: &PedersenGenerator<C>,
        tom_pedersen_generator: &PedersenGenerator<CC>,
    ) -> PointExpCommitments<C, CC>
    where
        R: CryptoRng + RngCore,
        CC: Cycle<C>,
    {
        PointExpCommitments {
            px: tom_pedersen_generator.commit(rng, self.point.x().to_cycle_scalar()),
            py: tom_pedersen_generator.commit(rng, self.point.y().to_cycle_scalar()),
            exp: base_pedersen_generator.commit(rng, self.exp),
        }
    }
}

impl<CC: Cycle<C>, C: Curve> ExpProof<C, CC> {
    const HASH_ID: &'static [u8] = b"exp-proof";

    pub fn construct<R: CryptoRng + RngCore>(
        rng: &mut R,
        base_pedersen_generator: &PedersenGenerator<C>,
        tom_pedersen_generator: &PedersenGenerator<CC>,
        secrets: &PointExpSecrets<C>,
        commitments: &PointExpCommitments<C, CC>,
        security_param: usize,
    ) -> Vec<Self> {
        let mut alpha_vec: Vec<Scalar<C>> = Vec::with_capacity(security_param);
        let mut r_vec: Vec<Scalar<C>> = Vec::with_capacity(security_param);
        let mut t_vec: Vec<Point<C>> = Vec::with_capacity(security_param);
        let mut a_vec: Vec<Point<C>> = Vec::with_capacity(security_param);
        let mut tx_vec: Vec<PedersenCommitment<CC>> = Vec::with_capacity(security_param);
        let mut ty_vec: Vec<PedersenCommitment<CC>> = Vec::with_capacity(security_param);

        for i in 0..security_param {
            // TODO: probably push instead of vec[i]
            alpha_vec[i] = Scalar::random(rng);
            r_vec[i] = Scalar::random(rng);
            t_vec[i] = Point::GENERATOR.scalar_mul(&alpha_vec[i]);
            a_vec[i] = &t_vec[i]
                + &base_pedersen_generator
                    .generator()
                    .clone()
                    .scalar_mul(&r_vec[i]);

            let coord_t = &t_vec[i].clone().into_affine();
            if coord_t.is_identity() {
                // TODO: dont panic or smth
                panic!("intermediate value is identity");
            }

            tx_vec[i] = tom_pedersen_generator.commit(rng, coord_t.x().to_cycle_scalar());
            ty_vec[i] = tom_pedersen_generator.commit(rng, coord_t.y().to_cycle_scalar());
        }

        let mut point_hasher = PointHasher::new(Self::HASH_ID);
        point_hasher.insert_point(commitments.px.clone().into_commitment());
        point_hasher.insert_point(commitments.py.clone().into_commitment());

        for i in 0..security_param {
            point_hasher.insert_point(a_vec[i].clone());
            point_hasher.insert_point(tx_vec[i].clone().into_commitment());
            point_hasher.insert_point(ty_vec[i].clone().into_commitment());
        }
        let mut challenge = point_hasher.finalize();

        let mut all_exp_proofs = Vec::<ExpProof<C, CC>>::with_capacity(security_param);

        for i in 0..security_param {
            if challenge.is_odd().into() {
                all_exp_proofs.push(ExpProof {
                    a: a_vec[i].clone(),
                    tx_p: tx_vec[i].clone().into_commitment(),
                    ty_p: ty_vec[i].clone().into_commitment(),
                    variant: ExpProofVariant::Odd {
                        alpha: alpha_vec[i],
                        r: r_vec[i],
                        tx_r: tx_vec[i].randomness().clone(),
                        ty_r: ty_vec[i].randomness().clone(),
                    },
                });
            } else {
                // TODO
            }
            challenge = challenge >> 1;
        }
        // TODO: all_exp_proofs
        vec![]
    }

    pub fn aggregate<R: CryptoRng + RngCore>(
        &self,
        rng: &mut R,
        base_pedersen_generator: &PedersenGenerator<C>,
        tom_pedersen_generator: &PedersenGenerator<CC>,
        commitment_1: &Point<C>,
        commitment_2: &Point<C>,
        multimult: &mut MultiMult<C>,
    ) {
    }

    pub fn verify<R: CryptoRng + RngCore>(
        &self,
        rng: &mut R,
        pedersen_generator: &PedersenGenerator<C>,
        commitment_1: &Point<C>,
        commitment_2: &Point<C>,
    ) -> bool {
        todo!();
    }
}
