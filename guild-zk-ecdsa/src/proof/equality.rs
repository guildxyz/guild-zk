use crate::eval::{MultiMult, Pair, Relation};
use crate::pedersen::Parameters;
use ark_ec::short_weierstrass::{Affine, SWCurveConfig};
use ark_ff::One;
use ark_std::{rand::Rng, UniformRand};

use core::ops::Neg;

const HASH_ID: &[u8] = b"equality proof";

pub struct EqualityProof<C: SWCurveConfig> {
    commitment_to_secret_1: Affine<C>,
    commitment_to_secret_2: Affine<C>,
    commitment_to_random_1: Affine<C>,
    commitment_to_random_2: Affine<C>,
    mask_secret: C::ScalarField,
    mask_random_1: C::ScalarField,
    mask_random_2: C::ScalarField,
}

impl<C> EqualityProof<C>
where
    C: SWCurveConfig,
{
    pub fn construct<R: Rng + ?Sized>(
        rng: &mut R,
        parameters: &Parameters<C>,
        secret: C::ScalarField,
    ) -> Self {
        // commit to the secret scalar twice
        let randomness_to_secret_1 = C::ScalarField::rand(rng);
        let commitment_to_secret_1 = parameters.commit(secret, randomness_to_secret_1);
        let randomness_to_secret_2 = C::ScalarField::rand(rng);
        let commitment_to_secret_2 = parameters.commit(secret, randomness_to_secret_2);

        // commit to a random scalar twice
        let random_scalar = C::ScalarField::rand(rng);
        let randomness_to_random_1 = C::ScalarField::rand(rng);
        let commitment_to_random_1 = parameters.commit(random_scalar, randomness_to_random_1);

        let randomness_to_random_2 = C::ScalarField::rand(rng);
        let commitment_to_random_2 = parameters.commit(random_scalar, randomness_to_random_2);

        let challenge = crate::hash::hash_points(
            HASH_ID,
            &[
                &commitment_to_secret_1,
                &commitment_to_secret_2,
                &commitment_to_random_1,
                &commitment_to_random_2,
            ],
        );

        let mask_secret = random_scalar - challenge * secret;
        let mask_random_1 = randomness_to_random_1 - challenge * randomness_to_secret_1;
        let mask_random_2 = randomness_to_random_2 - challenge * randomness_to_secret_2;

        Self {
            commitment_to_secret_1,
            commitment_to_secret_2,
            commitment_to_random_1,
            commitment_to_random_2,
            mask_secret,
            mask_random_1,
            mask_random_2,
        }
    }

    pub fn aggregate<R: Rng + ?Sized>(
        &self,
        rng: &mut R,
        parameters: &Parameters<C>,
        multimult: &mut MultiMult<C>,
    ) {
        let challenge = crate::hash::hash_points(
            HASH_ID,
            &[
                &self.commitment_to_secret_1,
                &self.commitment_to_secret_2,
                &self.commitment_to_random_1,
                &self.commitment_to_random_2,
            ],
        );

        let mut relation_1 = Relation::new();
        let mut relation_2 = Relation::new();

        relation_1.insert_pair(Pair {
            point: C::GENERATOR,
            scalar: self.mask_secret,
        });
        relation_1.insert_pair(Pair {
            point: parameters.h(),
            scalar: self.mask_random_1,
        });
        relation_1.insert_pair(Pair {
            point: self.commitment_to_secret_1,
            scalar: challenge,
        });
        relation_1.insert_pair(Pair {
            point: self.commitment_to_random_1.neg(),
            scalar: C::ScalarField::one(),
        });

        relation_2.insert_pair(Pair {
            point: C::GENERATOR,
            scalar: self.mask_secret,
        });
        relation_2.insert_pair(Pair {
            point: parameters.h(),
            scalar: self.mask_random_2,
        });
        relation_2.insert_pair(Pair {
            point: self.commitment_to_secret_2,
            scalar: challenge,
        });
        relation_2.insert_pair(Pair {
            point: self.commitment_to_random_2.neg(),
            scalar: C::ScalarField::one(),
        });

        relation_1.drain(rng, multimult);
        relation_2.drain(rng, multimult);
    }

    pub fn verify<R: Rng + ?Sized>(&self, rng: &mut R, parameters: &Parameters<C>) -> bool {
        let mut multimult = MultiMult::new();
        self.aggregate(rng, parameters, &mut multimult);

        multimult.evaluate() == Affine::identity()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use ark_ec::models::CurveConfig;
    use ark_secp256k1::Config;
    use ark_std::{
        rand::{rngs::StdRng, SeedableRng},
        UniformRand,
    };

    const SEED: u64 = 1234567890;

    #[test]
    fn completeness() {
        let mut rng = StdRng::seed_from_u64(SEED);
        let secret = <Config as CurveConfig>::ScalarField::rand(&mut rng);
        let parameters = Parameters::<Config>::new(&mut rng);
        let proof = EqualityProof::construct(&mut rng, &parameters, secret);
        assert!(proof.verify(&mut rng, &parameters));
    }

    #[test]
    fn soundness() {
        let mut rng = StdRng::seed_from_u64(SEED);
        // switch parameters
        let secret = <Config as CurveConfig>::ScalarField::rand(&mut rng);
        let parameters = Parameters::<Config>::new(&mut rng);
        let mut proof = EqualityProof::construct(&mut rng, &parameters, secret);
        let other_parameters = Parameters::<Config>::new(&mut rng);
        assert!(!proof.verify(&mut rng, &other_parameters));
        // change a commitment
        proof.commitment_to_secret_1 = Config::GENERATOR;
        assert!(!proof.verify(&mut rng, &parameters));
    }
}
