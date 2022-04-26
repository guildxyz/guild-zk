use crate::arithmetic::{Point, Scalar};
use crate::Curve;

use rand_core::{CryptoRng, RngCore};

#[derive(Clone, Debug)]
pub struct PedersenGenerator<C: Curve>(Point<C>);

impl<C: Curve> PedersenGenerator<C> {
    pub fn new<R: CryptoRng + RngCore>(rng: &mut R) -> Self {
        let random_scalar = Scalar::random(rng);
        Self(&Point::<C>::GENERATOR * random_scalar)
    }

    pub fn commit<R: CryptoRng + RngCore>(
        &self,
        rng: &mut R,
        secret: Scalar<C>,
    ) -> PedersenCommitment<C> {
        let randomness = Scalar::random(rng);
        let commitment = self
            .0
            .double_mul(&randomness, &Point::<C>::GENERATOR, &secret);

        PedersenCommitment {
            commitment,
            randomness,
        }
    }
}

#[derive(Clone, Debug)]
pub struct PedersenCommitment<C: Curve> {
    commitment: Point<C>,
    randomness: Scalar<C>,
}

impl<C: Curve> PedersenCommitment<C> {
    pub fn commitment(&self) -> &Point<C> {
        &self.commitment
    }

    pub fn randomness(&self) -> &Scalar<C> {
        &self.randomness
    }
}

impl<C: Curve> std::ops::Add<&PedersenCommitment<C>> for &PedersenCommitment<C> {
    type Output = PedersenCommitment<C>;
    fn add(self, rhs: &PedersenCommitment<C>) -> Self::Output {
        PedersenCommitment {
            commitment: &self.commitment + &rhs.commitment,
            randomness: self.randomness + rhs.randomness,
        }
    }
}

impl<C: Curve> std::ops::Sub<&PedersenCommitment<C>> for &PedersenCommitment<C> {
    type Output = PedersenCommitment<C>;
    fn sub(self, rhs: &PedersenCommitment<C>) -> Self::Output {
        PedersenCommitment {
            commitment: &self.commitment - &rhs.commitment,
            randomness: self.randomness - rhs.randomness,
        }
    }
}

impl<C: Curve> std::ops::Mul<&Scalar<C>> for &PedersenCommitment<C> {
    type Output = PedersenCommitment<C>;
    fn mul(self, rhs: &Scalar<C>) -> Self::Output {
        PedersenCommitment {
            commitment: &self.commitment * rhs,
            randomness: &self.randomness * rhs,
        }
    }
}
