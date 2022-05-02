use crate::arithmetic::{Point, Scalar};
use crate::pedersen::*;
use crate::Curve;

use super::equality::EqualityProof;
use super::multiplication::MultiplicationProof;

pub struct CommittedPoint<C: Curve> {
    affine: Point<C>,
    commitment_to_x: PedersenCommitment<C>,
    commitment_to_y: PedersenCommitment<C>,
}

impl<C: Curve> CommittedPoint<C> {
    // TODO: using `AffinePoint` this could be implied
    /// Ensures that the stored point is affine.
    pub fn new<R: CryptoRng + RngCore>(
        rng: &mut R,
        pedersen_generator: &PedersenGenerator,
        point: &Point<C>,
    ) -> Self {
        let affine = point.into_affine();
        let commitment_to_x = pedersen_generator.commit(affine.x());
        let commitment_to_y = pedersen_generator.commit(affine.y());
        Self {
            affine,
            commitment_to_x,
            commitment_to_y,
        }
    }
}

/// Represents commitments to P + Q = R
pub struct PointAddInput<C: Curve> {
    p: CommittedPoint<C>,
    q: CommittedPoint<C>,
    r: CommittedPoint<C>,
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
        input: PointAddInput,
    ) -> Self {
        // P + Q = R
        // P: (x1, y1)
        // Q: (x2, y2)
        // R: (x3, y3)
        // auxiliary variables (i8 is a type, so use aux8)
        let aux_8 = (input.q.affine.x() - input.p.affine.x()).inverse();
        let aux_9 = input.q.affine.y() - input.p.affine.y();
        let aux_10 = aux_8 * aux_9;
        let aux_11 = aux_10 * aux_10;
        let aux_12 = input.p.affine.x() - input.r.affine.x();
        let aux_13 = i10 * i12;
        let commitment_7 = input.q.commitment_to_x() - input.p.commitment_to_x();
        let commitment_8 = pedersen_generator.commit(rng, aux_8);
        let commitment_9 = input.q.commitment_to_y() - input.p.commitment_to_y();
        let commitment_10 = pedersen_generator.commit(rng, aux_10);
        let commitment_11 = pedersen_generator.commit(rng, aux_11);
        let commitment_12 = input.p.commitment_to_x() - input.r.commitment_to_x();
        let commitment_13 = pedersen_generator.commit(rng, aux_13);
        let commitment_14 = PedersenCommitment::new(Point::<C>::GENERATOR, Scalar::<C>::ZERO);

        let mult_proof_8 = MultiplicationProof::construct(todo!());
        let mult_proof_10 = MultiplicationProof::construct(todo!());
        let mult_proof_11 = MultiplicationProof::construct(todo!());

        let aux_commitment =
            input.r.commitment_to_x() + input.p.commitment_to_x() + input.q.commitment_to_x();
        let equality_proof_x = EqualityProof::construct(
            rng,
            pedersen_generator,
            &commitment_11,
            &aux_commitment,
            aux_11,
        );
        let mult_proof_13 = MultiplicationProof::construct(todo!());

        let aux_commitment = input.r.commitment_to_y() + input.p.commitment_to_y();
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
