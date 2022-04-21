use super::modular::Modular;
use crate::Curve;

use bigint::{NonZero, U256};

use std::marker::PhantomData;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FieldElement<C: Curve>(U256, PhantomData<C>);

impl<C: Curve> Modular for FieldElement<C> {
    const MODULUS: U256 = C::PRIME_MODULUS;

    fn new(number: U256) -> Self {
        // NOTE unwrap is fine here because the modulus
        // can be safely assumed to be nonzero
        Self(number % NonZero::new(Self::MODULUS).unwrap(), PhantomData)
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

impl<C: Curve> std::ops::Sub for FieldElement<C> {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Modular::sub(&self, &rhs)
    }
}

impl<C: Curve> std::ops::Neg for FieldElement<C> {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Modular::neg(&self)
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
    use crate::Secp256k1;

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct TestCurveSmallMod;

    impl Curve for TestCurveSmallMod {
        const PRIME_MODULUS: U256 = U256::from_u32(17);
        const ORDER: U256 = U256::ONE;
        const GENERATOR_X: U256 = U256::ZERO;
        const GENERATOR_Y: U256 = U256::ZERO;
    }

    type FeSmall = FieldElement<TestCurveSmallMod>;
    type FeLarge = FieldElement<Secp256k1>;

    #[test]
    fn operations_with_small_modulus() {
        let a = FeSmall::new(U256::from_u32(15));
        let b = FeSmall::new(U256::from_u32(9));
        assert_eq!(&a + &b, FeSmall::new(U256::from_u32(7)));
        assert_eq!(a * b, FeSmall::new(U256::from_u32(16)));
        assert_eq!(a + b, FeSmall::new(U256::from_u32(7)));
        assert_eq!(a - b, FeSmall::new(U256::from_u32(6)));
        assert_eq!(b - a, FeSmall::new(U256::from_u32(11)));
    }

    #[test]
    fn operations_with_large_modulus() {
        let a = FeLarge::new(U256::from_be_hex(
            "000000000000000000000000000000000000000ffffaaaabbbb123456789eeee",
        ));
        let b = FeLarge::new(U256::from_be_hex(
            "000000000000000000000000000012345678901234567890ffffddddeeee7890",
        ));
        assert_eq!(
            a + b,
            FeLarge::new(U256::from_be_hex(
                "00000000000000000000000000001234567890223451233cbbb101235678677e"
            ))
        );
        assert_eq!(
            a * b,
            FeLarge::new(U256::from_be_hex(
                "000123450671f20a8b0a93d71f37ba2ec0d166be8a54889e735d97664ad9f5e0"
            ))
        );
        let a = FeLarge::new(Secp256k1::GENERATOR_X);
        let b = FeLarge::new(Secp256k1::GENERATOR_Y);
        assert_eq!(
            a + b,
            FeLarge::new(U256::from_be_hex(
                "c1f940f620808011b3455e91dc9813afffb3b123d4537cf2f63a51eb1208ec50"
            ))
        );
        assert_eq!(
            a * b,
            FeLarge::new(U256::from_be_hex(
                "fd3dc529c6eb60fb9d166034cf3c1a5a72324aa9dfd3428a56d7e1ce0179fd9b"
            ))
        );
    }
}
