use super::*;
use crate::arithmetic::multimult::{MultiMult, Relation};
use crate::arithmetic::{AffinePoint, Point, Scalar};
use crate::curve::{Curve, Cycle};
use crate::hasher::PointHasher;
use crate::pedersen::{PedersenCommitment, PedersenCycle};
use crate::proofs::SEC_PARAM;

#[cfg(feature = "parallel")]
use rayon::prelude::*;

use std::sync::{Arc, Mutex};

pub struct AuxiliaryCommitments<C: Curve, CC: Cycle<C>> {
    pub alpha: Scalar<C>,
    pub r: Scalar<C>,
    pub a: Point<C>,
    pub t: AffinePoint<C>,
    pub tx: PedersenCommitment<CC>,
    pub ty: PedersenCommitment<CC>,
}

pub fn commitments_vector<C: Curve, CC: Cycle<C>>(
    base_gen: Point<C>,
    pedersen: &PedersenCycle<C, CC>,
) -> Vec<AuxiliaryCommitments<C, CC>> {
    #[cfg(feature = "parallel")]
    let iter = (0..SEC_PARAM).into_par_iter();
    #[cfg(not(feature = "parallel"))]
    let iter = 0..SEC_PARAM;

    iter.map(|_| {
        let mut rng = rand_core::OsRng;
        // exponent (XXX what if this is zero?)
        let alpha = Scalar::random(&mut rng);
        // random nonce
        let r = Scalar::random(&mut rng);
        // T = g^alpha
        let t: AffinePoint<C> = (base_gen * alpha).into();
        // A = g^alpha = h^r (essentially a commitment in the base curve)
        let a = t + (pedersen.base().generator() * r).to_affine();

        // commitment to Tx
        let tx = pedersen.cycle().commit(&mut rng, t.x().to_cycle_scalar());
        // commitment to Ty
        let ty = pedersen.cycle().commit(&mut rng, t.y().to_cycle_scalar());

        AuxiliaryCommitments {
            alpha,
            r,
            a,
            t,
            tx,
            ty,
        }
    })
    .collect()
}

pub fn proofs<C: Curve, CC: Cycle<C>>(
    aux_vec: Vec<AuxiliaryCommitments<C, CC>>,
    point_hasher: PointHasher,
    base_gen: Point<C>,
    pedersen: &PedersenCycle<C, CC>,
    secrets: &ExpSecrets<C>,
    commitments: &ExpCommitments<C, CC>,
    q_point: Option<Point<C>>,
) -> Result<Vec<SingleExpProof<C, CC>>, String> {
    let challenge = padded_bits(point_hasher.finalize(), SEC_PARAM);
    debug_assert_eq!(aux_vec.len(), challenge.len());

    #[cfg(feature = "parallel")]
    let aux_vec_iter = aux_vec.into_par_iter();
    #[cfg(feature = "parallel")]
    let challenge_iter = challenge.into_par_iter();
    #[cfg(not(feature = "parallel"))]
    let aux_vec_iter = aux_vec.into_iter();
    #[cfg(not(feature = "parallel"))]
    let challenge_iter = challenge.into_iter();

    let proofs = aux_vec_iter
        .zip(challenge_iter)
        .flat_map(|(aux, c_bit)| {
            if c_bit {
                let tx_r = aux.tx.randomness();
                let ty_r = aux.ty.randomness();
                Ok(SingleExpProof {
                    a: aux.a,
                    tx_p: aux.tx.commitment(),
                    ty_p: aux.ty.commitment(),
                    variant: ExpProofVariant::Odd {
                        alpha: aux.alpha,
                        r: aux.r,
                        tx_r,
                        ty_r,
                    },
                })
            } else {
                let z = aux.alpha - secrets.exp;
                let mut t1 = base_gen * z;
                if let Some(pt) = q_point.as_ref() {
                    t1 += pt;
                }

                if t1.is_identity() {
                    return Err("intermediate value is identity".to_owned());
                }

                // Generate point add proof
                let add_secret = PointAddSecrets::new(t1.into(), secrets.point, aux.t);
                let add_commitments = add_secret.commit_p_only(
                    &mut rand_core::OsRng,
                    &pedersen.cycle(),
                    commitments.px.clone(),
                    commitments.py.clone(),
                    aux.tx.clone(),
                    aux.ty.clone(),
                );
                let add_proof = PointAddProof::construct(
                    &mut rand_core::OsRng,
                    &pedersen.cycle(),
                    &add_commitments,
                    &add_secret,
                );

                Ok(SingleExpProof {
                    a: aux.a,
                    tx_p: aux.tx.commitment(),
                    ty_p: aux.ty.commitment(),
                    variant: ExpProofVariant::Even {
                        z,
                        r: aux.r - commitments.exp.randomness(),
                        t1_x: add_commitments.px.randomness(),
                        t1_y: add_commitments.py.randomness(),
                        add_proof,
                    },
                })
            }
        })
        .collect::<Vec<_>>();

    Ok(proofs)
}

#[allow(clippy::too_many_arguments)]
pub fn aggregate_proofs<C: Curve, CC: Cycle<C>>(
    base_gen: Point<C>,
    pedersen: &PedersenCycle<C, CC>,
    commitments: &ExpCommitmentPoints<C, CC>,
    q_point: Option<Point<C>>,
    proofs: &[SingleExpProof<C, CC>],
    point_hasher: PointHasher,
    tom_multimult: &Arc<Mutex<MultiMult<CC>>>,
    base_multimult: &Arc<Mutex<MultiMult<C>>>,
) -> Result<(), String> {
    let challenge = padded_bits(point_hasher.finalize(), proofs.len());

    #[cfg(feature = "parallel")]
    let proofs_iter = proofs.into_par_iter();
    #[cfg(feature = "parallel")]
    let challenge_iter = challenge.into_par_iter();
    #[cfg(not(feature = "parallel"))]
    let proofs_iter = proofs.iter();
    #[cfg(not(feature = "parallel"))]
    let challenge_iter = challenge.into_iter();

    proofs_iter
        .zip(challenge_iter)
        .try_for_each(|(proof, c_bit)| {
            let mut rng = rand_core::OsRng;
            match &proof.variant {
                ExpProofVariant::Odd {
                    alpha,
                    r,
                    tx_r,
                    ty_r,
                } => {
                    if !c_bit {
                        return Err("challenge hash mismatch".to_owned());
                    }

                    let t = base_gen.scalar_mul(alpha);
                    let mut relation_a = Relation::<C>::new();

                    relation_a.insert(t, Scalar::<C>::ONE);
                    relation_a.insert(pedersen.base().generator(), *r);
                    relation_a.insert((&proof.a).neg(), Scalar::<C>::ONE);

                    relation_a.drain(&mut rng, &mut base_multimult.lock().unwrap());

                    let coord_t: AffinePoint<C> = t.into();
                    if coord_t.is_identity() {
                        return Err("intermediate value is identity".to_owned());
                    }

                    let sx = coord_t.x().to_cycle_scalar::<CC>();
                    let sy = coord_t.y().to_cycle_scalar::<CC>();

                    let mut relation_tx = Relation::new();
                    let mut relation_ty = Relation::new();

                    relation_tx.insert(Point::<CC>::GENERATOR, sx);
                    relation_tx.insert(pedersen.cycle().generator(), *tx_r);
                    relation_tx.insert((&proof.tx_p).neg(), Scalar::<CC>::ONE);

                    relation_ty.insert(Point::<CC>::GENERATOR, sy);
                    relation_ty.insert(pedersen.cycle().generator(), *ty_r);
                    relation_ty.insert((&proof.ty_p).neg(), Scalar::<CC>::ONE);

                    relation_tx.drain(&mut rng, &mut tom_multimult.lock().unwrap());
                    relation_ty.drain(&mut rng, &mut tom_multimult.lock().unwrap());
                    Ok(())
                }
                ExpProofVariant::Even {
                    z,
                    r,
                    add_proof,
                    t1_x,
                    t1_y,
                } => {
                    if c_bit {
                        return Err("challenge hash mismatch".to_owned());
                    }

                    let mut t = base_gen.scalar_mul(z);

                    let mut relation_a = Relation::<C>::new();
                    relation_a.insert(t, Scalar::<C>::ONE);
                    relation_a.insert(commitments.exp, Scalar::<C>::ONE);
                    relation_a.insert((&proof.a).neg(), Scalar::<C>::ONE);
                    relation_a.insert(pedersen.base().generator(), *r);

                    relation_a.drain(&mut rng, &mut base_multimult.lock().unwrap());

                    if let Some(pt) = q_point.as_ref() {
                        t += pt;
                    }

                    let coord_t: AffinePoint<C> = t.into();
                    if coord_t.is_identity() {
                        return Err("intermediate value is identity".to_owned());
                    }

                    let sx = coord_t.x().to_cycle_scalar::<CC>();
                    let sy = coord_t.y().to_cycle_scalar::<CC>();

                    let t1_com_x = pedersen.cycle().commit_with_randomness(sx, *t1_x);
                    let t1_com_y = pedersen.cycle().commit_with_randomness(sy, *t1_y);

                    let point_add_commitments = PointAddCommitmentPoints::new(
                        t1_com_x.commitment(),
                        t1_com_y.commitment(),
                        commitments.px,
                        commitments.py,
                        proof.tx_p,
                        proof.ty_p,
                    );

                    add_proof.aggregate(
                        &mut rng,
                        &pedersen.cycle(),
                        &point_add_commitments,
                        &mut tom_multimult.lock().unwrap(),
                    );
                    Ok(())
                }
            }
        })
}
