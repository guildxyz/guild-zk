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
        secret: C::ScalarField,
    ) -> Result<Self, String> {
        // commit to the secret scalar twice
        let secret_scalar_bytes = secret.into_bigint().to_bytes_le();
        let randomness_to_secret_1 = Randomness::<Projective<C>>::rand(rng);
        let commitment_to_secret_1 = Commitment::<Projective<C>, Window>::commit(
            parameters,
            &secret_scalar_bytes,
            &randomness_to_secret_1,
        )
        .map_err(|e| e.to_string())?;
        let randomness_to_secret_2 = Randomness::<Projective<C>>::rand(rng);
        let commitment_to_secret_2 = Commitment::<Projective<C>, Window>::commit(
            parameters,
            &secret_scalar_bytes,
            &randomness_to_secret_2,
        )
        .map_err(|e| e.to_string())?;

        // commit to a random scalar twice
        let random_scalar = C::ScalarField::rand(rng);
        let random_scalar_bytes = random_scalar.into_bigint().to_bytes_le();

        let randomness_to_random_1 = Randomness::<Projective<C>>::rand(rng);
        let commitment_to_random_1 = Commitment::<Projective<C>, Window>::commit(
            parameters,
            &random_scalar_bytes,
            &randomness_to_random_1,
        )
        .map_err(|e| e.to_string())?;

        let randomness_to_random_2 = Randomness::<Projective<C>>::rand(rng);
        let commitment_to_random_2 = Commitment::<Projective<C>, Window>::commit(
            parameters,
            &random_scalar_bytes,
            &randomness_to_random_2,
        )
        .map_err(|e| e.to_string())?;

        let challenge = crate::hash::hash_points(&[
            &commitment_to_secret_1,
            &commitment_to_secret_2,
            &commitment_to_random_1,
            &commitment_to_random_2,
        ])?;

        let mask_secret = random_scalar - challenge * secret;
        let mask_random_1 = randomness_to_random_1.0 - challenge * randomness_to_secret_1.0;
        let mask_random_2 = randomness_to_random_2.0 - challenge * randomness_to_secret_2.0;

        Ok(Self {
            commitment_to_random_1,
            commitment_to_random_2,
            mask_secret,
            mask_random_1,
            mask_random_2,
        })
    }
}
