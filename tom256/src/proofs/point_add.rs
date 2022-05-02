use crate::arithmetic::{Point, Scalar};
use crate::pedersen::*;
use crate::Curve;

use super::equality::EqualityProof;
use super::multiplication::MultiplicationProof;

pub struct PointAddSecrets<C: Curve> {
    p: Point<C>,
    q: Point<C>,
    r: Point<C>,
}

pub struct PointAddCommitments<C: Curve> {
    px: PedersenCommitment<C>,
    py: PedersenCommitment<C>,
    qx: PedersenCommitment<C>,
    qy: PedersenCommitment<C>,
    rx: PedersenCommitment<C>,
    ry: PedersenCommitment<C>,
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

    pub fn commitments<R: CryptoRng + RngCore>(
        &self,
        rng: &mut R,
        pedersen_generator: &PedersenGenerator,
    ) -> PointAddCommitments {
        // NOTE -> commitments must be on tom curve because we are committing to field elements
        // TODO -> point field element should be converted into another curve's scalar field
        PointAddCommitments {
            px: pedersen_generator.commit(rng, self.p.x()),
            py: pedersen_generator.commit(rng, self.p.y()),
            qx: pedersen_generator.commit(rng, self.q.x()),
            qy: pedersen_generator.commit(rng, self.q.y()),
            rx: pedersen_generator.commit(rng, self.r.x()),
            ry: pedersen_generator.commit(rng, self.r.y()),
        }
    }
}

pub struct MultCommitProof<C: Curve> {
    commitment: Point<C>,
    proof: MultiplicationProof<C>,
}

impl<C: Curve> MultCommitProof<C: Curve> {
    pub fn new(commitment: Point<C>, proof: MultiplicationProof<C>) {
        Self { commitment, proof }
    }
}

pub struct PointAddProof<C: Curve> {
    mult_proof_8: MultCommitProof<C>,
    mult_proof_10: MultCommitProof<C>,
    mult_proof_11: MultCommitProof<C>,
    mult_proof_13: MultCommitProof<C>,
    equality_proof_x: EqualityProof<C>,
    equality_proof_y: EqualityProof<C>,
}

impl<C: Curve> PointAddProof<C> {
    pub fn construct<R: CryptoRng + RngCore>(
        rng: &mut R,
        pedersen_generator: &PedersenGenerator,
        points: &PointAddSecrets,
        commitments: &PointAddCommitments,
    ) -> Self {
        // P + Q = R
        // P: (x1, y1)
        // Q: (x2, y2)
        // R: (x3, y3)
        // auxiliary variables (i8 is a type, so use aux8)
        let aux_8 = (points.q.x() - points.p.x()).inverse();
        let aux_9 = points.q.y() - points.p.y();
        let aux_10 = aux_8 * aux_9;
        let aux_11 = aux_10 * aux_10;
        let aux_12 = points.p.x() - points.r.x();
        let aux_13 = i10 * i12;
        let commitment_7 = &commitments.qx - &commitments.px;
        let commitment_8 = pedersen_generator.commit(rng, aux_8);
        let commitment_9 = &commitments.qy - &commitments.py;
        let commitment_10 = pedersen_generator.commit(rng, aux_10);
        let commitment_11 = pedersen_generator.commit(rng, aux_11);
        let commitment_12 = &commitments.px - &commitments.rx;
        let commitment_13 = pedersen_generator.commit(rng, aux_13);
        let commitment_14 = PedersenCommitment::new(Point::<C>::GENERATOR, Scalar::<C>::ZERO);

        let mult_proof_8 = MultiplicationProof::construct(todo!());
        let mult_proof_10 = MultiplicationProof::construct(todo!());
        let mult_proof_11 = MultiplicationProof::construct(todo!());

        let mut aux_commitment = &(&commitments.rx + &commitments.px) + &commitments.qx;
        let equality_proof_x = EqualityProof::construct(
            rng,
            pedersen_generator,
            &commitment_11,
            &aux_commitment,
            aux_11,
        );
        let mult_proof_13 = MultiplicationProof::construct(todo!());

        let aux_commitment = &commitments.ry + &commitments.py;
        let equality_proof_x = EqualityProof::construct(
            rng,
            pedersen_generator,
            &commitment_13,
            &aux_commitment,
            aux_13,
        );

        Self {
            mult_proof_8: MultCommitProof::new(commitment_8.into_commitment(), mult_proof_8),
            mult_proof_10: MultCommitProof::new(commitment_10.into_commitment(), mult_proof_10),
            mult_proof_11: MultCommitProof::new(commitment_11.into_commitment(), mult_proof_11),
            mult_proof_13: MultCommitProof::new(commitment_13.into_commitment(), mult_proof_13),
            equality_proof_x,
            equality_proof_y,
        }
    }
}
