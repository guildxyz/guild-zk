use ark_ec::models::short_weierstrass::{Affine, SWCurveConfig};
use ark_std::{rand::Rng, UniformRand};

pub struct Parameters<C: SWCurveConfig>(Affine<C>);

impl<C: SWCurveConfig> Parameters<C> {
    pub fn new<R: Rng + ?Sized>(rng: &mut R) -> Self {
        Self((C::GENERATOR * C::ScalarField::rand(rng)).into())
    }

    pub fn commit(&self, secret: C::ScalarField, randomness: C::ScalarField) -> Affine<C> {
        (C::GENERATOR * secret + self.0 * randomness).into()
    }

    pub fn h(&self) -> Affine<C> {
        self.0
    }
}
