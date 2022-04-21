use super::modular::Modular;
use crate::Curve;

use bigint::U256;

use std::marker::PhantomData;

#[derive(Clone, Copy, Debug)]
pub struct Scalar<C: Curve>(U256, PhantomData<C>);

impl<C: Curve> Modular for Scalar<C> {
    const MODULUS: U256 = C::ORDER;

    fn new(number: U256) -> Self {
        // NOTE unwrap is fine here because the modulus
        // can be safely assumed to be nonzero
        Self(number % NonZero::new(Self::MODULUS).unwrap(), PhantomData)
    }

    fn inner(&self) -> &U256 {
        &self.0
    }
}

impl<C: Curve> std::ops::Add for Scalar<C> {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Modular::add(&self, &rhs)
    }
}

impl<C: Curve> std::ops::Sub for Scalar<C> {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Modular::sub(&self, &rhs)
    }
}

impl<C: Curve> std::ops::Neg for Scalar<C> {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Modular::neg(&self)
    }
}

impl<C: Curve> std::ops::Mul for Scalar<C> {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        Modular::mul(&self, &rhs)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{Secp256k1, Tom256k1};

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct TestCurveSmallMod;

    impl Curve for TestCurveSmallMod {
        const PRIME_MODULUS: U256 = U256::from_u32(17);
        const ORDER: U256 = U256::ONE;
        const GENERATOR_X: U256 = U256::ZERO;
        const GENERATOR_Y: U256 = U256::ZERO;
    }

    type ScalarSmall = Scalar<TestCurveSmallMod>;
    type ScalarSmall = Scalar<Secp256k1>;

    #[test]
    fn operations_with_small_modulus() {
        let a = ScalarSmall::new(U256::from_u32(15));
        let b = ScalarSmall::new(U256::from_u32(9));
        assert_eq!(&a + &b, ScalarSmall::new(U256::from_u32(7)));
        assert_eq!(a * b, ScalarSmall::new(U256::from_u32(16)));
        assert_eq!(a + b, ScalarSmall::new(U256::from_u32(7)));
        assert_eq!(a - b, ScalarSmall::new(U256::from_u32(6)));
        assert_eq!(b - a, ScalarSmall::new(U256::from_u32(11)));
    }

    #[test]
    fn operations_with_large_modulus() {
        let a = ScalarLarge::new(U256::from_be_hex(
            "000000000000000000000000000000000000000ffffaaaabbbb123456789eeee",
        ));
        let b = ScalarLarge::new(U256::from_be_hex(
            "000000000000000000000000000012345678901234567890ffffddddeeee7890",
        ));
        assert_eq!(
            a + b,
            ScalarLarge::new(U256::from_be_hex(
                "00000000000000000000000000001234567890223451233cbbb101235678677e"
            ))
        );
        assert_eq!(
            a * b,
            ScalarLarge::new(U256::from_be_hex(
                "000123450671f20a8b0a93d71f37ba2ec0d166be8a54889e735d97664ad9f5e0"
            ))
        );
        let a = ScalarLarge::new(Secp256k1::GENERATOR_X);
        let b = ScalarLarge::new(Secp256k1::GENERATOR_Y);
        assert_eq!(
            a + b,
            ScalarLarge::new(U256::from_be_hex(
                "c1f940f620808011b3455e91dc9813afffb3b123d4537cf2f63a51eb1208ec50"
            ))
        );
        assert_eq!(
            a * b,
            ScalarLarge::new(U256::from_be_hex(
                "fd3dc529c6eb60fb9d166034cf3c1a5a72324aa9dfd3428a56d7e1ce0179fd9b"
            ))
        );

        let a_min_b = a - b;
        let b_min_a = b - a;
        assert_eq!(a_min_b, -b_min_a);

        assert_eq!(
            a_min_b,
            ScalarLarge::new(U256::from_be_hex(
                "31838c07d338f746f7fb6699c076025e058448928748d4bfbdaab0cb1be742e0"
            ))
        );
        assert_eq!(
            b_min_a,
            ScalarLarge::new(U256::from_be_hex(
                "ce7c73f82cc708b9080499663f89fda1fa7bb76d78b72b4042554f33e418b94f"
            ))
        );

        // tom curve generator points summed/multiplied using secp256k1 order as modulus
        let a = ScalarLarge::new(Tom256k1::GENERATOR_X);
        let b = ScalarLarge::new(Tom256k1::GENERATOR_Y);
        assert_eq!(
            a + b,
            ScalarLarge::new(U256::from_be_hex(
                "17597ac62cc9e6c8f2e81f1999444583995cbc86d7f6ed34487cb74723bfad07"
            ))
        );
        assert_eq!(
            a * b,
            ScalarLarge::new(U256::from_be_hex(
                "062869f8c96e49475ff3596b7703d46e6183d7f987513f1ede13456a91dbd48e"
            ))
        );

        let a_min_b = a - b;
        let b_min_a = b - a;
        assert_eq!(a_min_b, -b_min_a);
    }
}
