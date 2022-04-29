use crate::arithmetic::multimult::{Multimult, Relation};
use crate::arithmetic::{Modular, Scalar};
use crate::pedersen::*;
use crate::utils::hash_points;
use crate::{Curve, U256};

use rand_core::{CryptoRng, RngCore};

#[derive(Clone, Debug)]
pub struct EqualityProof<C: Curve> {
    commitment_to_random_1: PedersenCommitment<C>,
    commitment_to_random_2: PedersenCommitment<C>,
    mask_secret: Scalar<C>,
    mask_random_1: Scalar<C>,
    mask_random_2: Scalar<C>,
}

impl<C: Curve> EqualityProof<C> {
    const HASH_ID: &'static [u8] = b"equality-proof";

    pub fn construct<R: CryptoRng + RngCore>(
        rng: &mut R,
        pedersen_generator: &PedersenGenerator<C>,
        secret: Scalar<C>,
        commitment_1: PedersenCommitment<C>,
        commitment_2: PedersenCommitment<C>,
    ) -> Self {
        let k = Scalar::random(rng);
        let commitment_to_random_1 = pedersen_generator.commit(rng, k);
        let commitment_to_random_2 = pedersen_generator.commit(rng, k);
        let challenge = hash_points(
            Self::HASH_ID,
            &[
                commitment_1.commitment(),
                commitment_2.commitment(),
                commitment_to_random_1.commitment(),
                commitment_to_random_2.commitment(),
            ],
        );

        let challenge_scalar = Scalar::new(challenge);
        let mask_secret = k - challenge_scalar * secret;
        let mask_random_1 =
            *commitment_to_random_1.randomness() - &challenge_scalar * commitment_1.randomness();
        let mask_random_2 =
            *commitment_to_random_2.randomness() - &challenge_scalar * commitment_2.randomness();
        Self {
            commitment_to_random_1,
            commitment_to_random_2,
            mask_secret,
            mask_random_1,
            mask_random_2,
        }
    }

    pub fn verify<R: CryptoRng + RngCore>(
        &self,
        rng: &mut R,
        pedersen_generator: &PedersenGenerator<C>,
        commitment_1: PedersenCommitment<C>,
        commitment_2: PedersenCommitment<C>,
    ) -> bool {
        let challenge = hash_points(
            Self::HASH_ID,
            &[
                commitment_1.commitment(),
                commitment_2.commitment(),
                self.commitment_to_random_1.commitment(),
                self.commitment_to_random_2.commitment(),
            ],
        );
        let challenge_scalar = Scalar::new(challenge);
        let mut relation_1 = Relation::new();
        let mut relation_2 = Relation::new();
        relation_1.insert(Point::<C>::GENERATOR, self.mask_secret);
        relation_1.insert(
            pedersen_generator.generator().clone(),
            self.commitment_to_random_1,
        );
        relation_1.insert(*commitment_1.commitment(), challenge_scalar);
        relation_1.insert(self.commitment_to_random_1.commitment().neg(), Scalar::ONE);

        relation_2.insert(Point::<C>::GENERATOR, self.mask_secret);
        relation_2.insert(
            pedersen_generator.generator().clone(),
            self.commitment_to_random_2,
        );
        relation_2.insert(*commitment_2.commitment(), challenge_scalar);
        relation_2.insert(self.commitment_to_random_1.commitment().neg(), Scalar::ONE);

        let mut multimult = Multimult::new();
        relation_1.drain(rng, &mut multimult);
        relation_2.drain(rng, &mut multimult);

        multimult.evaluate() == Point::<C>::IDENTITY
    }
}
