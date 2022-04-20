use super::modular::Modular;
use crate::Curve;

use bigint::U256;

use std::marker::PhantomData;

#[derive(Clone, Copy, Debug)]
pub struct Scalar<C: Curve>(U256, PhantomData<C>);

impl<C: Curve> Modular for Scalar<C> {
    const MODULUS: U256 = C::ORDER;

    fn new(number: U256) -> Self {
        Self(
            number.reduce(&Self::MODULUS).unwrap_or_else(|| U256::ZERO),
            PhantomData,
        )
    }

    fn inner(&self) -> &U256 {
        &self.0
    }
}
