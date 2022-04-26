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

#[cfg(test)]
mod test {
    use super::*;
    use crate::arithmetic::Modular;
    use crate::Tom256k1;
    use bigint::U256;
    use rand::rngs::StdRng;
    use rand_core::SeedableRng;

    #[test]
    fn single_commitment() {
        let mut rng = StdRng::from_seed([17; 32]);
        let p = PedersenGenerator::<Tom256k1>::new(&mut rng);
        let secret = Scalar::new(U256::from_be_hex(
            "d37f628ece72a462f0145cbefe3f0b355ee8332d37acdd83a358016aea029db7",
        ));

        let commitment = p.commit(&mut rng, secret);
        let c = commitment.commitment.into_affine();
        assert_eq!(
            c.x().inner(),
            &U256::from_be_hex("52ceb5ec5cdb13c87b87e60b709b8ad1b8f6b258e5a32f836028651bb9e093fc")
        );
        assert_eq!(
            c.y().inner(),
            &U256::from_be_hex("8edd2b609ccf7c5b942359e7916308062a53a1bda87fb5d2066d71a2751da247")
        );
        assert_eq!(c.z().inner(), &U256::ONE);
    }
}
