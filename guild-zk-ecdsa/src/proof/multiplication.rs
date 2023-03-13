use crate::eval::{MultiMult, Pair, Relation};
use crate::pedersen::Parameters;
use ark_ec::short_weierstrass::{Affine, SWCurveConfig};
use ark_ff::One;
use ark_std::{rand::Rng, UniformRand};

use core::ops::Neg;

const HASH_ID: &[u8] = b"multiplication proof";

pub struct MultiplicationProof<C: SWCurveConfig> {
    aux_41: Affine<C>,
    aux_42: Affine<C>,
    commitment_4: Affine<C>,
    commitment_to_secret_x: Affine<C>,
    commitment_to_secret_y: Affine<C>,
    commitment_to_secret_z: Affine<C>,
    commitment_to_random_1: Affine<C>,
    commitment_to_random_2: Affine<C>,
    commitment_to_random_3: Affine<C>,
    mask_r: C::ScalarField,
    mask_x: C::ScalarField,
    mask_y: C::ScalarField,
    mask_z: C::ScalarField,
    mask_random_x: C::ScalarField,
    mask_random_y: C::ScalarField,
    mask_random_z: C::ScalarField,
}

impl<C: SWCurveConfig> MultiplicationProof<C> {
    pub fn construct<R: Rng + ?Sized>(
        rng: &mut R,
        parameters: &Parameters<C>,
        secret_x: C::ScalarField,
        secret_y: C::ScalarField,
        secret_z: C::ScalarField,
    ) -> Self {
        let randomness_to_secret_x = C::ScalarField::rand(rng);
        let randomness_to_secret_y = C::ScalarField::rand(rng);
        let randomness_to_secret_z = C::ScalarField::rand(rng);
        let commitment_to_secret_x = parameters.commit(secret_x, randomness_to_secret_x);
        let commitment_to_secret_y = parameters.commit(secret_y, randomness_to_secret_y);
        let commitment_to_secret_z = parameters.commit(secret_z, randomness_to_secret_z);

        let commitment_4 = (commitment_to_secret_y * secret_x).into();
        let randomness_4 = randomness_to_secret_y * secret_x;

        let random_scalar_1 = C::ScalarField::rand(rng);
        let random_scalar_2 = C::ScalarField::rand(rng);
        let random_scalar_3 = C::ScalarField::rand(rng);
        let randomness_to_random_1 = C::ScalarField::rand(rng);
        let randomness_to_random_2 = C::ScalarField::rand(rng);
        let randomness_to_random_3 = C::ScalarField::rand(rng);
        let commitment_to_random_1 = parameters.commit(random_scalar_1, randomness_to_random_1);
        let commitment_to_random_2 = parameters.commit(random_scalar_2, randomness_to_random_2);
        let commitment_to_random_3 = parameters.commit(random_scalar_3, randomness_to_random_3);

        let randomness_to_aux_41 = C::ScalarField::rand(rng);
        let aux_41 = parameters.commit(random_scalar_3, randomness_to_aux_41);
        let aux_42 = (commitment_to_secret_y * random_scalar_1).into();

        let challenge = crate::hash::hash_points(
            HASH_ID,
            &[
                &aux_41,
                &aux_42,
                &commitment_4,
                &commitment_to_secret_x,
                &commitment_to_secret_y,
                &commitment_to_secret_z,
                &commitment_to_random_1,
                &commitment_to_random_2,
                &commitment_to_random_3,
            ],
        );

        let mask_x = random_scalar_1 - challenge * secret_x;
        let mask_y = random_scalar_2 - challenge * secret_y;
        let mask_z = random_scalar_3 - challenge * secret_z;

        let mask_random_x = randomness_to_random_1 - challenge * randomness_to_secret_x;
        let mask_random_y = randomness_to_random_2 - challenge * randomness_to_secret_y;
        let mask_random_z = randomness_to_random_3 - challenge * randomness_to_secret_z;
        let mask_r = randomness_to_aux_41 - challenge * randomness_4;

        Self {
            aux_41,
            aux_42,
            commitment_4,
            commitment_to_secret_x,
            commitment_to_secret_y,
            commitment_to_secret_z,
            commitment_to_random_1,
            commitment_to_random_2,
            commitment_to_random_3,
            mask_r,
            mask_x,
            mask_y,
            mask_z,
            mask_random_x,
            mask_random_y,
            mask_random_z,
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
                &self.aux_41,
                &self.aux_42,
                &self.commitment_4,
                &self.commitment_to_secret_x,
                &self.commitment_to_secret_y,
                &self.commitment_to_secret_z,
                &self.commitment_to_random_1,
                &self.commitment_to_random_2,
                &self.commitment_to_random_3,
            ],
        );

        let mut relation_x = Relation::new();
        let mut relation_y = Relation::new();
        let mut relation_z = Relation::new();
        let mut relation_aux_41 = Relation::new();
        let mut relation_aux_42 = Relation::new();

        relation_x.insert_pair(Pair {
            point: C::GENERATOR,
            scalar: self.mask_x,
        });
        relation_x.insert_pair(Pair {
            point: parameters.h(),
            scalar: self.mask_random_x,
        });
        relation_x.insert_pair(Pair {
            point: self.commitment_to_secret_x,
            scalar: challenge,
        });
        relation_x.insert_pair(Pair {
            point: self.commitment_to_random_1.neg(),
            scalar: C::ScalarField::one(),
        });

        relation_y.insert_pair(Pair {
            point: C::GENERATOR,
            scalar: self.mask_y,
        });
        relation_y.insert_pair(Pair {
            point: parameters.h(),
            scalar: self.mask_random_y,
        });
        relation_y.insert_pair(Pair {
            point: self.commitment_to_secret_y,
            scalar: challenge,
        });
        relation_y.insert_pair(Pair {
            point: self.commitment_to_random_2.neg(),
            scalar: C::ScalarField::one(),
        });

        relation_z.insert_pair(Pair {
            point: C::GENERATOR,
            scalar: self.mask_z,
        });
        relation_z.insert_pair(Pair {
            point: parameters.h(),
            scalar: self.mask_random_z,
        });
        relation_z.insert_pair(Pair {
            point: self.commitment_to_secret_z,
            scalar: challenge,
        });
        relation_z.insert_pair(Pair {
            point: self.commitment_to_random_3.neg(),
            scalar: C::ScalarField::one(),
        });

        relation_aux_41.insert_pair(Pair {
            point: C::GENERATOR,
            scalar: self.mask_z,
        });
        relation_aux_41.insert_pair(Pair {
            point: parameters.h(),
            scalar: self.mask_r,
        });
        relation_aux_41.insert_pair(Pair {
            point: self.commitment_4,
            scalar: challenge,
        });
        relation_aux_41.insert_pair(Pair {
            point: self.aux_41.neg(),
            scalar: C::ScalarField::one(),
        });

        relation_aux_42.insert_pair(Pair {
            point: self.commitment_to_secret_y,
            scalar: self.mask_x,
        });
        relation_aux_42.insert_pair(Pair {
            point: self.commitment_4,
            scalar: challenge,
        });
        relation_aux_42.insert_pair(Pair {
            point: self.aux_42.neg(),
            scalar: C::ScalarField::one(),
        });

        relation_x.drain(rng, multimult);
        relation_y.drain(rng, multimult);
        relation_z.drain(rng, multimult);
        relation_aux_41.drain(rng, multimult);
        relation_aux_42.drain(rng, multimult);
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
    use ark_ec::CurveConfig;
    use ark_secq256k1::Config;
    use ark_std::{
        rand::{rngs::StdRng, SeedableRng},
        UniformRand,
    };

    const SEED: u64 = 1234567890;

    #[test]
    fn completeness() {
        let mut rng = StdRng::seed_from_u64(SEED);
        let x = <Config as CurveConfig>::ScalarField::rand(&mut rng);
        let y = <Config as CurveConfig>::ScalarField::rand(&mut rng);
        let z = x * y;
        let parameters = Parameters::<Config>::new(&mut rng);
        let proof = MultiplicationProof::construct(&mut rng, &parameters, x, y, z);
        assert!(proof.verify(&mut rng, &parameters));
    }

    #[test]
    fn soundness() {
        let mut rng = StdRng::seed_from_u64(SEED);
        let x = <Config as CurveConfig>::ScalarField::rand(&mut rng);
        let y = <Config as CurveConfig>::ScalarField::rand(&mut rng);
        let z = x * y;
        let parameters = Parameters::<Config>::new(&mut rng);
        let mut proof = MultiplicationProof::construct(&mut rng, &parameters, x, y, z);
        // change parameters
        let other_parameters = Parameters::<Config>::new(&mut rng);
        assert!(!proof.verify(&mut rng, &other_parameters));
        // invalid commitment
        proof.commitment_to_secret_x = Config::GENERATOR;
        assert!(!proof.verify(&mut rng, &parameters));
        // invalid multiplication
        let other_z = <Config as CurveConfig>::ScalarField::rand(&mut rng);
        let proof = MultiplicationProof::construct(&mut rng, &parameters, x, y, other_z);
        assert!(!proof.verify(&mut rng, &parameters));
    }
}
