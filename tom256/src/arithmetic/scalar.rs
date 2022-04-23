use super::modular::Modular;
use crate::Curve;

use bigint::{NonZero, U256};

use std::marker::PhantomData;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Scalar<C: Curve>(U256, PhantomData<C>);

/*
impl<C: Curve> Scalar<C> {
    pub fn to_padded_string(&self) -> String {
        todo!();
    }
}
*/

impl<C: Curve> Modular for Scalar<C> {
    const MODULUS: U256 = C::ORDER;

    fn new(number: U256) -> Self {
        let reduced = if number < Self::MODULUS {
            number
        } else {
            // NOTE unwrap is fine here because the modulus
            // can be safely assumed to be nonzero
            number % NonZero::new(Self::MODULUS).unwrap()
        };
        Self(reduced, PhantomData)
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

impl<C: Curve> std::fmt::Display for Scalar<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
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
        const COEFF_A: U256 = U256::ZERO;
        const COEFF_B: U256 = U256::ZERO;
    }

    type ScalarSmall = Scalar<TestCurveSmallMod>;
    type ScalarLarge = Scalar<Secp256k1>;

    #[test]
    fn operations_with_small_modulus() {
        let a = ScalarSmall::new(U256::from_u32(15));
        let b = ScalarSmall::new(U256::from_u32(9));
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
                "805714a252d0c0b58910907e85b5b801fff610a36bdf46847a4bf5d9ae2d10ed"
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
                "ce7c73f82cc708b9080499663f89fda0b52a945427ffcb7c0227adc1b44efe61"
            ))
        );

        // tom curve generator points summed/multiplied using secp256k1 order as modulus
        let a = ScalarLarge::new(Tom256k1::GENERATOR_X);
        let b = ScalarLarge::new(Tom256k1::GENERATOR_Y);
        assert_eq!(
            a + b,
            ScalarLarge::new(U256::from_be_hex(
                "17597ac62cc9e6c8f2e81f1999444584deaddfa028ae4cf888aa58b9538967f5"
            ))
        );
        assert_eq!(
            a * b,
            ScalarLarge::new(U256::from_be_hex(
                "354880368b136b492e8cbce77a7b5ffc3dbef5087bc30537b87ca9d57648c840"
            ))
        );

        let a_min_b = a - b;
        let b_min_a = b - a;
        assert_eq!(a_min_b, -b_min_a);
    }

    #[test]
    fn base16_display() {
        let a = ScalarLarge::new(U256::from_u8(0xb1));
        let b = ScalarLarge::new(U256::from_be_hex(
            "00000000000000000000000000001234567890223451233cbbb101235678677e",
        ));

        let c = ScalarLarge::new(U256::from_be_hex(
            "354880368b136b492e8cbce77a7b5ffc3dbef5087bc30537b87ca9d57648c840",
        ));

        assert_eq!(a.to_string(), "000b1".to_uppercase());
        assert_eq!(b.to_string(), "01234567890223451233cbbb101235678677e".to_uppercase());
        assert_eq!(
            c.to_string(),
            "354880368b136b492e8cbce77a7b5ffc3dbef5087bc30537b87ca9d57648c840".to_uppercase()
        );

        // TODO pad to equal lengths
    }
}
