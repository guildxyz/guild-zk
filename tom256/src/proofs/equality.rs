use crate::arithmetic::multimult::{MultiMult, Relation};
use crate::arithmetic::{Modular, Point, Scalar};
use crate::pedersen::*;
use crate::utils::hash_points;
use crate::Curve;

use rand_core::{CryptoRng, RngCore};

use std::ops::Neg;

#[derive(Clone, Debug)]
pub struct EqualityProof<C: Curve> {
    commitment_to_random_1: Point<C>,
    commitment_to_random_2: Point<C>,
    mask_secret: Scalar<C>,
    mask_random_1: Scalar<C>,
    mask_random_2: Scalar<C>,
}

impl<C: Curve> EqualityProof<C> {
    const HASH_ID: &'static [u8] = b"equality-proof";

    pub fn construct<R: CryptoRng + RngCore>(
        rng: &mut R,
        pedersen_generator: &PedersenGenerator<C>,
        commitment_1: &PedersenCommitment<C>,
        commitment_2: &PedersenCommitment<C>,
        secret: Scalar<C>,
    ) -> Self {
        let random_scalar = Scalar::random(rng);
        let commitment_to_random_1 = pedersen_generator.commit(rng, random_scalar);
        let commitment_to_random_2 = pedersen_generator.commit(rng, random_scalar);
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
        let mask_secret = random_scalar - challenge_scalar * secret;
        let mask_random_1 =
            *commitment_to_random_1.randomness() - &challenge_scalar * commitment_1.randomness();
        let mask_random_2 =
            *commitment_to_random_2.randomness() - &challenge_scalar * commitment_2.randomness();

        Self {
            commitment_to_random_1: commitment_to_random_1.into_commitment(),
            commitment_to_random_2: commitment_to_random_2.into_commitment(),
            mask_secret,
            mask_random_1,
            mask_random_2,
        }
    }

    pub fn verify<R: CryptoRng + RngCore>(
        &self,
        rng: &mut R,
        pedersen_generator: &PedersenGenerator<C>,
        commitment_1: &Point<C>,
        commitment_2: &Point<C>,
    ) -> bool {
        let challenge = hash_points(
            Self::HASH_ID,
            &[
                commitment_1,
                commitment_2,
                &self.commitment_to_random_1,
                &self.commitment_to_random_2,
            ],
        );
        let challenge_scalar = Scalar::new(challenge);
        let mut relation_1 = Relation::new();
        let mut relation_2 = Relation::new();
        relation_1.insert(Point::<C>::GENERATOR, self.mask_secret);
        relation_1.insert(pedersen_generator.generator().clone(), self.mask_random_1);
        relation_1.insert(commitment_1.clone(), challenge_scalar);
        relation_1.insert((&self.commitment_to_random_1).neg(), Scalar::ONE);

        relation_2.insert(Point::<C>::GENERATOR, self.mask_secret);
        relation_2.insert(pedersen_generator.generator().clone(), self.mask_random_2);
        relation_2.insert(commitment_2.clone(), challenge_scalar);
        relation_2.insert((&self.commitment_to_random_2).neg(), Scalar::ONE);

        let mut multimult = MultiMult::new();
        relation_1.drain(rng, &mut multimult);
        relation_2.drain(rng, &mut multimult);

        multimult.evaluate() == Point::<C>::IDENTITY
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::Tom256k1;
    use rand::rngs::StdRng;
    use rand_core::SeedableRng;

    #[test]
    fn valid_equality_proof() {
        let mut rng = StdRng::from_seed([1; 32]);
        let secret = Scalar::<Tom256k1>::random(&mut rng);
        let pedersen_generator = PedersenGenerator::new(&mut rng);
        let secret_commitment_1 = pedersen_generator.commit(&mut rng, secret);
        let secret_commitment_2 = pedersen_generator.commit(&mut rng, secret);

        let equality_proof = EqualityProof::construct(
            &mut rng,
            &pedersen_generator,
            &secret_commitment_1,
            &secret_commitment_2,
            secret,
        );

        assert!(equality_proof.verify(
            &mut rng,
            &pedersen_generator,
            secret_commitment_1.commitment(),
            secret_commitment_2.commitment(),
        ));
    }

    #[test]
    fn invalid_equality_proof() {
        let mut rng = StdRng::from_seed([1; 32]);
        let secret = Scalar::<Tom256k1>::random(&mut rng);
        let pedersen_generator = PedersenGenerator::new(&mut rng);
        let secret_commitment_1 = pedersen_generator.commit(&mut rng, secret);
        let secret_commitment_2 = pedersen_generator.commit(&mut rng, secret);

        let equality_proof = EqualityProof::construct(
            &mut rng,
            &pedersen_generator,
            &secret_commitment_1,
            &secret_commitment_2,
            secret,
        );

        let invalid_pedersen_generator = PedersenGenerator::new(&mut rng);
        assert!(!equality_proof.verify(
            &mut rng,
            &invalid_pedersen_generator,
            secret_commitment_1.commitment(),
            secret_commitment_2.commitment(),
        ));

        let invalid_secret = Scalar::<Tom256k1>::random(&mut rng);
        let invalid_secret_commitment_1 = pedersen_generator.commit(&mut rng, invalid_secret);
        let invalid_secret_commitment_2 = pedersen_generator.commit(&mut rng, invalid_secret);

        assert!(!equality_proof.verify(
            &mut rng,
            &pedersen_generator,
            invalid_secret_commitment_1.commitment(),
            invalid_secret_commitment_2.commitment(),
        ));
    }
}
