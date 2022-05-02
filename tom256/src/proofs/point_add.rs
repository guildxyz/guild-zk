use crate::arithmetic::{Modular, Point, Scalar};
use crate::pedersen::*;
use crate::{Curve, Cycle};

use super::equality::EqualityProof;
use super::multiplication::MultiplicationProof;

use rand_core::{CryptoRng, RngCore};
use std::marker::PhantomData;

#[derive(Clone)]
pub struct PointAddSecrets<C> {
    p: Point<C>,
    q: Point<C>,
    r: Point<C>,
}

impl<C: Curve> PointAddSecrets<C> {
    // TODO: using `AffinePoint` this could be implied
    /// Ensures that the stored point is affine.
    pub fn new(p: Point<C>, q: Point<C>, r: Point<C>) -> Self {
        // TODO debug_assert!(p + q = r) ?
        Self {
            p: p.into_affine(),
            q: q.into_affine(),
            r: r.into_affine(),
        }
    }

    pub fn commitments<R, CC>(
        &self,
        rng: &mut R,
        pedersen_generator: &PedersenGenerator<CC>,
    ) -> PointAddCommitments<CC>
    where
        R: CryptoRng + RngCore,
        CC: Cycle<C>,
    {
        PointAddCommitments {
            px: pedersen_generator.commit(rng, self.p.x().to_cycle_scalar()),
            py: pedersen_generator.commit(rng, self.p.y().to_cycle_scalar()),
            qx: pedersen_generator.commit(rng, self.q.x().to_cycle_scalar()),
            qy: pedersen_generator.commit(rng, self.q.y().to_cycle_scalar()),
            rx: pedersen_generator.commit(rng, self.r.x().to_cycle_scalar()),
            ry: pedersen_generator.commit(rng, self.r.y().to_cycle_scalar()),
        }
    }
}

#[derive(Clone)]
pub struct PointAddCommitments<C> {
    px: PedersenCommitment<C>,
    py: PedersenCommitment<C>,
    qx: PedersenCommitment<C>,
    qy: PedersenCommitment<C>,
    rx: PedersenCommitment<C>,
    ry: PedersenCommitment<C>,
}

impl<C: Curve> PointAddCommitments<C> {
    pub fn commitments(&self) -> [&Point<C>; 6] {
        [
            self.px.commitment(),
            self.py.commitment(),
            self.qx.commitment(),
            self.qx.commitment(),
            self.rx.commitment(),
            self.rx.commitment(),
        ]
    }
}

pub struct MultCommitProof<C> {
    commitment: Point<C>,
    proof: MultiplicationProof<C>,
}

impl<C: Curve> MultCommitProof<C> {
    pub fn new(commitment: Point<C>, proof: MultiplicationProof<C>) -> Self {
        Self { commitment, proof }
    }
}

pub struct PointAddProof<C, CC> {
    mult_proof_8: MultCommitProof<C>,
    mult_proof_10: MultCommitProof<C>,
    mult_proof_11: MultCommitProof<C>,
    mult_proof_13: MultCommitProof<C>,
    equality_proof_x: EqualityProof<C>,
    equality_proof_y: EqualityProof<C>,
    cycle: PhantomData<CC>,
}

impl<C: Cycle<CC>, CC: Cycle<C>> PointAddProof<C, CC> {
    pub fn construct<R: CryptoRng + RngCore>(
        rng: &mut R,
        pedersen_generator: &PedersenGenerator<C>,
        commitments: &PointAddCommitments<C>,
        points: &PointAddSecrets<CC>,
    ) -> Self {
        // P + Q = R
        // P: (x1, y1)
        // Q: (x2, y2)
        // R: (x3, y3)
        // auxiliary variables (i8 is a type, so use aux8)
        let aux_7 = points.q.x() - points.p.x();
        let aux_8 = aux_7.inverse();
        let aux_9 = points.q.y() - points.p.y();
        let aux_10 = aux_8 * aux_9;
        let aux_11 = aux_10 * aux_10;
        let aux_12 = points.p.x() - points.r.x();
        let aux_13 = aux_10 * aux_12;
        let commitment_7 = &commitments.qx - &commitments.px;
        let commitment_8 = pedersen_generator.commit(rng, aux_8.to_cycle_scalar());
        let commitment_9 = &commitments.qy - &commitments.py;
        let commitment_10 = pedersen_generator.commit(rng, aux_10.to_cycle_scalar());
        let commitment_11 = pedersen_generator.commit(rng, aux_11.to_cycle_scalar());
        let commitment_12 = &commitments.px - &commitments.rx;
        let commitment_13 = pedersen_generator.commit(rng, aux_13.to_cycle_scalar());
        let commitment_14 = PedersenCommitment::new(Point::<C>::GENERATOR, Scalar::<C>::ZERO);

        let mult_proof_8 = MultiplicationProof::construct(
            rng,
            pedersen_generator,
            &commitment_7,
            &commitment_8,
            &commitment_14,
            aux_7.to_cycle_scalar(),
            aux_8.to_cycle_scalar(),
            Scalar::<C>::ONE,
        );
        let mult_proof_10 = MultiplicationProof::construct(
            rng,
            pedersen_generator,
            &commitment_8,
            &commitment_9,
            &commitment_10,
            aux_8.to_cycle_scalar(),
            aux_9.to_cycle_scalar(),
            aux_10.to_cycle_scalar(),
        );
        let mult_proof_11 = MultiplicationProof::construct(
            rng,
            pedersen_generator,
            &commitment_10,
            &commitment_10,
            &commitment_11,
            aux_10.to_cycle_scalar(),
            aux_10.to_cycle_scalar(),
            aux_11.to_cycle_scalar(),
        );

        let aux_commitment = &(&commitments.rx + &commitments.px) + &commitments.qx;
        let equality_proof_x = EqualityProof::construct(
            rng,
            pedersen_generator,
            &commitment_11,
            &aux_commitment,
            aux_11.to_cycle_scalar(),
        );
        let mult_proof_13 = MultiplicationProof::construct(
            rng,
            pedersen_generator,
            &commitment_10,
            &commitment_12,
            &commitment_13,
            aux_10.to_cycle_scalar(),
            aux_12.to_cycle_scalar(),
            aux_13.to_cycle_scalar(),
        );

        let aux_commitment = &commitments.ry + &commitments.py;
        let equality_proof_y = EqualityProof::construct(
            rng,
            pedersen_generator,
            &commitment_13,
            &aux_commitment,
            aux_13.to_cycle_scalar(),
        );

        Self {
            mult_proof_8: MultCommitProof::new(commitment_8.into_commitment(), mult_proof_8),
            mult_proof_10: MultCommitProof::new(commitment_10.into_commitment(), mult_proof_10),
            mult_proof_11: MultCommitProof::new(commitment_11.into_commitment(), mult_proof_11),
            mult_proof_13: MultCommitProof::new(commitment_13.into_commitment(), mult_proof_13),
            equality_proof_x,
            equality_proof_y,
            cycle: PhantomData,
        }
    }
}
