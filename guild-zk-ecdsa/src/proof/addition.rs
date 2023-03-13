use crate::CycleCurveConfig;
use crate::pedersen::Parameters;
use super::multiplication::MultiplicationProof;
use ark_ec::short_weierstrass::{Affine, Projective, SWCurveConfig};
use ark_ec::{AffineRepr, CurveConfig, CurveGroup};
use ark_ff::Field;
use ark_std::{rand::Rng, UniformRand};

/// Proof that `P + Q = R` where all three points are on curve `C`.
pub struct AdditionProof<C, CC> {
    base: core::marker::PhantomData<C>,
    cycle: core::marker::PhantomData<CC>,
}

impl<C, CC> AdditionProof<C, CC>
where
    C: SWCurveConfig,
    CC: SWCurveConfig + CycleCurveConfig<BaseCurveConfig = C>,
    Projective<C>: CurveGroup<
        BaseField = <CC as CurveConfig>::ScalarField,
        ScalarField = <CC as CurveConfig>::BaseField,
    >,
{
    pub fn construct<R: Rng + ?Sized>(
        rng: &mut R,
        parameters: &Parameters<CC>,
        point_p: Affine<C>,
        point_q: Affine<C>,
        point_r: Affine<C>,
    ) -> Self {
        // NOTE unwraps below are fine because it is
        // assumed that all input points are valid
        let aux_7 = *point_q.x().unwrap() - point_p.x().unwrap();
        let aux_8 = aux_7.inverse().unwrap_or_default();
        let aux_9 = *point_q.y().unwrap() - point_p.y().unwrap();
        let aux_10 = aux_8 * aux_9;
        let aux_11 = aux_10 * aux_10;
        let aux_12 = *point_p.x().unwrap() - point_r.x().unwrap();
        let aux_13 = aux_10 * aux_12;

        let mult_proof_8 = MultiplicationProof::construct(
            rng,
            parameters,
            CC::to_cycle(&aux_7),
            CC::to_cycle(&aux_8),
            CC::ScalarField::one(),
        );

        Self {
            base: core::marker::PhantomData,
            cycle: core::marker::PhantomData,
        }
    }
}


// TODO test what happens if P = Q
