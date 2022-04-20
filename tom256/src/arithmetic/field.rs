use super::modular::Modular;
use crate::Curve;

use bigint::U256;

use std::marker::PhantomData;

#[derive(Clone, Copy, Debug)]
pub struct FieldElement<C: Curve>(U256, PhantomData<C>);

impl<C: Curve> Modular for FieldElement<C> {
    const MODULUS: U256 = C::PRIME_MODULUS;

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

#[cfg(test)]
mod test {
    use super::*;

    struct TestCurve;
    impl Curve for TestCurve {
        const PRIME_MODULUS: U256 = U256::from_be_hex("7");
        const ORDER: U256 = U256::ONE;
        const GENERATOR_X: U256 = U256::ZERO;
        const GENERATOR_Y: U256 = U256::ZERO;
    }

    #[test]
    fn operations() {}
}
