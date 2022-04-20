use super::modular::Modular;
use crate::Curve;

use bigint::U256;

use std::marker::PhantomData;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

impl<'a, 'b, C: Curve> std::ops::Add<&'b FieldElement<C>> for &'a FieldElement<C> {
    type Output = FieldElement<C>;
    fn add(self, rhs: &'b FieldElement<C>) -> Self::Output {
        Modular::add(self, rhs)
    }
}

impl<C: Curve> std::ops::Add for FieldElement<C> {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Modular::add(&self, &rhs)
    }
}

impl<C: Curve> std::ops::Mul for FieldElement<C> {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        Modular::mul(&self, &rhs)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct TestCurveSmallMod;

    impl Curve for TestCurveSmallMod {
        const PRIME_MODULUS: U256 = U256::from_u32(17);
        const ORDER: U256 = U256::ONE;
        const GENERATOR_X: U256 = U256::ZERO;
        const GENERATOR_Y: U256 = U256::ZERO;
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct TestCurveLargeMod;

    impl Curve for TestCurveLargeMod {
        const PRIME_MODULUS: U256 =
            U256::from_be_hex("fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2f");
        const ORDER: U256 = U256::ONE;
        const GENERATOR_X: U256 = U256::ZERO;
        const GENERATOR_Y: U256 = U256::ZERO;
    }

    type FeSmall = FieldElement<TestCurveSmallMod>;
    type FeLarge = FieldElement<TestCurveLargeMod>;

    #[test]
    fn operations_with_small_modulus() {
        let a = FeSmall::new(U256::from_u32(15));
        let b = FeSmall::new(U256::from_u32(9));
        assert_eq!(&a + &b, FeSmall::new(U256::from_u32(7)));
        assert_eq!(a * b, FeSmall::new(U256::from_u32(16)));
        assert_eq!(a + b, FeSmall::new(U256::from_u32(7)));
    }

    #[test]
    fn operations_with_large_modulus() {
    }
}
