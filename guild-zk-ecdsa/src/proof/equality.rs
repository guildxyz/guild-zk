use crate::pedersen::{Commitment, CommitmentScheme, Parameters, Randomness, Window};
use ark_ec::models::short_weierstrass::{Affine, Projective, SWCurveConfig};
use ark_ec::models::CurveConfig;
use ark_ff::{BigInteger, PrimeField};
use ark_std::{rand::Rng, UniformRand};

pub struct EqualityProof<C: CurveConfig + SWCurveConfig> {
    commitment_to_random_1: Affine<C>,
    commitment_to_random_2: Affine<C>,
    mask_secret: C::ScalarField,
    mask_random_1: C::ScalarField,
    mask_random_2: C::ScalarField,
}

impl<C> EqualityProof<C>
where
    C: CurveConfig + SWCurveConfig,
{
    pub fn construct<R: Rng + ?Sized>(
        rng: &mut R,
        parameters: &Parameters<Projective<C>>,
        commitment_1: &Commitment<Projective<C>, Window>,
        commitment_2: &Commitment<Projective<C>, Window>,
        secret: C::ScalarField,
    ) -> Result<Self, String> {
        let random_scalar_bytes = C::ScalarField::rand(rng).into_bigint().to_bytes_le();

        let randomness_1 = Randomness::<Projective<C>>::rand(rng);
        let commitment_to_random_1 = Commitment::<Projective<C>, Window>::commit(
            parameters,
            &random_scalar_bytes,
            &randomness_1,
        )
        .map_err(|e| e.to_string())?;

        let randomness_2 = Randomness::<Projective<C>>::rand(rng);
        let commitment_to_random_1 = Commitment::<Projective<C>, Window>::commit(
            parameters,
            &random_scalar_bytes,
            &randomness_2,
        )
        .map_err(|e| e.to_string())?;
        todo!()
    }
}
